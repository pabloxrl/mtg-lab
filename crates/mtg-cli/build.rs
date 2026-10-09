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
    benchmark_metadata();
}

fn benchmark_metadata() {
    use std::process::Command;
    let rustc = Command::new(std::env::var_os("RUSTC").unwrap())
        .arg("-vV")
        .output()
        .unwrap();
    assert!(rustc.status.success());
    println!(
        "cargo:rustc-env=MTG_BENCH_RUSTC={}",
        String::from_utf8(rustc.stdout).unwrap().replace('\n', "; ")
    );
    for (key, name) in [
        ("TARGET", "TARGET"),
        ("OPT_LEVEL", "OPT_LEVEL"),
        ("CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS"),
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        println!(
            "cargo:rustc-env=MTG_BENCH_{name}={}",
            std::env::var(key).unwrap_or_default().replace('\x1f', " ")
        );
    }
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".into());
    println!("cargo:rustc-env=MTG_BENCH_COMMIT={commit}");
    fn collect(path: &Path, paths: &mut Vec<std::path::PathBuf>) {
        println!("cargo:rerun-if-changed={}", path.display());
        if path.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                collect(&entry.unwrap().path(), paths);
            }
        } else {
            paths.push(path.to_owned());
        }
    }
    let mut paths = Vec::new();
    for path in [
        "src",
        "build.rs",
        "Cargo.toml",
        "../mtg-core/src",
        "../mtg-policy/src",
        "../mtg-recorder/src",
        "../../Cargo.lock",
        "../../Cargo.toml",
        "../../data",
        "../../fixtures/bench/scalar-workload-v1.json",
    ] {
        collect(Path::new(path), &mut paths);
    }
    paths.sort();
    let mut hash = Sha256::new();
    for path in paths {
        hash.update(path.as_os_str().as_encoded_bytes());
        hash.update([0]);
        hash.update(fs::read(path).unwrap());
    }
    println!("cargo:rustc-env=MTG_BENCH_SOURCE={:x}", hash.finalize());
}
