use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::{Compression, GzBuilder};

fn main() {
    let operating_system =
        env::var("CARGO_CFG_TARGET_OS").expect("Cargo supplies the target operating system");
    let architecture =
        env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo supplies the target architecture");
    let suffix = match (operating_system.as_str(), architecture.as_str()) {
        ("macos", "aarch64") => "macos-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "x86_64") => "linux-x86_64",
        unsupported => panic!("unsupported Kipferl host: {unsupported:?}"),
    };
    let assets =
        Path::new(&env::var("CARGO_MANIFEST_DIR").expect("manifest directory")).join("assets");
    let full = assets.join(format!("pocketpy-kipferl-{suffix}"));
    let core = assets.join(format!("pocketpy-kipferl-core-{suffix}"));
    println!("cargo:rerun-if-changed={}", full.display());
    println!("cargo:rerun-if-changed={}", core.display());

    let full_content = fs::read(&full).expect("read embedded full PocketPy runtime");
    let core_content = fs::read(&core).expect("read embedded core PocketPy runtime");
    let output_directory = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"));
    compress(&full_content, &output_directory.join("full-runtime.gz"));
    compress(&core_content, &output_directory.join("core-runtime.gz"));
    fs::write(
        output_directory.join("embedded_runtime_keys.rs"),
        format!(
            "const EMBEDDED_FULL_RUNTIME_KEY: u64 = 0x{:016x};\n#[cfg(test)]\nconst EMBEDDED_CORE_RUNTIME_KEY: u64 = 0x{:016x};\n",
            stable_hash(&full_content),
            stable_hash(&core_content)
        ),
    )
    .expect("write embedded runtime cache keys");
}

fn compress(content: &[u8], output: &Path) {
    let file = fs::File::create(output).expect("create compressed runtime");
    let mut encoder = GzBuilder::new().mtime(0).write(file, Compression::best());
    encoder
        .write_all(content)
        .expect("compress embedded runtime");
    encoder.finish().expect("finish compressed runtime");
}

fn stable_hash(content: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
