use std::io::{Read, Write, Seek, SeekFrom, Result};

pub struct CountingIO<IO> {
    inner: IO,
    counter: Option<u64>,
}

impl<IO> CountingIO<IO> {
    pub fn new(inner: IO) -> Self {
        Self {
            inner,
            counter: None,
        }
    }

    pub fn without_offset(mut self) -> Self {
        self.counter = Some(0);
        self
    }

    #[allow(dead_code)]
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.counter = Some(offset);
        self
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, offset: u64) {
        self.counter = Some(offset);
    }

    pub fn position(&self) -> Option<u64> {
        self.counter
    }

    pub fn into_inner(self) -> IO {
        self.inner
    }
}

impl<IO> Read for CountingIO<IO> 
where IO: Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let result = self.inner.read(buf);

        if let Ok(count) = result {
            if let Some(ref mut counter) = self.counter {
                *counter += count as u64;
            }
        }

        result
    }
}

impl<IO> Write for CountingIO<IO> 
where IO: Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let result = self.inner.write(buf);

        if let Ok(count) = result {
            if let Some(ref mut counter) = self.counter {
                *counter += count as u64;
            }
        }

        result
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<IO> Seek for CountingIO<IO>
where IO: Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let result = self.inner.seek(pos);

        if let Ok(count) = result {
            self.counter = Some(count as u64);
        }

        result
    }
}