#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::exit,
    clippy::panic_in_result_fn
)]

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use flate2::{Compression, GzBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let operating_system = env::var("CARGO_CFG_TARGET_OS")?;
    let architecture = env::var("CARGO_CFG_TARGET_ARCH")?;
    let suffix = match (operating_system.as_str(), architecture.as_str()) {
        ("macos", "aarch64") => "macos-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "x86_64") => "linux-x86_64",
        unsupported => return Err(format!("unsupported Kipferl host: {unsupported:?}").into()),
    };
    let assets = Path::new(&env::var("CARGO_MANIFEST_DIR")?).join("assets");
    let full = assets.join(format!("pocketpy-kipferl-{suffix}"));
    let core = assets.join(format!("pocketpy-kipferl-core-{suffix}"));
    println!("cargo:rerun-if-changed={}", full.display());
    println!("cargo:rerun-if-changed={}", core.display());

    let full_content = read_asset(&full)?;
    let core_content = read_asset(&core)?;
    let output_directory =
        PathBuf::from(env::var_os("OUT_DIR").ok_or("Cargo must supply OUT_DIR")?);
    compress(&full_content, &output_directory.join("full-runtime.gz"))?;
    compress(&core_content, &output_directory.join("core-runtime.gz"))?;
    let evidence = assets.join("../../../compatibility/packages");
    for name in ["catalog", "popularity-catalog", "popularity-audit"] {
        let source = evidence.join(format!("{name}.json"));
        println!("cargo:rerun-if-changed={}", source.display());
        let content = fs::read(&source)?;
        if content.len() > 8 * 1024 * 1024 {
            return Err(format!("embedded evidence {} exceeds 8 MiB", source.display()).into());
        }
        compress(&content, &output_directory.join(format!("{name}.json.gz")))?;
    }
    fs::write(
        output_directory.join("embedded_runtime_keys.rs"),
        format!(
            "const EMBEDDED_FULL_RUNTIME_KEY: u64 = {};\n#[cfg(test)]\nconst EMBEDDED_CORE_RUNTIME_KEY: u64 = {};\n",
            rust_hex(stable_hash(&full_content)),
            rust_hex(stable_hash(&core_content))
        ),
    )?;
    Ok(())
}

fn read_asset(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("cannot read embedded runtime {}: {error}", path.display()),
        )
    })
}

fn compress(content: &[u8], output: &Path) -> io::Result<()> {
    let file = fs::File::create(output).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "cannot create compressed runtime {}: {error}",
                output.display()
            ),
        )
    })?;
    let mut encoder = GzBuilder::new().mtime(0).write(file, Compression::best());
    encoder.write_all(content)?;
    encoder.finish()?;
    Ok(())
}

fn stable_hash(content: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn rust_hex(value: u64) -> String {
    let [byte0, byte1, byte2, byte3, byte4, byte5, byte6, byte7] = value.to_be_bytes();
    format!(
        "0x{byte0:02x}{byte1:02x}_{byte2:02x}{byte3:02x}_{byte4:02x}{byte5:02x}_{byte6:02x}{byte7:02x}"
    )
}
