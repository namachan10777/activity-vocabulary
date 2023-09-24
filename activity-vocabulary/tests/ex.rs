use activity_vocabulary::*;
use activity_vocabulary_core::WithContext;
use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub fn check<T, P: AsRef<Path>>(path: P)
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let json_src = fs::read_to_string(path).unwrap();
    let deserialized: T = serde_json::from_str(&json_src).unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&json_src).unwrap();
    let re_serialized = serde_json::to_value(deserialized).unwrap();
    if re_serialized != json {
        let json = serde_json::to_string_pretty(&json).unwrap();
        let re_serialized = serde_json::to_string_pretty(&re_serialized).unwrap();
        for diff in diff::lines(&json, &re_serialized) {
            match diff {
                diff::Result::Left(l) => println!("-{}", l),
                diff::Result::Both(l, _) => println!(" {}", l),
                diff::Result::Right(r) => println!("+{}", r),
            }
        }
        panic!("differ");
    }
}

macro_rules! gen_tests {
    ($base:literal, $( $test:ident => $ty:ty ),* $(,)? ) => {
        $(
            #[test]
            fn $test() {
                check::<$ty, _>(concat!("tests/", $base, "/", stringify!($test), ".json"));
            }
        )*
    };
}

mod vocab {
    use super::*;

    gen_tests!(
        "vocab",
        ex1 => WithContext<Object>,
        ex2 => WithContext<Link>,
        ex3 => WithContext<Activity>,
        ex4 => WithContext<Travel>,
        ex5 => WithContext<Collection>,
        ex6 => WithContext<OrderedCollection>,
        ex7 => WithContext<CollectionPage>,
        ex8 => WithContext<OrderedCollectionPage>,
    );
}
