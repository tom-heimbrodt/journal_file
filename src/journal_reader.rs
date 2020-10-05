use std::io::{Read, Seek, SeekFrom, BufReader, ErrorKind};
use std::fs::File;
use std::marker::PhantomData;
use std::fmt::Debug;

use crate::*;

pub trait JournalDeserialize<T> : Copy + 'static {
    type Error: std::error::Error;
    fn deserialize(&self, reader: &mut dyn Read) -> Result<Option<T>, Self::Error>;
}

//impl<T, X, Y> JournalDeserialize<T> for Y
//where X: JournalDeserialize<T>,
//      Y: Deref<Target=X> + Copy {
//    type Error = X::Error;
//
//    fn deserialize(&self, reader: &mut dyn Read) -> Result<Option<T>, Self::Error> {
//        (**self).deserialize(reader)
//    }
//}

//pub trait JournalRead<'a, T, D> : IntoIterator<Item=Result<JournalEntry<T>, JournalError<D::Error>>>
//where D: JournalDeserialize<T> {
//    type IntoIter: Iterator<Item=Result<JournalEntry<T>, JournalError<D::Error>>>;
//    fn iter(&'a mut self) -> <Self as JournalRead<'a, T, D>>::IntoIter;
//}

#[derive(Debug)]
pub struct JournalReader<'a, T, D> {
    file_handle: Option<OwnedOrRef<'a, File>>,
    deserializer: D,
    type_phantom: PhantomData<*const T>,
    seek: bool,
} 

#[derive(Debug, Clone, Copy)]
pub struct BincodeDeserializer;

impl<T> JournalDeserialize<T> for BincodeDeserializer
where T: for<'de> serde::Deserialize<'de> {
    type Error = bincode::Error;

    fn deserialize(&self, reader: &mut dyn Read) -> Result<Option<T>, Self::Error> {
        use bincode::config::Options;
        match bincode::options()
            .with_varint_encoding()
            .allow_trailing_bytes()
            .deserialize_from(reader) {                
            
            Ok(value) => Ok(Some(value)),
            Err(err) => {
                if let bincode::ErrorKind::Io(ref io_err) = *err {
                    if io_err.kind() == ErrorKind::UnexpectedEof {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl<'a, T> JournalReader<'a, T, BincodeDeserializer>
where T: for<'de> serde::Deserialize<'de> + Debug {
    pub fn new<FILE>(file_handle: FILE) -> Self
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        Self::with_deserializer(file_handle, BincodeDeserializer)
    }
}

impl<'a, T, D> JournalReader<'a, T, D>
where D: JournalDeserialize<T> + Debug, T: Debug {
    pub fn with_deserializer<FILE>(file_handle: FILE, deserializer: D) -> Self
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        Self {
            seek: true,
            type_phantom: PhantomData::default(),
            deserializer,
            file_handle: Some(file_handle.into()),
        }
    }

    pub fn seek_on_iter_start(&mut self, seek: bool) {
        self.seek = seek;
    }

    pub fn iter<'outer>(&'outer mut self) -> JournalReaderIterUnwrapped<'a, 'outer, T, D> {
        self.into_iter()
    }

    pub(crate) fn iter_entries<'outer>(&'outer mut self) -> JournalReaderIter<'a, 'outer, T, D> {
        JournalReaderIter {
            seek: self.seek,
            reader: OwnedOrRef::Ref(self),
            buf_reader: None,
        }
    }

    pub fn into_iter<'outer>(self) -> JournalReaderIterUnwrapped<'a, 'outer, T, D> {
        JournalReaderIterUnwrapped(JournalReaderIter {
            seek: self.seek,
            reader: OwnedOrRef::Owned(self),
            buf_reader: None,
        })
    }
}

pub(crate) struct JournalReaderIter<'inner, 'outer, T, D> {
    pub(crate) reader: OwnedOrRef<'outer, JournalReader<'inner, T, D>>,
    pub(crate) buf_reader: Option<CountingIO<BufReader<OwnedOrRef<'inner, File>>>>,
    pub(crate) seek: bool,
}

pub struct JournalReaderIterUnwrapped<'inner, 'outer, T, D>(JournalReaderIter<'inner, 'outer, T, D>);

impl<'inner, 'outer, T, D> Drop for JournalReaderIter<'inner, 'outer, T, D> {
    fn drop(&mut self) {
        if self.reader.as_mut().file_handle.is_none() {
            let file_handle = self.buf_reader.take().map(|x| x.into_inner().into_inner());
            self.reader.as_mut().file_handle = file_handle;
        } 
    }
}

impl<'inner, 'outer, T, D> Iterator for JournalReaderIter<'inner, 'outer, T, D>
where D: JournalDeserialize<T> + Debug, T: Debug {
    type Item = Result<JournalEntry<T>, JournalError<D::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut reader = if let Some(reader) = self.buf_reader.take() {
            reader
        } else {
            let mut file = self.reader.as_mut().file_handle.take().unwrap();
            if self.seek {
                file.as_mut().seek(SeekFrom::Start(0)).unwrap();
            }
            CountingIO::new(BufReader::new(file)).without_offset()
        };

        let start_offset = reader.position().unwrap();
        let result = self.reader.as_mut().deserializer.deserialize(&mut reader);
        let end_offset = reader.position().unwrap();

        self.buf_reader = Some(reader);

        match result {
            Ok(Some(value)) => Some(Ok(JournalEntry::new(value, start_offset))),
            Ok(None) => {
                // we are at EOF, but we want to make shure that there were no trailing bytes,
                // since that would mean the last write operation wasn't successful
                if start_offset == end_offset {
                    None
                } else {
                    Some(Err(JournalError::IOError(std::io::Error::new(ErrorKind::UnexpectedEof, "Journal file is dirty"))))
                }
            },
            Err(err) => Some(Err(JournalError::SerializationError(err)))
        }
    }
}

impl<'inner, 'outer, T, D> Iterator for JournalReaderIterUnwrapped<'inner, 'outer, T, D>
where D: JournalDeserialize<T> + Debug, T: Debug {
    type Item = Result<T, JournalError<D::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let myself = &mut self.0;
        let mut reader = if let Some(reader) = myself.buf_reader.take() {
            reader
        } else {
            let mut file = myself.reader.as_mut().file_handle.take().unwrap();
            if myself.seek {
                file.as_mut().seek(SeekFrom::Start(0)).unwrap();
            }
            CountingIO::new(BufReader::new(file)).without_offset()
        };

        let start_offset = reader.position().unwrap();
        let result = myself.reader.as_mut().deserializer.deserialize(&mut reader);
        let end_offset = reader.position().unwrap();

        myself.buf_reader = Some(reader);

        match result {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => {
                // we are at EOF, but we want to make shure that there were no trailing bytes,
                // since that would mean the last write operation wasn't successful
                if start_offset == end_offset {
                    None
                } else {
                    Some(Err(JournalError::IOError(std::io::Error::new(ErrorKind::UnexpectedEof, "Journal file is dirty"))))
                }
            },
            Err(err) => Some(Err(JournalError::SerializationError(err)))
        }
    }
}

impl<'inner, 'outer, T, D> IntoIterator for &'outer mut JournalReader<'inner, T, D>
where D: JournalDeserialize<T> + Debug, T: Debug {
    type Item = <JournalReaderIterUnwrapped<'inner, 'outer, T, D> as Iterator>::Item;
    type IntoIter = JournalReaderIterUnwrapped<'inner, 'outer, T, D>;

    fn into_iter(self) -> Self::IntoIter {
        JournalReaderIterUnwrapped(JournalReaderIter {
            seek: self.seek,
            reader: OwnedOrRef::Ref(self),
            buf_reader: None,
        })
    }
}

//impl<T, D> IntoIterator for JournalReader<'static, T, D>
//where D: JournalDeserialize<T> + Debug + 'static,
//      T: Debug + 'static {
//
//    type Item = <JournalReaderIter<'static, 'static, T, D> as Iterator>::Item;
//
//    type IntoIter = JournalReaderIter<'static, 'static, T, D>;
//
//    fn into_iter(self) -> JournalReaderIter<'static, 'static, T, D> {
//        JournalReaderIter {
//            seek: self.seek,
//            reader: OwnedOrRef::Owned(self),
//            buf_reader: None,
//        }
//    }
//}

//impl<'a, T, D> JournalRead<'a, T, D> for JournalReader<'a, T, D>
//where D: JournalDeserialize<T> + Debug + 'a,
//      T: Debug + 'a {
//    type IntoIter = JournalReaderIter<'a, 'static, T, D>;
//
//    fn iter(&'a mut self) -> <Self as JournalRead<'a, T, D>>::IntoIter {
//        IntoIterator::into_iter(self)
//    }
//}