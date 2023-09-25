use std::{env, fs, path::Path};

fn main() {
    let src = fs::read_to_string("vocab.yml").unwrap();
    let src = serde_yaml::from_str(&src).unwrap();
    let src = activity_vocabulary_derive::gen(&src).unwrap();
    let out_path = env::var("OUT_DIR").unwrap();
    let out_path: &Path = out_path.as_ref();
    println!("cargo:rerun-if-changed=vocab.yml");
    fs::write(out_path.join("vocab.rs"), src.as_bytes()).unwrap();
}
