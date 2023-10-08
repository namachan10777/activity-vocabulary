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
        "activitystreams/test/core-ex2-jsonld.json",
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
        "activitystreams/test/core-ex4-jsonld.json",
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
        "activitystreams/test/core-ex20-jsonld.json",
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

#[test]
fn simple0001() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0001.json",
        "activitystreams/test/simple0001.json",
    )
    .unwrap();
}

#[test]
fn simple0002() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0002.json",
        "activitystreams/test/simple0002.json",
    )
    .unwrap();
}

#[test]
fn simple0003() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0003.json",
        "activitystreams/test/simple0003.json",
    )
    .unwrap();
}

#[test]
fn simple0004() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0004.json",
        "activitystreams/test/simple0004.json",
    )
    .unwrap();
}

#[test]
fn simple0005() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0005.json",
        "tests/simple0005.json",
    )
    .unwrap();
}

#[test]
fn simple0006() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0006.json",
        "tests/simple0006.json",
    )
    .unwrap();
}

#[test]
fn simple0007() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0007.json",
        "tests/simple0007.json",
    )
    .unwrap();
}

#[test]
fn simple0008() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0008.json",
        "activitystreams/test/simple0008.json",
    )
    .unwrap();
}

#[test]
fn simple0009() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0009.json",
        "activitystreams/test/simple0009.json",
    )
    .unwrap();
}

#[test]
fn simple0010() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0010.json",
        "activitystreams/test/simple0010.json",
    )
    .unwrap();
}
#[test]
fn simple0011() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0011.json",
        "tests/simple0011.json",
    )
    .unwrap();
}
#[test]
fn simple0012() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0012.json",
        "tests/simple0012.json",
    )
    .unwrap();
}
#[test]
fn simple0013() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0013.json",
        "activitystreams/test/simple0013.json",
    )
    .unwrap();
}

#[test]
fn simple0014() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0014.json",
        "activitystreams/test/simple0014.json",
    )
    .unwrap();
}

#[test]
fn simple0015() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/simple0015.json",
        "activitystreams/test/simple0015.json",
    )
    .unwrap();
}

#[test]
fn simple0016() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0016.json",
        "activitystreams/test/simple0016.json",
    )
    .unwrap();
}

#[test]
fn simple0017() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0017.json",
        "activitystreams/test/simple0017.json",
    )
    .unwrap();
}

#[test]
fn simple0018() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0018.json",
        "activitystreams/test/simple0018.json",
    )
    .unwrap();
}

#[test]
fn simple0019() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0019.json",
        "activitystreams/test/simple0019.json",
    )
    .unwrap();
}

#[test]
fn simple0020() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0020.json",
        "activitystreams/test/simple0020.json",
    )
    .unwrap();
}

#[test]
fn simple0021() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0021.json",
        "tests/simple0021.json",
    )
    .unwrap();
}

#[test]
fn simple0022() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/simple0022.json",
        "activitystreams/test/simple0022.json",
    )
    .unwrap();
}

#[test]
fn simple0023() {
    check::<WithContext<Undo>, _>(
        "activitystreams/test/simple0023.json",
        "tests/simple0023.json",
    )
    .unwrap();
}

#[test]
fn simple0024() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0024.json",
        "tests/simple0024.json",
    )
    .unwrap();
}

#[test]
fn simple0025() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/simple0025.json",
        "tests/simple0025.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex1() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/vocabulary-ex1-jsonld.json",
        "activitystreams/test/vocabulary-ex1-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex2() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex2-jsonld.json",
        "activitystreams/test/vocabulary-ex2-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex3() {
    check::<WithContext<Activity>, _>(
        "activitystreams/test/vocabulary-ex3-jsonld.json",
        "activitystreams/test/vocabulary-ex3-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex5() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex5-jsonld.json",
        "activitystreams/test/vocabulary-ex5-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex6() {
    check::<WithContext<OrderedCollection>, _>(
        "activitystreams/test/vocabulary-ex6-jsonld.json",
        "activitystreams/test/vocabulary-ex6-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex6b() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex6b-jsonld.json",
        "activitystreams/test/vocabulary-ex6b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex6c() {
    check::<WithContext<OrderedCollectionPage>, _>(
        "activitystreams/test/vocabulary-ex6c-jsonld.json",
        "activitystreams/test/vocabulary-ex6c-jsonld.json",
    )
    .unwrap();
}
#[test]
fn vocabulary_ex7() {
    check::<WithContext<Accept>, _>(
        "activitystreams/test/vocabulary-ex7-jsonld.json",
        "tests/vocabulary-ex7-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex7a() {
    check::<WithContext<Accept>, _>(
        "activitystreams/test/vocabulary-ex7a-jsonld.json",
        "tests/vocabulary-ex7a-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex7b() {
    check::<WithContext<Accept>, _>(
        "activitystreams/test/vocabulary-ex7b-jsonld.json",
        "activitystreams/test/vocabulary-ex7b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex8() {
    check::<WithContext<TentativeAccept>, _>(
        "activitystreams/test/vocabulary-ex8-jsonld.json",
        "tests/vocabulary-ex8-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex9() {
    check::<WithContext<Add>, _>(
        "activitystreams/test/vocabulary-ex9-jsonld.json",
        "activitystreams/test/vocabulary-ex9-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex10() {
    check::<WithContext<Add>, _>(
        "activitystreams/test/vocabulary-ex10-jsonld.json",
        "activitystreams/test/vocabulary-ex10-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex11() {
    check::<WithContext<Arrive>, _>(
        "activitystreams/test/vocabulary-ex11-jsonld.json",
        "activitystreams/test/vocabulary-ex11-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex12() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/vocabulary-ex12-jsonld.json",
        "activitystreams/test/vocabulary-ex12-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex13() {
    check::<WithContext<Delete>, _>(
        "activitystreams/test/vocabulary-ex13-jsonld.json",
        "activitystreams/test/vocabulary-ex13-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex15() {
    check::<WithContext<Follow>, _>(
        "activitystreams/test/vocabulary-ex15-jsonld.json",
        "activitystreams/test/vocabulary-ex15-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex16() {
    check::<WithContext<Ignore>, _>(
        "activitystreams/test/vocabulary-ex16-jsonld.json",
        "activitystreams/test/vocabulary-ex16-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex17() {
    check::<WithContext<Join>, _>(
        "activitystreams/test/vocabulary-ex17-jsonld.json",
        "activitystreams/test/vocabulary-ex17-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex18() {
    check::<WithContext<Leave>, _>(
        "activitystreams/test/vocabulary-ex18-jsonld.json",
        "activitystreams/test/vocabulary-ex18-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex19() {
    check::<WithContext<Leave>, _>(
        "activitystreams/test/vocabulary-ex19-jsonld.json",
        "activitystreams/test/vocabulary-ex19-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex20() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/vocabulary-ex20-jsonld.json",
        "activitystreams/test/vocabulary-ex20-jsonld.json",
    )
    .unwrap();
}
#[test]
fn vocabulary_ex21() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex21-jsonld.json",
        "activitystreams/test/vocabulary-ex21-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex22() {
    check::<WithContext<Relationship>, _>(
        "activitystreams/test/vocabulary-ex22-jsonld.json",
        "activitystreams/test/vocabulary-ex22-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex22a() {
    check::<WithContext<Relationship>, _>(
        "activitystreams/test/vocabulary-ex22a-jsonld.json",
        "activitystreams/test/vocabulary-ex22a-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex22c() {
    check::<WithContext<Relationship>, _>(
        "activitystreams/test/vocabulary-ex22c-jsonld.json",
        "activitystreams/test/vocabulary-ex22c-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex24() {
    check::<WithContext<Invite>, _>(
        "activitystreams/test/vocabulary-ex24-jsonld.json",
        "activitystreams/test/vocabulary-ex24-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex26() {
    check::<WithContext<Reject>, _>(
        "activitystreams/test/vocabulary-ex26-jsonld.json",
        "tests/vocabulary-ex26-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex27() {
    check::<WithContext<TentativeReject>, _>(
        "activitystreams/test/vocabulary-ex27-jsonld.json",
        "tests/vocabulary-ex27-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex28() {
    check::<WithContext<TentativeReject>, _>(
        "activitystreams/test/vocabulary-ex28-jsonld.json",
        "activitystreams/test/vocabulary-ex28-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex29() {
    check::<WithContext<Remove>, _>(
        "activitystreams/test/vocabulary-ex29-jsonld.json",
        "activitystreams/test/vocabulary-ex29-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex32() {
    check::<WithContext<Undo>, _>(
        "activitystreams/test/vocabulary-ex32-jsonld.json",
        "tests/vocabulary-ex32-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex33() {
    check::<WithContext<Update>, _>(
        "activitystreams/test/vocabulary-ex33-jsonld.json",
        "activitystreams/test/vocabulary-ex33-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex34() {
    check::<WithContext<Application>, _>(
        "activitystreams/test/vocabulary-ex34-jsonld.json",
        "activitystreams/test/vocabulary-ex34-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex37() {
    check::<WithContext<Group>, _>(
        "activitystreams/test/vocabulary-ex37-jsonld.json",
        "activitystreams/test/vocabulary-ex37-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex39() {
    check::<WithContext<Person>, _>(
        "activitystreams/test/vocabulary-ex39-jsonld.json",
        "activitystreams/test/vocabulary-ex39-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex42() {
    check::<WithContext<Person>, _>(
        "activitystreams/test/vocabulary-ex42-jsonld.json",
        "activitystreams/test/vocabulary-ex42-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex43() {
    check::<WithContext<Article>, _>(
        "activitystreams/test/vocabulary-ex43-jsonld.json",
        "tests/vocabulary-ex43-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex48() {
    check::<WithContext<Document>, _>(
        "activitystreams/test/vocabulary-ex48-jsonld.json",
        "activitystreams/test/vocabulary-ex48-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex49() {
    check::<WithContext<Audio>, _>(
        "activitystreams/test/vocabulary-ex49-jsonld.json",
        "activitystreams/test/vocabulary-ex49-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex50() {
    check::<WithContext<Image>, _>(
        "activitystreams/test/vocabulary-ex50-jsonld.json",
        "activitystreams/test/vocabulary-ex50-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex51() {
    check::<WithContext<Video>, _>(
        "activitystreams/test/vocabulary-ex51-jsonld.json",
        "activitystreams/test/vocabulary-ex51-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex52() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex52-jsonld.json",
        "activitystreams/test/vocabulary-ex52-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex53() {
    check::<WithContext<Page>, _>(
        "activitystreams/test/vocabulary-ex53-jsonld.json",
        "activitystreams/test/vocabulary-ex53-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex55() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex55-jsonld.json",
        "activitystreams/test/vocabulary-ex55-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex55a() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex55a-jsonld.json",
        "activitystreams/test/vocabulary-ex55a-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex55b() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex55b-jsonld.json",
        "activitystreams/test/vocabulary-ex55b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex56() {
    check::<WithContext<Event>, _>(
        "activitystreams/test/vocabulary-ex56-jsonld.json",
        "activitystreams/test/vocabulary-ex56-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex57() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex57-jsonld.json",
        "activitystreams/test/vocabulary-ex57-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex58() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex58-jsonld.json",
        "tests/vocabulary-ex58-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex59() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex59-jsonld.json",
        "tests/vocabulary-ex59-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex60() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex60-jsonld.json",
        "tests/vocabulary-ex60-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex61() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex61-jsonld.json",
        "tests/vocabulary-ex61-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex64() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex64-jsonld.json",
        "tests/vocabulary-ex64-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex66() {
    check::<WithContext<Image>, _>(
        "activitystreams/test/vocabulary-ex66-jsonld.json",
        "tests/vocabulary-ex66-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex68() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex68-jsonld.json",
        "tests/vocabulary-ex68-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex69() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex69-jsonld.json",
        "tests/vocabulary-ex69-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex70() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex70-jsonld.json",
        "tests/vocabulary-ex70-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex71() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex71-jsonld.json",
        "activitystreams/test/vocabulary-ex71-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex72() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex72-jsonld.json",
        "activitystreams/test/vocabulary-ex72-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex73() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex73-jsonld.json",
        "activitystreams/test/vocabulary-ex73-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex74() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex74-jsonld.json",
        "activitystreams/test/vocabulary-ex74-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex75() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex75-jsonld.json",
        "activitystreams/test/vocabulary-ex75-jsonld.json",
    )
    .unwrap();
}

/* invalid example
#[test]
fn vocabulary_ex77() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex77-jsonld.json",
        "activitystreams/test/vocabulary-ex77-jsonld.json",
    )
    .unwrap();
}*/

/*
#[test]
fn vocabulary_ex78() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex78-jsonld.json",
        "activitystreams/test/vocabulary-ex78-jsonld.json",
    )
    .unwrap();
}
*/

#[test]
fn vocabulary_ex80() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex80-jsonld.json",
        "activitystreams/test/vocabulary-ex80-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex81() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex81-jsonld.json",
        "activitystreams/test/vocabulary-ex81-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex83() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex83-jsonld.json",
        "activitystreams/test/vocabulary-ex83-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex84() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex84-jsonld.json",
        "activitystreams/test/vocabulary-ex84-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex87() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex87-jsonld.json",
        "activitystreams/test/vocabulary-ex87-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex88() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex88-jsonld.json",
        "activitystreams/test/vocabulary-ex88-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex89() {
    check::<WithContext<Person>, _>(
        "activitystreams/test/vocabulary-ex89-jsonld.json",
        "tests/vocabulary-ex89-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex91() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex91-jsonld.json",
        "activitystreams/test/vocabulary-ex91-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex92() {
    check::<WithContext<OrderedCollection>, _>(
        "activitystreams/test/vocabulary-ex92-jsonld.json",
        "activitystreams/test/vocabulary-ex92-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex93() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex93-jsonld.json",
        "activitystreams/test/vocabulary-ex93-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex94() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex94-jsonld.json",
        "activitystreams/test/vocabulary-ex94-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex94b() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex94b-jsonld.json",
        "activitystreams/test/vocabulary-ex94b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex96() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex96-jsonld.json",
        "activitystreams/test/vocabulary-ex96-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex97() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex97-jsonld.json",
        "activitystreams/test/vocabulary-ex97-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex98() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/vocabulary-ex98-jsonld.json",
        "tests/vocabulary-ex98-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex99() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/vocabulary-ex99-jsonld.json",
        "tests/vocabulary-ex99-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex100() {
    check::<WithContext<Like>, _>(
        "activitystreams/test/vocabulary-ex100-jsonld.json",
        "tests/vocabulary-ex100-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex101() {
    check::<WithContext<Listen>, _>(
        "activitystreams/test/vocabulary-ex101-jsonld.json",
        "activitystreams/test/vocabulary-ex101-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex104() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex104-jsonld.json",
        "activitystreams/test/vocabulary-ex104-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex105() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex105-jsonld.json",
        "activitystreams/test/vocabulary-ex105-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex106() {
    check::<WithContext<Video>, _>(
        "activitystreams/test/vocabulary-ex106-jsonld.json",
        "tests/vocabulary-ex106-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex108() {
    check::<WithContext<Activity>, _>(
        "activitystreams/test/vocabulary-ex108-jsonld.json",
        "tests/vocabulary-ex108-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex112() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex112-jsonld.json",
        "tests/vocabulary-ex112-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex113() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex113-jsonld.json",
        "activitystreams/test/vocabulary-ex113-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex118() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex118-jsonld.json",
        "tests/vocabulary-ex118-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex120() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex120-jsonld.json",
        "tests/vocabulary-ex120-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex121() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex121-jsonld.json",
        "tests/vocabulary-ex121-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex123() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex123-jsonld.json",
        "tests/vocabulary-ex123-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex124() {
    check::<WithContext<Document>, _>(
        "activitystreams/test/vocabulary-ex124-jsonld.json",
        "activitystreams/test/vocabulary-ex124-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex125() {
    check::<WithContext<Document>, _>(
        "activitystreams/test/vocabulary-ex125-jsonld.json",
        "activitystreams/test/vocabulary-ex125-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex126() {
    check::<WithContext<Document>, _>(
        "activitystreams/test/vocabulary-ex126-jsonld.json",
        "activitystreams/test/vocabulary-ex126-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex127() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex127-jsonld.json",
        "activitystreams/test/vocabulary-ex127-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex129() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex129-jsonld.json",
        "activitystreams/test/vocabulary-ex129-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex130() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex130-jsonld.json",
        "activitystreams/test/vocabulary-ex130-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex130b() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex130b-jsonld.json",
        "activitystreams/test/vocabulary-ex130b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex131() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex131-jsonld.json",
        "activitystreams/test/vocabulary-ex131-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex132() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex132-jsonld.json",
        "activitystreams/test/vocabulary-ex132-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex133() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex133-jsonld.json",
        "activitystreams/test/vocabulary-ex133-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex134() {
    check::<WithContext<Video>, _>(
        "activitystreams/test/vocabulary-ex134-jsonld.json",
        "activitystreams/test/vocabulary-ex134-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex136() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex136-jsonld.json",
        "activitystreams/test/vocabulary-ex136-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex137() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex137-jsonld.json",
        "activitystreams/test/vocabulary-ex137-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex138() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex138-jsonld.json",
        "activitystreams/test/vocabulary-ex138-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex139() {
    check::<WithContext<CollectionPage>, _>(
        "activitystreams/test/vocabulary-ex139-jsonld.json",
        "activitystreams/test/vocabulary-ex139-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex140() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex140-jsonld.json",
        "tests/vocabulary-ex140-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex141() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex141-jsonld.json",
        "tests/vocabulary-ex141-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex142() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex142-jsonld.json",
        "activitystreams/test/vocabulary-ex142-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex144() {
    check::<WithContext<Event>, _>(
        "activitystreams/test/vocabulary-ex144-jsonld.json",
        "activitystreams/test/vocabulary-ex144-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex145() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex145-jsonld.json",
        "activitystreams/test/vocabulary-ex145-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex146() {
    check::<WithContext<Event>, _>(
        "activitystreams/test/vocabulary-ex146-jsonld.json",
        "activitystreams/test/vocabulary-ex146-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex147() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex147-jsonld.json",
        "tests/vocabulary-ex147-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex149() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex149-jsonld.json",
        "activitystreams/test/vocabulary-ex149-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex150() {
    check::<WithContext<OrderedCollectionPage>, _>(
        "activitystreams/test/vocabulary-ex150-jsonld.json",
        "activitystreams/test/vocabulary-ex150-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex152() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex152-jsonld.json",
        "activitystreams/test/vocabulary-ex152-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex153() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex153-jsonld.json",
        "activitystreams/test/vocabulary-ex153-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex156() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex156-jsonld.json",
        "activitystreams/test/vocabulary-ex156-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex157() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex157-jsonld.json",
        "tests/vocabulary-ex157-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex158() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex158-jsonld.json",
        "activitystreams/test/vocabulary-ex158-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex159() {
    check::<WithContext<Link>, _>(
        "activitystreams/test/vocabulary-ex159-jsonld.json",
        "activitystreams/test/vocabulary-ex159-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex161() {
    check::<WithContext<View>, _>(
        "activitystreams/test/vocabulary-ex161-jsonld.json",
        "activitystreams/test/vocabulary-ex161-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex163() {
    check::<WithContext<Listen>, _>(
        "activitystreams/test/vocabulary-ex163-jsonld.json",
        "activitystreams/test/vocabulary-ex163-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex164() {
    check::<WithContext<Read>, _>(
        "activitystreams/test/vocabulary-ex164-jsonld.json",
        "activitystreams/test/vocabulary-ex164-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex166() {
    check::<WithContext<Move>, _>(
        "activitystreams/test/vocabulary-ex166-jsonld.json",
        "tests/vocabulary-ex166-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex168() {
    check::<WithContext<Move>, _>(
        "activitystreams/test/vocabulary-ex168-jsonld.json",
        "activitystreams/test/vocabulary-ex168-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex169() {
    check::<WithContext<Travel>, _>(
        "activitystreams/test/vocabulary-ex169-jsonld.json",
        "activitystreams/test/vocabulary-ex169-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex170() {
    check::<WithContext<Announce>, _>(
        "activitystreams/test/vocabulary-ex170-jsonld.json",
        "tests/vocabulary-ex170-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex171() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex171-jsonld.json",
        "tests/vocabulary-ex171-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex173() {
    check::<WithContext<Block>, _>(
        "activitystreams/test/vocabulary-ex173-jsonld.json",
        "tests/vocabulary-ex173-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex174() {
    check::<WithContext<Flag>, _>(
        "activitystreams/test/vocabulary-ex174-jsonld.json",
        "tests/vocabulary-ex174-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex175() {
    check::<WithContext<Dislike>, _>(
        "activitystreams/test/vocabulary-ex175-jsonld.json",
        "tests/vocabulary-ex175-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex180() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex180-jsonld.json",
        "activitystreams/test/vocabulary-ex180-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex181() {
    check::<WithContext<Mention>, _>(
        "activitystreams/test/vocabulary-ex181-jsonld.json",
        "activitystreams/test/vocabulary-ex181-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex181b() {
    check::<WithContext<Create>, _>(
        "activitystreams/test/vocabulary-ex181-jsonldb.json",
        "tests/vocabulary-ex181-jsonldb.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex182() {
    check::<WithContext<Travel>, _>(
        "activitystreams/test/vocabulary-ex182-jsonld.json",
        "activitystreams/test/vocabulary-ex182-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex182b() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex182-jsonldb.json",
        "activitystreams/test/vocabulary-ex182-jsonldb.json",
    )
    .unwrap();
}

/* invalid example
#[test]
fn vocabulary_ex183() {
    check::<WithContext<Place>, _>(
        "activitystreams/test/vocabulary-ex183-jsonld.json",
        "activitystreams/test/vocabulary-ex183-jsonld.json",
    )
    .unwrap();
}*/

#[test]
fn vocabulary_ex184() {
    check::<WithContext<OrderedCollection>, _>(
        "activitystreams/test/vocabulary-ex184-jsonld.json",
        "activitystreams/test/vocabulary-ex184-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex184a() {
    check::<WithContext<Profile>, _>(
        "activitystreams/test/vocabulary-ex184a-jsonld.json",
        "activitystreams/test/vocabulary-ex184a-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex184b() {
    check::<WithContext<OrderedCollection>, _>(
        "activitystreams/test/vocabulary-ex184b-jsonld.json",
        "activitystreams/test/vocabulary-ex184b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex185() {
    check::<WithContext<Profile>, _>(
        "activitystreams/test/vocabulary-ex185-jsonld.json",
        "tests/vocabulary-ex185-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex186() {
    check::<WithContext<Organization>, _>(
        "activitystreams/test/vocabulary-ex186-jsonld.json",
        "activitystreams/test/vocabulary-ex186-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex187() {
    check::<WithContext<Offer>, _>(
        "activitystreams/test/vocabulary-ex187-jsonld.json",
        "activitystreams/test/vocabulary-ex187-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex188() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex188-jsonld.json",
        "activitystreams/test/vocabulary-ex188-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex189() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex189-jsonld.json",
        "activitystreams/test/vocabulary-ex189-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex190() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex190-jsonld.json",
        "activitystreams/test/vocabulary-ex190-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex191() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/vocabulary-ex191-jsonld.json",
        "tests/vocabulary-ex191-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex192() {
    check::<WithContext<Question>, _>(
        "activitystreams/test/vocabulary-ex192-jsonld.json",
        "tests/vocabulary-ex192-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex193() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex193-jsonld.json",
        "tests/vocabulary-ex193-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex193b() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex193b-jsonld.json",
        "tests/vocabulary-ex193b-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex194() {
    check::<WithContext<Collection>, _>(
        "activitystreams/test/vocabulary-ex194-jsonld.json",
        "tests/vocabulary-ex194-jsonld.json",
    )
    .unwrap();
}

/* invalid json
#[test]
fn vocabulary_ex196() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex196-jsonld.json",
        "tests/vocabulary-ex196-jsonld.json",
    )
    .unwrap();
}
*/

#[test]
fn vocabulary_ex197() {
    check::<WithContext<Note>, _>(
        "activitystreams/test/vocabulary-ex197-jsonld.json",
        "tests/vocabulary-ex197-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_ex198() {
    check::<WithContext<Move>, _>(
        "activitystreams/test/vocabulary-ex198-jsonld.json",
        "tests/vocabulary-ex198-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_exid() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/vocabulary-exid-jsonld.json",
        "activitystreams/test/vocabulary-exid-jsonld.json",
    )
    .unwrap();
}

#[test]
fn vocabulary_extype() {
    check::<WithContext<Object>, _>(
        "activitystreams/test/vocabulary-extype-jsonld.json",
        "activitystreams/test/vocabulary-extype-jsonld.json",
    )
    .unwrap();
}
