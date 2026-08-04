use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let operating_system =
        env::var("CARGO_CFG_TARGET_OS").expect("Cargo supplies the target operating system");
    let architecture =
        env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo supplies the target architecture");
    let filename = match (operating_system.as_str(), architecture.as_str()) {
        ("macos", "aarch64") => "pocketpy-ucharm-macos-aarch64",
        ("macos", "x86_64") => "pocketpy-ucharm-macos-x86_64",
        ("linux", "aarch64") => "pocketpy-ucharm-linux-aarch64",
        ("linux", "x86_64") => "pocketpy-ucharm-linux-x86_64",
        unsupported => panic!("unsupported μcharm host: {unsupported:?}"),
    };
    let manifest = Path::new(&env::var("CARGO_MANIFEST_DIR").expect("manifest directory"))
        .join("../../cli/src/stubs")
        .join(filename);
    println!("cargo:rerun-if-changed={}", manifest.display());

    let content = fs::read(&manifest).expect("read embedded PocketPy runtime");
    let key = stable_hash(&content);
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"))
        .join("embedded_runtime_key.rs");
    fs::write(
        output,
        format!("const EMBEDDED_RUNTIME_KEY: u64 = 0x{key:016x};\n"),
    )
    .expect("write embedded runtime cache key");
}

fn stable_hash(content: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
