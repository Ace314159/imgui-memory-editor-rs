use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.cpp");
    println!("cargo:rustc-link-lib=cimgui");

    let imgui_dir = std::env::var_os("DEP_IMGUI_THIRD_PARTY").expect("imgui-sys expected to specify imgui dir");
    let imgui_dir = Path::new(&imgui_dir).join("imgui");

    cc::Build::new()
    .cpp(true)
    .include(imgui_dir)
    .file("wrapper.cpp")
    .compile("imgui_memory_editor");
}