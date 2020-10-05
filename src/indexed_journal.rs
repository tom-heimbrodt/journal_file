use std::io::{Seek, SeekFrom, ErrorKind, BufReader};
use std::fs::File;
use std::marker::PhantomData;
use std::fmt::Debug;

use crate::*;
use crate::journal_writer::*;
use crate::journal_reader::*;

#[derive(Debug)]
pub struct IndexedJournal<'a, T, S, D> {
    file_handle: Option<OwnedOrRef<'a, File>>,
    serializer: S,
    deserializer: D,
    index: JournalIndex,
    type_phantom: PhantomData<*const T>,
} 

#[derive(Debug)]
struct JournalIndex {
    entry_indices: Vec<u64>,
}

impl JournalIndex {
    fn entry_offset(&self, entry_index: usize) -> Result<u64, ()> {
        if entry_index >= self.entry_indices.len() {
            Err(())
        } else {
            Ok(self.entry_indices[entry_index])
        }
    }

    fn build<D, T>(file: &mut File, deserializer: &D) -> Result<Self, JournalError<D::Error>>
    where D: JournalDeserialize<T> + Debug,
          T: Debug {

        JournalReader::with_deserializer(file, *deserializer)
            .iter_entries()
            .map(|entry_result| {
                entry_result.map(|entry| entry.offset)
            })
            .collect::<Result<Vec<u64>, JournalError<D::Error>>>()
            .map(|entry_indices| {
                Self {
                    entry_indices,
                }
            })
   }
}

impl<'a, T> IndexedJournal<'a, T, BincodeSerializer, BincodeDeserializer>
where T: serde::Serialize + for<'de> serde::Deserialize<'de> + Debug {
    pub fn new<FILE>(file_handle: FILE) -> Result<Self, JournalError<<BincodeDeserializer as JournalDeserialize<T>>::Error>> 
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        Self::with_serializer(file_handle, BincodeSerializer, BincodeDeserializer)
    }
}


impl<'a, T, S, D> IndexedJournal<'a, T, S, D>
where S: JournalSerialize<T> + Debug,
      D: JournalDeserialize<T> + Debug,
      T: Debug {
    pub fn with_serializer<FILE>(file_handle: FILE, serializer: S, deserializer: D) -> Result<Self, JournalError<D::Error>> 
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        let mut file_handle = file_handle.into();
        Ok(Self {
            index: JournalIndex::build(file_handle.as_mut(), &deserializer)?,
            type_phantom: PhantomData::default(),
            serializer,
            deserializer,
            file_handle: Some(file_handle),
        })
    }

    pub fn iter<'outer>(&'outer mut self) -> IndexedJournalIter<'a, 'outer, T, S, D> {
        IndexedJournalIter {
            buf_reader: None,
            outer: self,
            seek: true,
            type_phantom: Default::default(),
        }
    }

    pub fn load_entry(&mut self, index: usize) -> Result<T, JournalError<D::Error>> {
        let offset = self.index.entry_offset(index)
            .map_err(|()| JournalError::IndexOutOfBounds)?;
        
        self.file_handle.as_mut().unwrap().seek(SeekFrom::Start(offset))
            .map_err(|err| JournalError::IOError(err))?;

        match self.deserializer.deserialize(self.file_handle.as_mut().unwrap()) {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(JournalError::IOError(std::io::Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF"))),
            Err(x) => Err(JournalError::SerializationError(x)),
        }
    }

    pub fn iter_from<'outer>(&'outer mut self, index: usize) -> Result<IndexedJournalIter<'a, 'outer, T, S, D>, JournalError<D::Error>> {
        let offset = self.index.entry_offset(index)
            .map_err(|()| JournalError::IndexOutOfBounds)?;
        
        self.file_handle.as_mut().unwrap().seek(SeekFrom::Start(offset))
            .map_err(|err| JournalError::IOError(err))?;

        //let mut reader = JournalReader::with_deserializer(&mut self.file_handle, self.deserializer);
        //reader.seek_on_iter_start(false);
        //Ok(reader.into_iter())
        //Ok(JournalReaderIter {
        //    seek: false,
        //    reader: OwnedOrRef::Owned(JournalReader::with_deserializer(self.file_handle.as_mut().unwrap(), self.deserializer)),
        //    buf_reader: None,
        //})
        Ok(IndexedJournalIter {
            buf_reader: None,
            outer: self,
            seek: false,
            type_phantom: Default::default(),
        })
    }

    pub fn store_entry(&mut self, entry: T) -> Result<(), JournalError<S::Error>> {
        let mut writer = JournalWriter::with_serializer(
            self.file_handle.as_mut().unwrap().as_mut(), self.serializer);
        writer.store_entry(entry)
    }

    pub fn store_entries<I>(&mut self, entries: I) -> Result<(), JournalError<S::Error>> 
    where I: Iterator<Item=T> {
        let mut writer = JournalWriter::with_serializer(
            self.file_handle.as_mut().unwrap().as_mut(), self.serializer);
        writer.store_entries(entries)
    }
}

impl<'inner, 'outer, T, S, D> IntoIterator for &'outer mut IndexedJournal<'inner, T, S, D>
where D: JournalDeserialize<T> + Debug + 'inner,
      T: Debug + 'inner {
    type Item = <IndexedJournalIter<'inner, 'outer, T, S, D> as Iterator>::Item;
    type IntoIter = IndexedJournalIter<'inner, 'outer, T, S, D>;

    fn into_iter(self) -> IndexedJournalIter<'inner, 'outer, T, S, D> {
        IndexedJournalIter {
            buf_reader: None,
            outer: self,
            seek: true,
            type_phantom: Default::default(),
        }
        //JournalReaderIter {
        //    seek: true,
        //    reader: OwnedOrRef::Owned(reader),
        //    buf_reader: None,
        //}
    }
}

pub struct IndexedJournalIter<'inner, 'outer, T, S, D> {
    pub(crate) outer: &'outer mut IndexedJournal<'inner, T, S, D>,
    pub(crate) buf_reader: Option<CountingIO<BufReader<OwnedOrRef<'inner, File>>>>,
    pub(crate) seek: bool,
    pub(crate) type_phantom: PhantomData<*const T>,
}

impl<'inner, 'outer, T, S, D> Drop for IndexedJournalIter<'inner, 'outer, T, S, D> {
    fn drop(&mut self) {
        if self.outer.file_handle.is_none() {
            let file_handle = self.buf_reader.take().map(|x| x.into_inner().into_inner());
            self.outer.file_handle = file_handle;
        } 
    }
}

impl<'inner, 'outer, T, S, D> Iterator for IndexedJournalIter<'inner, 'outer, T, S, D>
where D: JournalDeserialize<T> + Debug, T: Debug {
    type Item = Result<T, JournalError<D::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf_reader.is_none() {
            self.buf_reader = Some(CountingIO::new(BufReader::new(self.outer.file_handle.take().unwrap())).without_offset())
        }

        let buf_reader = self.buf_reader.as_mut().unwrap();

        if self.seek {
            buf_reader.seek(SeekFrom::Start(0)).unwrap();
            self.seek = false;
        }

        let start_offset = buf_reader.position().unwrap();
        let result = self.outer.deserializer.deserialize(buf_reader);
        let end_offset = buf_reader.position().unwrap();

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
