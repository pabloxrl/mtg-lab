use sha2::{Digest, Sha256};
use std::{fs, path::Path};
fn main() {
    let root = Path::new("../mtg-core/src");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut paths: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    let mut hash = Sha256::new();
    for p in paths {
        if p.extension().is_some_and(|e| e == "rs") {
            hash.update(p.file_name().unwrap().as_encoded_bytes());
            hash.update([0]);
            hash.update(fs::read(p).unwrap());
        }
    }
    println!("cargo:rustc-env=MTG_ENGINE_SHA256={:x}", hash.finalize());
}
