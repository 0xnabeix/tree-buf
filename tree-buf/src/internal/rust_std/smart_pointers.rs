use crate::prelude::*;
use std::ops::Deref;

// TODO: impl Writable for () {
#[cfg(feature = "write")]
#[derive(Default)]
pub struct BoxWriterArray<T> {
    inner: T,
}

#[cfg(feature = "write")]
impl<'a, T: Writable<'a>> Writable<'a> for Box<T> {
    type WriterArray = BoxWriterArray<T::WriterArray>;
    fn write_root<'b: 'a>(&'b self, stream: &mut impl WriterStream) -> RootTypeId {
        self.deref().write_root(stream)
    }
}

#[cfg(feature = "read")]
pub struct BoxReaderArray<T> {
    inner: T,
}

#[cfg(feature = "read")]
impl<T: Readable> Readable for Box<T> {
    type ReaderArray = BoxReaderArray<T::ReaderArray>;
    fn read(sticks: DynRootBranch<'_>) -> ReadResult<Self> {
        Ok(Box::new(T::read(sticks)?))
    }
}

#[cfg(feature = "write")]
impl<'a, T: WriterArray<'a>> WriterArray<'a> for BoxWriterArray<T> {
    type Write = Box<T::Write>;
    fn buffer<'b: 'a>(&mut self, value: &'b Self::Write) {
        self.inner.buffer(&value)
    }
    fn flush(self, stream: &mut impl WriterStream) -> ArrayTypeId {
        self.inner.flush(stream)
    }
}

#[cfg(feature = "read")]
impl<T: ReaderArray> ReaderArray for BoxReaderArray<T> {
    type Read = Box<T::Read>;
    fn new(sticks: DynArrayBranch<'_>) -> ReadResult<Self> {
        Ok(BoxReaderArray { inner: T::new(sticks)? })
    }
    fn read_next(&mut self) -> ReadResult<Self::Read> {
        Ok(Box::new(self.inner.read_next()?))
    }
}
