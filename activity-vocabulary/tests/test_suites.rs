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

#[test]
fn core_ex6() {
    check::<WithContext<Person>, _>(
        "activitystreams/test/core-ex6-jsonld.json",
        "activitystreams/test/core-ex6-jsonld.json",
    )
    .unwrap();
}
