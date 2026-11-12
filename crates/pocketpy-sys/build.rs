use std::path::PathBuf;

fn main() {
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../pocketpy/vendor");
    let source = vendor.join("pocketpy.c");
    let header = vendor.join("pocketpy.h");

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", header.display());

    cc::Build::new()
        .file(source)
        .include(vendor)
        .define("NDEBUG", None)
        .flag_if_supported("-std=c11")
        .flag_if_supported("-fno-strict-aliasing")
        // Python ints are 64-bit in PocketPy; hashing and slicing rely on
        // two's-complement wrapping, which C leaves undefined. Arithmetic that
        // overflows raises OverflowError (patch 0019) instead.
        .flag_if_supported("-fwrapv")
        .warnings(false)
        .compile("pocketpy");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    }
}
