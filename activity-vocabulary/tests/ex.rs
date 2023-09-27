use activity_vocabulary::*;
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
    use activity_vocabulary_core::*;

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
        ex9 => WithContext<Accept>,
        ex10 => WithContext<Accept>,
        ex11 => WithContext<TentativeAccept>,
        ex12 => WithContext<Add>,
        ex13 => WithContext<Add>,
        ex14 => WithContext<Arrive>,
        ex15 => WithContext<Create>,
        ex16 => WithContext<Delete>,
        ex17 => WithContext<Follow>,
        ex18 => WithContext<Ignore>,
        ex19 => WithContext<Join>,
        ex20 => WithContext<Leave>,
        ex21 => WithContext<Leave>,
        ex22 => WithContext<Like>,
        ex23 => WithContext<Offer>,
        ex24 => WithContext<Invite>,
        ex25 => WithContext<Reject>,
        ex26 => WithContext<TentativeReject>,
        ex27 => WithContext<Remove>,
        ex28 => WithContext<Remove>,
        ex29 => WithContext<Undo>,
        ex30 => WithContext<Update>,
        ex31 => WithContext<View>,
        ex32 => WithContext<Listen>,
        ex33 => WithContext<Read>,
        ex34 => WithContext<Move>,
        ex35 => WithContext<Travel>,
        ex36 => WithContext<Announce>,
        ex37 => WithContext<Block>,
        ex38 => WithContext<Flag>,
        ex39 => WithContext<Dislike>,
        ex40 => WithContext<Question>,
        ex41 => WithContext<Question>,
    );
}
