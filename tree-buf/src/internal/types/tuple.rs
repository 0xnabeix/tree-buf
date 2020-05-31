#![allow(non_snake_case)]

use crate::prelude::*;

// https://www.reddit.com/r/rust/comments/339yj3/tuple_indexing_in_a_macro/
macro_rules! expr {
    ($x:expr) => {
        $x
    };
} // HACK
macro_rules! tuple_index {
    ($tuple:expr, $idx:tt) => {
        expr!($tuple.$idx)
    };
}

macro_rules! parallel_new_rhs {
    ($opts:ident, ) => {
      ()
    };
    ($opts:ident, $ts:ident) => {
        $ts::new($ts, $opts)
    };
    ($opts:ident, $ts:ident, $($remainder:ident),+) => {
        parallel(move || $ts::new($ts, $opts), move || parallel_new_rhs!($opts, $($remainder),*), $opts)
    }
}

macro_rules! parallel_read_rhs {
    ($opts: ident) => {
      ()
    };
    ($opts: ident, $ts:ident) => {
        $ts::read($ts, $opts)
    };
    ($opts: ident, $ts:ident, $($remainder:ident),+) => {
        parallel(move || $ts::read($ts, $opts), move || parallel_read_rhs!($opts, $($remainder),*), $opts)
    }
}

macro_rules! parallel_lhs {
    () => {
      ()
    };
    ($ts:ident) => {
        $ts
    };
    ($ts:ident, $($remainder:ident),+) => {
        ($ts, parallel_lhs!($($remainder),*))
    }
}

macro_rules! parallel_new {
    ($opts:ident, $($ts:ident),*) => {
        let parallel_lhs!($($ts),*) = parallel_new_rhs!($opts, $($ts),*);
    };
}

macro_rules! parallel_read {
    ($opts:ident, $($ts:ident),*) => {
        let parallel_lhs!($($ts),*) = parallel_read_rhs!($opts, $($ts),*);
    };
}

macro_rules! impl_tuple {
    ($count:expr, $trid:expr, $taid:expr, $($ts:ident, $ti:tt,)+) => {
        #[cfg(feature = "write")]
        impl <$($ts: Writable),+> Writable for ($($ts),+) {
            type WriterArray=($($ts::WriterArray),+);
            fn write_root<O: EncodeOptions>(&self, stream: &mut WriterStream<'_, O>) -> RootTypeId {
                profile!("Writable::write_root");
                $(
                    stream.write_with_id(|stream| tuple_index!(self, $ti).write_root(stream));
                )+
                $trid
            }
        }

        #[cfg(feature = "write")]
        impl<$($ts: Writable),+> WriterArray<($($ts),+)> for ($($ts::WriterArray),+) {
            fn buffer<'a, 'b: 'a>(&'a mut self, value: &'b ($($ts),+)) {
                $(
                    tuple_index!(self, $ti).buffer(&tuple_index!(value, $ti));
                )+
            }
            fn flush<O: EncodeOptions>(self, stream: &mut WriterStream<'_, O>) -> ArrayTypeId {
                profile!("WriterArray::flush");
                let ($($ts,)+) = self;
                $(
                    stream.write_with_id(|stream|
                        $ts.flush(stream)
                    );
                )+
                $taid
            }
        }

        #[cfg(feature = "read")]
        impl <$($ts: Readable + Send),+> Readable for ($($ts),+)
        // Overly verbose because of `?` requiring `From` See also ec4fa3ba-def5-44eb-9065-e80b59530af6
        where $(ReadError : From<<$ts::ReaderArray as ReaderArray>::Error>),+ {
            type ReaderArray=($($ts::ReaderArray),+);
            fn read(sticks: DynRootBranch<'_>, options: &impl DecodeOptions) -> ReadResult<Self> {
                profile!("Readable::read");
                match sticks {
                    DynRootBranch::Tuple { mut fields } => {
                        // See also abb368f2-6c99-4c44-8f9f-4b00868adaaf
                        if fields.len() != $count {
                            return Err(ReadError::SchemaMismatch)
                        }
                        let mut fields = fields.drain(..);

                        // Move the fields out of the vec
                        $(
                            // This unwrap is ok because we verified the len already. See also abb368f2-6c99-4c44-8f9f-4b00868adaaf
                            let $ts = fields.next().unwrap();
                        )+

                        parallel_read!(options, $($ts),*);

                        Ok(($($ts?),*))
                    },
                    _ => Err(ReadError::SchemaMismatch),
                }
            }
        }

        #[cfg(feature = "read")]
        impl <$($ts: ReaderArray),+> ReaderArray for ($($ts),+)
        // Overly verbose because of `?` requiring `From` See also ec4fa3ba-def5-44eb-9065-e80b59530af6
        where $(ReadError : From<$ts::Error>),+ {
            type Read=($($ts::Read),+);
            // TODO: It would be nice to know somehow whether or not
            // all the fields are infallible types. Perhaps specialization
            // can achieve this.
            type Error=ReadError;
            fn new(sticks: DynArrayBranch<'_>, options: &impl DecodeOptions) -> ReadResult<Self> {
                profile!("ReaderArray::new");

                match sticks {
                    DynArrayBranch::Tuple { mut fields } => {
                        // See also abb368f2-6c99-4c44-8f9f-4b00868adaaf
                        if fields.len() != $count {
                            return Err(ReadError::SchemaMismatch)
                        }
                        let mut fields = fields.drain(..);

                        // Move the fields out of the vec
                        $(
                            // This unwrap is ok because we verified the len already. See also abb368f2-6c99-4c44-8f9f-4b00868adaaf
                            let $ts = fields.next().unwrap();
                        )+

                        parallel_new!(options, $($ts),*);

                        Ok(($($ts?),*))
                    },
                    _ => Err(ReadError::SchemaMismatch)
                }
            }
            fn read_next(&mut self) -> Result<Self::Read, Self::Error> {
                Ok(($(
                    tuple_index!(self, $ti).read_next()?,
                )+))
            }
        }
    };
}

// TODO: Consider 0 and 1 sized tuples.
// These should probably be no serialization at all,
// and pass-through serialization respectively and just
// not use the tuple construct. The tuple construct isn't invalid
// though, which opens considerations for matching either for a schema
// which may not be trivial - like a recursive descent parser.
impl_tuple!(2, RootTypeId::Tuple2, ArrayTypeId::Tuple2, T0, 0, T1, 1,);
impl_tuple!(3, RootTypeId::Tuple3, ArrayTypeId::Tuple3, T0, 0, T1, 1, T2, 2,);
impl_tuple!(4, RootTypeId::Tuple4, ArrayTypeId::Tuple4, T0, 0, T1, 1, T2, 2, T3, 3,);
impl_tuple!(5, RootTypeId::Tuple5, ArrayTypeId::Tuple5, T0, 0, T1, 1, T2, 2, T3, 3, T4, 4,);
impl_tuple!(6, RootTypeId::Tuple6, ArrayTypeId::Tuple6, T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5,);

// TODO: Support tuple structs in the macro

// TODO: Move these into macro

impl<T, T0: Compressor<T>> CompressorSet<T> for (T0,) {
    fn len(&self) -> usize {
        1
    }
    fn fast_size_for(&self, compressor: usize, data: &[T]) -> Option<usize> {
        match compressor {
            0 => self.0.fast_size_for(data),
            _ => unreachable!("No compressor at that index"),
        }
    }
    fn compress(&self, compressor: usize, data: &[T], bytes: &mut Vec<u8>, lens: &mut Vec<usize>) -> Result<ArrayTypeId, ()> {
        match compressor {
            0 => self.0.compress(data, bytes, lens),
            _ => unreachable!("No compressor at that index"),
        }
    }
}

impl<T, T0: Compressor<T>, T1: Compressor<T>> CompressorSet<T> for (T0, T1) {
    fn len(&self) -> usize {
        2
    }
    fn fast_size_for(&self, compressor: usize, data: &[T]) -> Option<usize> {
        match compressor {
            0 => self.0.fast_size_for(data),
            1 => self.1.fast_size_for(data),
            _ => unreachable!("No compressor at that index"),
        }
    }
    fn compress(&self, compressor: usize, data: &[T], bytes: &mut Vec<u8>, lens: &mut Vec<usize>) -> Result<ArrayTypeId, ()> {
        match compressor {
            0 => self.0.compress(data, bytes, lens),
            1 => self.1.compress(data, bytes, lens),
            _ => unreachable!("No compressor at that index"),
        }
    }
}


impl<T, T0: Compressor<T>, T1: Compressor<T>, T2: Compressor<T>> CompressorSet<T> for (T0, T1, T2) {
    fn len(&self) -> usize {
        3
    }
    fn fast_size_for(&self, compressor: usize, data: &[T]) -> Option<usize> {
        match compressor {
            0 => self.0.fast_size_for(data),
            1 => self.1.fast_size_for(data),
            2 => self.2.fast_size_for(data),
            _ => unreachable!("No compressor at that index"),
        }
    }
    fn compress(&self, compressor: usize, data: &[T], bytes: &mut Vec<u8>, lens: &mut Vec<usize>) -> Result<ArrayTypeId, ()> {
        match compressor {
            0 => self.0.compress(data, bytes, lens),
            1 => self.1.compress(data, bytes, lens),
            2 => self.2.compress(data, bytes, lens),
            _ => unreachable!("No compressor at that index"),
        }
    }
}