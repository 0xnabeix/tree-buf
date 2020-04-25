use tree_buf::prelude::*;
use std::mem::size_of;



/// Verifies that the infallible tuple read has a zero-cost error
#[test]
pub fn tuples_reduce_error_size() {
    type T = (f64, f64);
    let orig = size_of::<T>();
    let wrapped = size_of::<Result<T, <<T as ::tree_buf::internal::Readable>::ReaderArray as ::tree_buf::internal::ReaderArray>::Error>>();
    assert_eq!(orig, wrapped);
}