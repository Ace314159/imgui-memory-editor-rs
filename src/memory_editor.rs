extern crate imgui;
use imgui::{ImStr, Ui};
use std::ffi::c_void;

enum Data<'a> {
    None,
    Bytes(&'a mut Vec<u8>),
    Callback(Box<dyn FnMut(usize) -> u8 + 'a>),
}

pub struct MemoryEditor<'a> {
    data: Data<'a>,
    mem_size: usize,
    base_addr: usize,
    raw: sys::MemoryEditor,
}

// TODO: Implement write handler
impl<'a> MemoryEditor<'a> {
    pub fn new() -> MemoryEditor<'a> {
        let mut raw = Default::default();
        unsafe { sys::Editor_Create(&mut raw) }
        MemoryEditor {
            data: Data::None,
            mem_size: 0,
            base_addr: 0,
            raw,
        }
    }

    #[inline]
    pub fn mem_size(mut self, mem_size: usize) -> Self {
        self.mem_size = mem_size;
        self
    }

    #[inline]
    pub fn base_addr(mut self, base_addr: usize) -> Self {
        self.base_addr = base_addr;
        self
    }

    #[inline]
    // Set to false when DrawWindow() was closed. Ignore if not using DrawWindow().
    pub fn open(&self) -> bool {
        self.raw.Open
    }
    // disable any editing.
    #[inline]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.raw.ReadOnly = read_only;
        self
    }
    // number of columns to display.
    #[inline]
    pub fn cols(mut self, cols: i32) -> Self {
        self.raw.Cols = cols;
        self
    }
    // display options button/context menu. when disabled, options will be locked unless you provide your own UI for them.
    #[inline]
    pub fn show_options(mut self, show_options: bool) -> Self {
        self.raw.OptShowOptions = show_options;
        self
    }
    // display a footer previewing the decimal/binary/hex/float representation of the currently selected bytes.
    #[inline]
    pub fn show_data_preview(mut self, show_data_preview: bool) -> Self {
        self.raw.OptShowDataPreview = show_data_preview;
        self
    }
    // display values in HexII representation instead of regular hexadecimal: hide null/zero bytes, ascii values as ".X".
    #[inline]
    pub fn show_hexii(mut self, show_hexii: bool) -> Self {
        self.raw.OptShowHexII = show_hexii;
        self
    }
    // display ASCII representation on the right side.
    #[inline]
    pub fn show_ascii(mut self, show_ascii: bool) -> Self {
        self.raw.OptShowAscii = show_ascii;
        self
    }
    // display null/zero bytes using the TextDisabled color.
    #[inline]
    pub fn grey_out_zeroes(mut self, grey_out_zeroes: bool) -> Self {
        self.raw.OptGreyOutZeroes = grey_out_zeroes;
        self
    }
    // display hexadecimal values as "FF" instead of "ff".
    #[inline]
    pub fn upper_case_hex(mut self, upper_case_hex: bool) -> Self {
        self.raw.OptUpperCaseHex = upper_case_hex;
        self
    }
    // set to 0 to disable extra spacing between every mid-cols.
    #[inline]
    pub fn mid_cols_count(mut self, mid_cols_count: i32) -> Self {
        self.raw.OptMidColsCount = mid_cols_count;
        self
    }
    // number of addr digits to display (default calculated based on maximum displayed addr).
    #[inline]
    pub fn addr_digits_count(mut self, addr_digits_count: i32) -> Self {
        self.raw.OptAddrDigitsCount = addr_digits_count;
        self
    }
    // background color of highlighted bytes.
    #[inline]
    pub fn highlight_color(mut self, r: u32, g: u32, b: u32, a: u32) -> Self {
        self.raw.HighlightColor = a << 24 | b << 16 | g << 8 | r << 0;
        self
    }
    // optional handler to read bytes.
    #[inline]
    pub fn read_fn<F>(mut self, read_fn: F) -> Self where F: FnMut(usize) -> u8 + 'a {
        self.data = Data::Callback(Box::new(Box::new(read_fn)));
        self
    }

    #[inline]
    pub fn bytes(mut self, bytes: &'a mut Vec<u8>) -> Self {
        self.mem_size = bytes.len();
        self.data = Data::Bytes(bytes);
        self
    }

    #[inline]
    pub fn build_with_window(&mut self, _: &Ui, title: &ImStr) {
        unsafe {
            sys::Editor_DrawWindow(
                &mut self.raw,
                title.as_ptr(),
                self.data(),
                self.mem_size,
                self.base_addr,
            );
        };
    }

    #[inline]
    pub fn build_without_window(&mut self, _: &Ui) {
        unsafe {
            sys::Editor_DrawContents(
                &mut self.raw,
                self.data(),
                self.mem_size,
                self.base_addr,
            );
        };
    }

    fn data(&mut self) -> *mut c_void {
        self.raw.ReadFn = None;
        match &mut self.data {
            Data::None => panic!("No data specified!"),
            Data::Bytes(bytes) => bytes.as_mut_ptr() as *mut _ as *mut c_void,
            Data::Callback(read_fn) => {
                unsafe extern "C" fn wrapper(data: *const u8, off: usize) -> u8 {
                    (*(data as *mut Box<dyn FnMut(usize) -> u8>))(off)
                }
                self.raw.ReadFn = Some(wrapper);
                read_fn as *mut Box<dyn FnMut(usize) -> u8> as *mut c_void
            },
        }
    }
}


