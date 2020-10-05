use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use std::marker::PhantomData;
use std::fmt::Debug;

use crate::*;

pub trait JournalSerialize<T> : Copy + 'static {
    type Error: std::error::Error;
    fn serialize(&self, value: T, writer: &mut dyn Write) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct JournalWriter<'a, T, S> {
    file_handle: OwnedOrRef<'a, File>,
    serializer: S,
    type_phantom: PhantomData<*const T>,
} 

#[derive(Debug, Copy, Clone)]
pub struct BincodeSerializer;

impl<T> JournalSerialize<T> for BincodeSerializer
where T: serde::Serialize {
    type Error = bincode::Error;

    fn serialize(&self, value: T, writer: &mut dyn Write) -> Result<(), Self::Error> {
        use bincode::Options;
        bincode::options()
            .with_varint_encoding()
            .serialize_into(writer, &value)
    }
}

impl<'a, T> JournalWriter<'a, T, BincodeSerializer>
where T: serde::Serialize + Debug {
    pub fn new<FILE>(file_handle: FILE) -> Self 
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        Self::with_serializer(file_handle, BincodeSerializer)
    }
}

impl<'a, T, S> JournalWriter<'a, T, S>
where S: JournalSerialize<T> + Debug, T: Debug {
    pub fn with_serializer<FILE>(file_handle: FILE, serializer: S) -> Self 
    where FILE: Into<OwnedOrRef<'a, File>> + 'a {
        Self {
            type_phantom: PhantomData::default(),
            serializer,
            file_handle: file_handle.into(),
        }
    }

    pub fn store_entry(&mut self, entry: T) -> Result<(), JournalError<S::Error>> {
        self.file_handle.seek(SeekFrom::End(0))
            .map_err(|err| JournalError::IOError(err))?;
        self.serializer.serialize(entry, self.file_handle.as_mut())
            .map_err(|err| JournalError::SerializationError(err))
    }

    pub fn store_entries<I>(&mut self, entries: I) -> Result<(), JournalError<S::Error>> 
    where I: Iterator<Item=T> {
        self.file_handle.seek(SeekFrom::End(0))
            .map_err(|err| JournalError::IOError(err))?;
        
        for entry in entries {
            self.serializer.serialize(entry, self.file_handle.as_mut())
                .map_err(|err| JournalError::SerializationError(err))?;
        }

        Ok(())
    }
}