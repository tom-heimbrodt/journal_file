use std::io::{Read, Seek, SeekFrom};
use std::ops::{DerefMut, Deref};

#[derive(Debug)]
pub enum OwnedOrRef<'a, T> {
    Ref(&'a mut T),
    Owned(T),
}

impl<'a, T> Deref for OwnedOrRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T> DerefMut for OwnedOrRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'a, X> Read for OwnedOrRef<'a, X>
where X: Read {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.as_mut().read(buf)
    }
}
impl<'a, X> Seek for OwnedOrRef<'a, X>
where X: Seek {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.as_mut().seek(pos)
    }
}

impl<T> From<T> for OwnedOrRef<'static, T> {
    fn from(value: T) -> Self {
        OwnedOrRef::Owned(value)
    }
}
impl<'a, T> From<&'a mut T> for OwnedOrRef<'a, T> {
    fn from(value: &'a mut T) -> Self {
        OwnedOrRef::Ref(value)
    }
}

impl<'a, T> OwnedOrRef<'a, T> {
    pub fn as_mut(&mut self) -> &mut T {
        match self {
            OwnedOrRef::Ref(reference) => *reference,
            OwnedOrRef::Owned(owned) => owned,
        }
    }

    pub fn as_ref(&self) -> &T {
        match self {
            OwnedOrRef::Ref(reference) => *reference,
            OwnedOrRef::Owned(owned) => owned,
        }
    }

    pub fn unwrap_owned(self) -> T {
        match self {
            OwnedOrRef::Ref(_) => panic!("Tried to unwrap a reference as owned value."),
            OwnedOrRef::Owned(owned) => owned,
        }
    }
}
