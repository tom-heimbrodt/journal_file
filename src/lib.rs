use std::io::{Read, Seek, SeekFrom};
use std::ops::{DerefMut, Deref};


mod counting_io;
use counting_io::*;

mod owned_or_ref;
pub use owned_or_ref::*;

pub mod indexed_journal;
use indexed_journal::*;
pub type SimpleIndexedJournal<'a, T> = IndexedJournal<'a, T, BincodeSerializer, BincodeDeserializer>;


pub mod journal_writer;
use journal_writer::*;
pub type SimpleJournalWriter<'a, T> = JournalWriter<'a, T, BincodeSerializer>;

pub mod journal_reader;
use journal_reader::*;
pub type SimpleJournalReader<'a, T> = JournalReader<'a, T, BincodeDeserializer>;

#[derive(Debug)]
pub enum JournalError<SE> {
    IndexOutOfBounds,
    IOError(std::io::Error),
    SerializationError(SE),
}

#[derive(Debug)]
pub(crate) struct JournalEntry<T> {
    pub value: T,
    pub offset: u64,
}

impl<T> JournalEntry<T> {
    pub(crate) fn new(value: T, offset: u64) -> Self {
        Self { value, offset }
    }
}

impl<T> Deref for JournalEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for JournalEntry<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
//    use super::*;
//    use std::path::Path;
//    use std::fs::File;
//
    #[test]
    fn test_example_01() {
    }
}
//    pub fn test_journal_writer() {
//        let mut raw_file = File::with_options()
//            .create(true)
//            .truncate(true)
//            .read(true)
//            .write(true)
//            .open(Path::new("test"))
//            .unwrap();
//        let mut file = journal_writer::JournalWriter::new(&mut raw_file);
//
//        file.store_entry("hello".to_string()).unwrap();
//        file.store_entry("world".to_string()).unwrap();
//        file.store_entry("3".to_string()).unwrap();
//        file.store_entry("4".to_string()).unwrap();
//        file.store_entry("5".to_string()).unwrap();
//        file.store_entry("6".to_string()).unwrap();
//        //for _ in 0..100 {
//        //    large_test.push_str("some longer test here just to test some larger entries lalala tata hhhhhhmmmmmmmmmmmmmmmmmmmmmmm");
//        //}
//        //file.store_entry(large_test).unwrap();
//
//        //let mut file: JournalReader<String, _> = journal_reader::JournalReader::new(&mut raw_file);
//        //for value in file.iter() {
//        //    dbg!(value);
//        //}
//
//        //let mut file: JournalReader<String, _> = journal_reader::JournalReader::new(&mut raw_file).unwrap();
//        //for value in &mut file {
//        //    dbg!(value);
//        //}
//
//        let mut file: IndexedJournal<String, _, _> = IndexedJournal::new_owned(raw_file).unwrap();
//        dbg!(file.load_entry(1));
//        dbg!(file.load_entry(3));
//        for value in file.iter_from(2).unwrap() {
//            dbg!(value);
//        }
//    }
////    #[test]
////    fn test() {
////        let mut file = JournalFile::new(
////            File::with_options().create(true).truncate(true).read(true).write(true).open(Path::new("test")).unwrap())
////            .unwrap();
////        let entry1 = "Hello World!".to_string();
////        let entry2 = "Another Test Entry.".to_string();
////        let entry3 = "".to_string();
////        let entry4 = "The final Test Entry".to_string();
////        file.store_entry(entry1.clone()).unwrap();
////        file.store_entry(entry2.clone()).unwrap();
////        file.store_entry(entry3.clone()).unwrap();
////        file.store_entry(entry4.clone()).unwrap();
////
////        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
////        dbg!(&file);
////        assert_eq!(file.load_entry(1).unwrap(), entry2);
////        assert_eq!(file.load_entry(2).unwrap(), entry3);
////        assert_eq!(file.load_entry(0).unwrap(), entry1);
////        assert_eq!(file.load_entry(3).unwrap(), entry4);
////        assert_eq!(file.load_entry(0).unwrap(), entry1);
////
////        let mut file: JournalFile<String, _> = JournalFile::new(File::open(Path::new("test")).unwrap())
////        .unwrap();
////
////        assert_eq!(file.load_entry(1).unwrap(), entry2);
////        assert_eq!(file.load_entry(2).unwrap(), entry3);
////        assert_eq!(file.load_entry(0).unwrap(), entry1);
////        assert_eq!(file.load_entry(3).unwrap(), entry4);
////        assert_eq!(file.load_entry(0).unwrap(), entry1);
////    }
////}
//////
//////use std::io;
//////use std::io::{Write, Read, Seek, SeekFrom};
//////use std::fs::{File, OpenOptions};
//////use std::collections::HashMap;
//////use std::path::{Path};
//////use std::mem::size_of;
//////
//////pub struct StructuredFile {
//////    file: File,
//////    byte_offset: u64,
//////    next_id: u64,
//////    indices: HashMap<u64, u64>,
//////} 
//////
//////impl StructuredFile {
//////    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
//////        let size = data.len() as u64;
//////        let size_bytes = size.to_be_bytes();
//////        self.file.seek(SeekFrom::End(0))?;
//////        self.file.write_all(&size_bytes)?;
//////        self.file.write_all(&data)?;
//////        self.indices.insert(self.next_id, self.byte_offset);
//////        self.next_id += 1;
//////        self.byte_offset += size + size_of::<u64>() as u64;
//////        Ok(())
//////    }
//////
//////    pub fn create(path: &Path) -> io::Result<Self> {
//////        Ok(Self {
//////            file: OpenOptions::new()
//////                .truncate(true)
//////                .create(true)
//////                .read(true)
//////                .write(true)
//////                .open(path)?,
//////            byte_offset: 0,
//////            next_id: 0,
//////            indices: HashMap::new(),
//////        })
//////    }
//////
//////    pub fn open(path: &Path) -> io::Result<Self> {
//////        let mut myself = Self {
//////            file: OpenOptions::new()
//////                .create(false)
//////                .read(true)
//////                .write(true)
//////                .open(path)?,
//////            byte_offset: 0,
//////            next_id: 0,
//////            indices: HashMap::new(),
//////        };
//////        myself.initialize()?;
//////        Ok(myself)
//////    }
//////
//////    pub fn open_or_create(path: &Path) -> io::Result<Self> {
//////        let mut myself = Self {
//////            file: OpenOptions::new()
//////                .create(true)
//////                .read(true)
//////                .write(true)
//////                .open(path)?,
//////            byte_offset: 0,
//////            next_id: 0,
//////            indices: HashMap::new(),
//////        };
//////        myself.initialize()?;
//////        Ok(myself)
//////    }
//////
//////    fn initialize(&mut self) -> io::Result<()> {
//////        let stream_size = self.file.seek(SeekFrom::End(0))?;
//////        if stream_size == 0 {
//////            return Ok(());
//////        } 
//////        self.file.seek(SeekFrom::Start(0))?;
//////
//////        loop {
//////            let mut size_bytes = [0u8; size_of::<u64>()];
//////            self.file.read_exact(&mut size_bytes)?;
//////
//////            let size = u64::from_be_bytes(size_bytes);
//////
//////            self.file.seek(SeekFrom::Current(size as i64))?;
//////
//////            self.indices.insert(self.next_id, self.byte_offset);
//////            self.next_id += 1;
//////            self.byte_offset += size_of::<u64>() as u64 + size;
//////
//////            if self.byte_offset > stream_size {
//////                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, 
//////                    "Unexpected EOF inside data chunk."));
//////            } else if self.byte_offset == stream_size {
//////                return Ok(());
//////            }
//////        }
//////    }
//////
//////    pub fn read(&mut self, index: u64) -> io::Result<Vec<u8>> {
//////        let addr = self.indices.get(&index).ok_or(io::Error::new(
//////            io::ErrorKind::InvalidInput,
//////            "The requested chunk index was not found.",
//////        ))?;
//////        self.file.seek(SeekFrom::Start(*addr))?;
//////        let mut size_bytes = [0u8; size_of::<u64>()];
//////        self.file.read_exact(&mut size_bytes)?;
//////
//////        let size = u64::from_be_bytes(size_bytes) as usize;
//////        let mut buffer = vec![0; size];
//////        self.file.read_exact(&mut buffer)?;
//////        Ok(buffer)
//////    }
//}