use std::fmt::Debug;
use tree_buf::prelude::*;
mod common;
use common::*;
use std::collections::HashMap;
use tree_buf::encode_options;
use tree_buf::options;

// Create this namespace to hide the prelude. This is a check that the hygenics do not require any types from tree_buf to be imported
mod hide_namespace {
    use tree_buf::{Read, Write};
    #[derive(Read, Write, PartialEq, Debug, Clone)]
    pub struct Bits {
        pub f: f64,
        pub obj_array: Vec<Bobs>,
        pub extra: Option<Bobs>,
        pub s: Box<String>,
    }

    #[derive(Read, Write, PartialEq, Debug, Clone)]
    pub struct Bobs {
        pub one: Vec<u64>,
        pub tup: (f64, f64),
    }
}

use hide_namespace::{Bits, Bobs};

// TODO: Compare to Avro - https://github.com/flavray/avro-rs

fn make_item() -> Bits {
    Bits {
        f: 5.0,
        extra: Some(Bobs {
            one: vec![99],
            tup: (9999.99, 200.1),
        }),
        s: Box::new("abc".to_owned()),
        obj_array: vec![
            Bobs {
                one: vec![3, 2, 1, 0],
                tup: (10.0, 200.2),
            },
            Bobs { one: vec![], tup: (2.2, 200.3) },
            Bobs {
                one: vec![20, 20, 20, 20, 20, 20, 20],
                tup: (0.0, 200.4),
            },
        ],
    }
}

#[test]
fn broken_int() {
    round_trip(&75339u64, 4, 10);
}

#[test]
fn bools_root() {
    round_trip(&true, 1, 5);
    round_trip(&false, 1, 5);
}

#[test]
fn opts_root() {
    round_trip(&Some(true), 1, 8);
    round_trip(&Option::<bool>::None, 1, 3);
}

#[test]
fn bool_array() {
    round_trip(&vec![false, true, true, false, true, true, true, false, false, true, false, true], 6, 9);
}

#[test]
fn ints_root() {
    round_trip(&0u32, 1, 4);
    round_trip(&1u32, 1, 6);
    for i in 2..=127u32 {
        round_trip(&i, 2, 6);
    }
    for i in 128..=255u32 {
        round_trip(&i, 2, 6);
    }
    for i in 256..1024u32 {
        round_trip(&i, 3, 8);
    }
}

// Special case for 1 element array encodes root object
#[test]
fn array1() {
    round_trip(&vec![99u64], 3, 8);
    round_trip(&vec![1u64], 2, 8);
}

#[test]
fn int_vec() {
    round_trip(&vec![99u64, 100], 6, 10);
}

#[test]
fn float64_vec() {
    round_trip(&vec![0.99], 10, 16);
    round_trip(&vec![0.01, 0.02, 0.03, 0.04], 36, 65);
}

#[test]
fn float32_vec() {
    round_trip(&vec![0.99f32], 6, 14);
    round_trip(&vec![0.01f32, 0.02, 0.03, 0.04], 20, 38);
}

#[test]
fn lossy_f64_vec() {
    let mut data = Vec::new();
    for i in 0..50 {
        data.push(0.01 * i as f64);
    }
    let tolerance = 10;
    let options = encode_options! { options::LossyFloatTolerance(tolerance) };
    let binary = tree_buf::write_with_options(&data, &options);
    assert_eq!(binary.len(), 104);
    let decoded = read::<Vec<f64>>(&binary).unwrap();
    assert_eq!(data.len(), decoded.len());
    for (e, d) in data.iter().zip(decoded.iter()) {
        assert!((e - d).abs() <= 0.001);
    }

    // Show how much smaller this is than lossless
    let options = encode_options! { options::LosslessFloat };
    let binary = tree_buf::write_with_options(&data, &options);
    assert_eq!(binary.len(), 376);

    // Show that this is much better than fixed, since this would be a minimum for exactly 0 schema overhead.
    assert_eq!(std::mem::size_of::<f64>() * data.len(), 400);
}

#[test]
fn nested_float_vec() {
    round_trip(&vec![vec![10.0, 11.0], vec![], vec![99.0]], 24, 32);
}

#[test]
fn array_tuple() {
    round_trip(&vec![vec![(1u32, 2u32), (3, 4), (5, 6)]], 14, 19);
}

#[test]
fn item() {
    let item = make_item();
    round_trip(&item, 136, 212);
}

#[test]
fn item_vec() {
    let item = make_item();
    let item = vec![item; 5];
    round_trip(&item, 379, 659);
}

#[test]
fn nullable_array() {
    round_trip(&vec![Some(1u32), Some(2)], 9, 13);
}

#[test]
fn visibility_modifiers() {
    #[derive(Default, Read, Write, Debug, PartialEq, Clone)]
    struct Inherited {
        a: u64,
    }

    #[derive(Default, Read, Write, Debug, PartialEq, Clone)]
    pub(crate) struct Crate {
        a: u64,
    }

    #[derive(Default, Read, Write, Debug, PartialEq, Clone)]
    pub struct Public {
        a: u64,
    }

    round_trip_default::<Inherited>(4, 7);
    round_trip_default::<Crate>(4, 7);
    round_trip_default::<Public>(4, 7);
}

#[test]
fn ignores() {
    use tree_buf::Ignore;
    round_trip(&Ignore, 1, 3);

    #[derive(Default, Read, Write, Debug, PartialEq, Clone)]
    struct X {
        i: Ignore,
    }

    let x = X { i: Ignore };
    round_trip(&x, 4, 6);

    #[derive(Read, Write, Debug, PartialEq, Clone)]
    enum E {
        A(Ignore),
        B(Ignore),
    }

    let e = E::A(Ignore);
    round_trip(&e, 4, 9);

    #[derive(Read, Write, Debug, PartialEq, Clone)]
    struct N {
        e: E,
    }

    let o = vec![N { e: E::A(Ignore) }, N { e: E::B(Ignore) }];
    round_trip(&o, 17, 21);
}

// TODO: Using Quickcheck and Arbitrary with quickcheck_derive.
#[test]
fn various_types() {
    round_trip_default::<u64>(1, 4);
    round_trip_default::<u32>(1, 4);
    round_trip_default::<u16>(1, 4);
    round_trip_default::<u8>(1, 4);
    round_trip_default::<(u64, u64)>(3, 7);
    round_trip_default::<(u64, u32)>(3, 7);
    round_trip_default::<f64>(1, 4);
    // See also: 84d15459-35e4-4f04-896f-0f4ea9ce52a9
    round_trip_default::<Vec<u32>>(1, 5);
    round_trip_default::<Option<Vec<u32>>>(1, 3);
    round_trip_default::<Option<u32>>(1, 3);
    round_trip_default::<Vec<Option<u32>>>(1, 5);
    round_trip_default::<String>(1, 4);
}

#[test]
fn conversions() {
    // TODO: f32
    //serialize_eq(1.0f64, 1.0f32, 0);
    //serialize_eq(1.0f32, 1.0f64, 0);
    //serialize_eq(9.0f32, 9.0f64, 0);

    // TODO: A bunch more of these
}

#[test]
fn small_structs() {
    #[derive(Read, Write, Default, Debug, PartialEq, Clone)]
    struct _1 {
        a: u64,
    }

    round_trip_default::<_1>(4, 7);
}

#[test]
fn large_structs() {
    #[derive(Read, Write, Default, Debug, PartialEq, Clone)]
    struct _14 {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
    }

    #[derive(Read, Write, Default, Debug, PartialEq, Clone)]
    struct _15 {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
        o: f64,
    }

    #[derive(Read, Write, Default, Debug, PartialEq, Clone)]
    struct _16 {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
        o: f64,
        p: f64,
    }
    #[derive(Read, Write, Default, Debug, PartialEq, Clone)]
    struct _17 {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
        o: f64,
        p: f64,
        q: f64,
    }

    round_trip_default::<_14>(44, 60);
    round_trip_default::<_15>(47, 64);
    round_trip_default::<_16>(50, 68);
    round_trip_default::<_17>(53, 72);
}

#[test]
fn map_0_root() {
    // See also: 84d15459-35e4-4f04-896f-0f4ea9ce52a9
    let data = HashMap::<u32, u32>::new();
    round_trip(&data, 2, 7);
}

#[test]
fn map_1_root() {
    let mut data = HashMap::new();
    data.insert("test".to_owned(), 5u32);
    round_trip(&data, 10, 23);
}

#[test]
fn map_n_root() {
    let mut data = HashMap::new();
    data.insert("test3".to_owned(), 5u32);
    data.insert("test2".to_owned(), 5);
    data.insert("test1".to_owned(), 0);
    round_trip(&data, None, None);
}

#[test]
fn maps_array() {
    let mut data = Vec::new();
    for i in 0..5u32 {
        let mut h = HashMap::new();
        h.insert(i, Vec::<u32>::new());
        h.insert(10, vec![10, 9, 8, 7]);
        data.push(h);
    }
    // Interestingly, the output size is not deterministic in this case.
    // It depends on whether the last key or value from iterating the HashMap is Default
    round_trip(&data, None, None);
}

#[test]
fn maps_void() {
    let mut data = Vec::new();
    for _ in 0..5 {
        let h = HashMap::<String, String>::new();
        data.push(h);
    }
    round_trip(&data, 9, 11);
}

#[test]
fn fixed_arrays() {
    round_trip(&[0u32, 1, 2, 3], 8, 10);
    round_trip(&[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4, 6);
}

// This failed to compile at one point when moving generics for WriterArray out of associated type.
#[test]
fn enum_with_vec() {
    #[derive(Write, Read, Debug, PartialEq, Clone)]
    enum X {
        X(Vec<u64>),
    }

    round_trip(&X::X(vec![25, 30, 0, 0, 0]), 9, 19);
}

fn owned_vec(strs: Vec<&'static str>) -> Vec<String> {
    strs.iter().map(|s| String::from(*s)).collect()
}

#[test]
fn strings_using_rle() {
    let data = vec!["abcd", "abcd", "def", "abcd", "abcd", "abcd", ""];
    round_trip(&owned_vec(data), 26, 43);

    let data = vec!["abcd", "abcd", "abcd", "abcd", "abcd"];
    round_trip(&owned_vec(data), 17, 19);

    let data = vec!["abcd", "abcd", "abcd", "abcd", "abcd", "def", "def"];
    round_trip(&owned_vec(data), 21, 32);

    let data = vec!["abcd", "abcd", "abcd", "abcd", "abcd", "abcd", "def"];
    round_trip(&owned_vec(data), 21, 32);
}

#[test]
fn nested_strings_using_rle() {
    //let data = vec![owned_vec(vec!["a", "a"])];

    let data = (
        //owned_vec(vec![]),
        owned_vec(vec!["abc", "abc", "abc"]),
        1u32,
    );

    round_trip(&data, None, None);
}
