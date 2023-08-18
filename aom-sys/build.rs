use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn format_write(builder: bindgen::Builder) -> String {
    builder
        .generate()
        .unwrap()
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*")
}

fn bindgen(headers: Vec<&PathBuf>) {
    let mut builder = bindgen::builder()
        .header("data/aom.h")
        .blocklist_type("max_align_t")
        .size_t_is_usize(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts);

    for header in headers {
        builder = builder.clang_arg("-I").clang_arg(header.to_str().unwrap());
    }

    // Manually fix the comment so rustdoc won't try to pick them
    let s = format_write(builder);
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut file = File::create(out_path.join("aom.rs")).unwrap();
    let _ = file.write(s.as_bytes());
}

fn linux_shared_lib() {
    let libs = system_deps::Config::new().probe().unwrap();
    let headers = libs.all_include_paths();
    bindgen(headers);
}

// uses a submodule to build aom
fn build_aom() {
    let src_path = PathBuf::from("c").join("aom");
    let build_dir = cmake::Config::new(src_path).build();

    println!(
        "cargo:info=aom source path used: {:?}.",
        build_dir
            .canonicalize()
            .expect("Could not canonicalise to absolute path")
    );

    println!("cargo:rustc-link-search=native={}/lib", build_dir.display());
    println!("cargo:rustc-link-lib=static=aom");

    bindgen(vec![&build_dir]);
}

// uses a precompiled library
fn precompiled_aom(source_dir: &str) {
    println!("cargo:info=Linking aom lib: {}", source_dir);
    println!("cargo:rustc-link-search=native={}", source_dir);
    println!("cargo:rustc-link-lib=static=aom");

    bindgen(vec![&PathBuf::from(source_dir)]);
}

// for Windows and MacOs, need to statically link.
// also need to statically link for mobile (Android/IOS) but for that, also need to cross compile.
// for cross compiling, use the precompiled setting.
// otherwise, build_aom will compile libaom from a git submodule and statically link it.
fn main() {
    match env::var("LIB_AOM_MODE") {
        Ok(mode) if mode == "shared-library" => {
            linux_shared_lib();
        }
        Ok(mode) if mode == "precompiled" => {
            let src_dir = env::var("LIB_AOM_DIR").unwrap();
            precompiled_aom(&src_dir);
        }
        _ => {
            build_aom();
        }
    }
}
