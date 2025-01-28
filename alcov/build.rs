use std::env;
use std::path::PathBuf;

fn main() {
    let alcov_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");

    if cfg!(feature = "v0") {
        let alcov_hdr = alcov_root.join("v0/alcov.h");
        println!("cargo:rerun-if-changed={}", alcov_hdr.display());

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        let bindings = bindgen::builder()
            .header(alcov_hdr.as_os_str().to_str().unwrap())
            .generate()
            .unwrap();

        bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
    }
}
