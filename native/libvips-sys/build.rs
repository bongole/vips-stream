use std::env;
use std::path::PathBuf;

fn main() {
    let lib = pkg_config::Config::new()
        .atleast_version("8.9")
        .probe("vips")
        .unwrap();

    let bindings = {
        let mut builder = bindgen::Builder::default()
            .header("src/wrapper.h")
            .rustified_enum("*")
            .bitfield_enum("*Flags?")
            .whitelist_function("vips_.*")
            .whitelist_function("g_(setenv|object|signal|value|type).*")
            .layout_tests(false)
            .generate_comments(false);
        
        for path in lib.include_paths {
            builder = builder.clang_arg(format!("-I{}", path.to_str().unwrap()));
        }

        builder
    }
    .generate()
    .expect("Unable to generate bindings!");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
