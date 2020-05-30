fn main() {
    /*println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .generate_inline_functions(true)
        .whitelist_type("MemoryEditor")
        .whitelist_function("MemoryEditor_.*")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");*/
    
    cc::Build::new()
    .cpp(true)
    .file("wrapper.cpp")
    .compile("imgui-memory-editor");
}