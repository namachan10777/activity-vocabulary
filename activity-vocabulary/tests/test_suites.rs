use activity_vocabulary::*;
use activity_vocabulary_core::WithContext;
use anyhow::bail;
use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub fn check<T, P: AsRef<Path>>(input: P, output: P) -> anyhow::Result<()>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let input: serde_json::Value = serde_json::from_str(&fs::read_to_string(input)?)?;
    let output: serde_json::Value = serde_json::from_str(&fs::read_to_string(output)?)?;
    let deserialized: T = serde_json::from_value(input.clone())?;
    let re_serialized = serde_json::to_value(deserialized)?;
    if re_serialized != output {
        let json = serde_json::to_string_pretty(&output)?;
        let re_serialized = serde_json::to_string_pretty(&re_serialized)?;
        for diff in diff::lines(&json, &re_serialized) {
            match diff {
                diff::Result::Left(l) => println!("-{}", l),
                diff::Result::Both(l, _) => println!(" {}", l),
                diff::Result::Right(r) => println!("+{}", r),
            }
        }
        bail!("differ");
    }
    Ok(())
}

#[test]
fn core_ex1() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/core-ex1-jsonld.json",
        "activitystreams/test/core-ex1-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex2() {
    check::<WithContext<Add>, _>(
        "activitystreams/test/core-ex2-jsonld.json",
        "tests/core-ex2-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex3() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/core-ex3-jsonld.json",
        "tests/core-ex3-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex4() {
    check::<WithContext<Person>, _>(
        "activitystreams/test/core-ex4-jsonld.json",
        "tests/core-ex4-jsonld.json",
    )
    .unwrap();
}

/// Custom field is unsupported
#[test]
fn core_ex6() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/core-ex6-jsonld.json",
        "tests/core-ex6-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex7() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex7-jsonld.json",
        "activitystreams/test/core-ex7-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex8() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex8-jsonld.json",
        "activitystreams/test/core-ex8-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex11b() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex11b-jsonld.json",
        "activitystreams/test/core-ex11b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex11c() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex11c-jsonld.json",
        "activitystreams/test/core-ex11c-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex11e() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex11e-jsonld.json",
        "activitystreams/test/core-ex11e-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex11f() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex11f-jsonld.json",
        "activitystreams/test/core-ex11f-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex12() {
    check::<WithContext<Application>, _>(
        "activitystreams/test/core-ex12-jsonld.json",
        "activitystreams/test/core-ex12-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex13() {
    check::<WithContext<Application>, _>(
        "activitystreams/test/core-ex13-jsonld.json",
        "activitystreams/test/core-ex13-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex14() {
    check::<WithContext<Application>, _>(
        "activitystreams/test/core-ex14-jsonld.json",
        "activitystreams/test/core-ex14-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex15() {
    check::<WithContext<Application>, _>(
        "activitystreams/test/core-ex15-jsonld.json",
        "activitystreams/test/core-ex15-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex17() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/core-ex17-jsonld.json",
        "tests/core-ex17-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex19() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/core-ex19-jsonld.json",
        "tests/core-ex19-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex20() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/core-ex20-jsonld.json",
        "tests/core-ex20-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex21() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/core-ex21-jsonld.json",
        "activitystreams/test/core-ex21-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex21b() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/core-ex21b-jsonld.json",
        "tests/core-ex21b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex22() {
    check::<WithContext<OrderedCollection>, _>(
        "activitystreams/test/core-ex22-jsonld.json",
        "activitystreams/test/core-ex22-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex23() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/core-ex23-jsonld.json",
        "activitystreams/test/core-ex23-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex24() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/core-ex24-jsonld.json",
        "tests/core-ex24-jsonld.json",
    )
    .unwrap();
}

#[test]
fn core_ex27() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/core-ex27-jsonld.json",
        "tests/core-ex27-jsonld.json",
    )
    .unwrap();
}
