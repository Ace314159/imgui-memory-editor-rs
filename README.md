# Dear ImGui Memory Editor Rust Bindings

[![Latest release on crates.io](https://meritbadge.herokuapp.com/imgui-memory-editor)](https://crates.io/crates/imgui-memory-editor)
[![Documentation on docs.rs](https://docs.rs/imgui-memory-editor/badge.svg)](https://docs.rs/imgui-memory-editor)

## Usage

This package is intended to be used with imgui-rs.

You can either use memory using a custom struct and closures or a slice.

### Using a Slice

```rust
let mut vec = vec![0xFF; 0x100];
// Can also use a &mut [u8] if you want to use the editor to modify the slice
let mut memory_editor = MemoryEditor::<&[u8]>::new()
    .draw_window(im_str!("Memory")) // Can omit if you don't want to create a window
    .read_only(true);

// In your main loop, draw the memory editor with draw_vec()
if memory_editor.open() { // open() can be omitted if draw_window was not used
    memory_editor.draw_vec(&ui, &mut vec)
}
```

### Using a Custom Struct

```rust
let mut mem = Memory::new(); // Custom struct
let mut times_written = 0; // Variable captured in closure
let mut memory_editor = MemoryEditor::<Memory>::new()
    .draw_window(im_str!("Name"))
    .read_only(false)
    .mem_size(0x100)
    .read_fn(|mem, offset| mem.read(offset))
    .write_fn(|mem, offset, value| {
        mem.write(offset);
        times_written += 1; // Variable used in closure
        println!("Written {} times", times_written);
    });

// In your main loop, draw the memory editor with draw()
if memory_editor.open() {
    memory_editor.draw(&ui, &mut mem)
}
```
