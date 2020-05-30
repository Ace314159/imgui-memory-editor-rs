fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .generate_inline_functions(true)
        .default_enum_style(bindgen::EnumVariation::Consts)
        .size_t_is_usize(true)
        .prepend_enum_name(false)
        .generate_comments(false)
        .layout_tests(true)
        .whitelist_type("MemoryEditor")
        .whitelist_function("MemoryEditor_.*")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
    
    cc::Build::new()
    .cpp(true)
    .file("wrapper.cpp")
    .flag_if_supported("-fno-inline-functions") // Clang
    .flag_if_supported("-fkeep-inline-functions") // G++
    .flag_if_supported("-Ob0") // MSVC
    .flag_if_supported("-Zc:inline")
    .compile("imgui-memory-editor");
}