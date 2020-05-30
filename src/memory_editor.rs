extern crate imgui;
use imgui::{ImStr, Ui};
use std::ffi::c_void;
use std::mem::transmute;

pub struct MemoryEditor {
    data: *mut c_void,
    mem_size: usize,
    base_addr: usize,
    memory_editor: sys::MemoryEditor,
}

// TODO: Implement supplying array of bytes
// TODO: Implement write handler
// TODO: Implement DrawContents
impl MemoryEditor {
    pub fn new(mem_size: usize) -> MemoryEditor {
        MemoryEditor {
            data: std::ptr::null_mut(),
            mem_size: mem_size as usize,
            base_addr: 0,
            memory_editor: sys::MemoryEditor {
                Open: true,
                ReadOnly: true,
                Cols: 16,
                OptShowOptions: true,
                OptShowDataPreview: false,
                OptShowHexII: false,
                OptShowAscii: true,
                OptGreyOutZeroes: true,
                OptUpperCaseHex: true,
                OptMidColsCount: 8,
                OptAddrDigitsCount: 0,
                HighlightColor: Self::get_color32(255, 255, 255, 50),
                ReadFn: None,
                WriteFn: None,
                HighlightFn: None,

                // State/Internals
                ContentsWidthChanged: false,
                DataPreviewAddr: usize::max_value() - 1,
                DataEditingAddr: usize::max_value() - 1,
                DataEditingTakeFocus: false,
                DataInputBuf: [0; 32],
                AddrInputBuf: [0; 32],
                GotoAddr: usize::max_value() - 1,
                HighlightMin: usize::max_value() - 1,
                HighlightMax: usize::max_value() - 1,
                PreviewEndianess: 0,
                PreviewDataType: 4, // ImGuiDataType_S32
            },
        }
    }

    fn get_color32(r: u32, g: u32, b: u32, a: u32) -> u32 {
        a << 24 | b << 16 | g << 8 | r << 0
    }

    #[inline]
    pub fn base_addr(mut self, base_addr: usize) -> Self {
        self.base_addr = base_addr;
        self
    }

    #[inline]
    // Set to false when DrawWindow() was closed. Ignore if not using DrawWindow().
    pub fn open(&self) -> bool {
        self.memory_editor.Open
    }
    // disable any editing.
    #[inline]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.memory_editor.ReadOnly = read_only;
        self
    }
    // number of columns to display.
    #[inline]
    pub fn cols(mut self, cols: i32) -> Self {
        self.memory_editor.Cols = cols;
        self
    }
    // display options button/context menu. when disabled, options will be locked unless you provide your own UI for them.
    #[inline]
    pub fn show_options(mut self, show_options: bool) -> Self {
        self.memory_editor.OptShowOptions = show_options;
        self
    }
    // display a footer previewing the decimal/binary/hex/float representation of the currently selected bytes.
    #[inline]
    pub fn show_data_preview(mut self, show_data_preview: bool) -> Self {
        self.memory_editor.OptShowDataPreview = show_data_preview;
        self
    }
    // display values in HexII representation instead of regular hexadecimal: hide null/zero bytes, ascii values as ".X".
    #[inline]
    pub fn show_hexii(mut self, show_hexii: bool) -> Self {
        self.memory_editor.OptShowHexII = show_hexii;
        self
    }
    // display ASCII representation on the right side.
    #[inline]
    pub fn show_ascii(mut self, show_ascii: bool) -> Self {
        self.memory_editor.OptShowAscii = show_ascii;
        self
    }
    // display null/zero bytes using the TextDisabled color.
    #[inline]
    pub fn grey_out_zeroes(mut self, grey_out_zeroes: bool) -> Self {
        self.memory_editor.OptGreyOutZeroes = grey_out_zeroes;
        self
    }
    // display hexadecimal values as "FF" instead of "ff".
    #[inline]
    pub fn upper_case_hex(mut self, upper_case_hex: bool) -> Self {
        self.memory_editor.OptUpperCaseHex = upper_case_hex;
        self
    }
    // set to 0 to disable extra spacing between every mid-cols.
    #[inline]
    pub fn mid_cols_count(mut self, mid_cols_count: i32) -> Self {
        self.memory_editor.OptMidColsCount = mid_cols_count;
        self
    }
    // number of addr digits to display (default calculated based on maximum displayed addr).
    #[inline]
    pub fn addr_digits_count(mut self, addr_digits_count: i32) -> Self {
        self.memory_editor.OptAddrDigitsCount = addr_digits_count;
        self
    }
    // background color of highlighted bytes.
    #[inline]
    pub fn highlight_color(mut self, r: u32, g: u32, b: u32, a: u32) -> Self {
        self.memory_editor.HighlightColor = Self::get_color32(r, g, b, a);
        self
    }
    // optional handler to read bytes.
    #[inline]
    pub fn read_fn<F>(mut self, read_fn: F) -> Self where F: Fn(usize) -> u8 {
        let cb: Box<Box<dyn Fn(usize) -> u8>> = Box::new(Box::new(read_fn));
        self.data = Box::into_raw(cb) as *mut _;
        self.memory_editor.ReadFn = Some(read_handler);
        self
    }

    #[inline]
    pub fn build_with_window(&mut self, _: &Ui, title: &ImStr) {
        unsafe {
            self.memory_editor.DrawWindow(
                title.as_ptr(),
                self.data,
                self.mem_size,
                self.base_addr,
            );
        };
    }

    #[inline]
    pub fn build_without_window(&mut self, _: &Ui) {
        unsafe {
            self.memory_editor.DrawContents(
                self.data,
                self.mem_size,
                self.base_addr,
            );
        };
    }
}

impl std::ops::Drop for MemoryEditor {
    fn drop(&mut self) {
        // Drop the callback
        let _: Box<Box<dyn Fn(usize) -> u8>> = unsafe { Box::from_raw(self.data as *mut _) };
    }
}

extern "C" fn read_handler(data: *const ::std::os::raw::c_uchar , off: usize) -> u8 {
    let closure: &mut Box<dyn Fn(usize) -> u8> = unsafe { transmute(data) };
    closure(off)
}
