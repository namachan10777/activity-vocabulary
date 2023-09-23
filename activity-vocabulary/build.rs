use std::{env, fs, path::Path};

fn main() {
    let src = activity_vocabulary_derive::define_types("vocab.yml").unwrap();
    let out_path = env::var("OUT_DIR").unwrap();
    let out_path: &Path = out_path.as_ref();
    fs::write(out_path.join("vocab.rs"), src.as_bytes()).unwrap();
}
