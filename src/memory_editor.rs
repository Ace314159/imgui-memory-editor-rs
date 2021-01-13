extern crate imgui;
use imgui::{ImColor, ImStr, Ui};
use std::ffi::c_void;

// TODO: Alias ReadHandlerTrait and writeHandlerTrait to FnMuts once trait_alias is stabilized
type ReadHandler<'a, T> = Option<Box<dyn FnMut(&T, usize) -> u8 + 'a>>;
type WriteHandler<'a, T> = Option<Box<dyn FnMut(&mut T, usize, u8) + 'a>>;
type MemData<'a, 'b, T> = (&'b mut ReadHandler<'a, T>, &'b mut WriteHandler<'a, T>, &'b mut T);

pub struct MemoryEditor<'a, T> {
    window_name: Option<&'a ImStr>,
    read_fn: ReadHandler<'a, T>,
    write_fn: WriteHandler<'a, T>,
    mem_size: usize,
    base_addr: usize,
    raw: sys::MemoryEditor,
}

impl<'a, T> MemoryEditor<'a, T> {
    pub fn new() -> MemoryEditor<'a, T> {
        let mut raw = Default::default();
        unsafe { sys::Editor_Create(&mut raw) }
        MemoryEditor {
            window_name: None,
            read_fn: None,
            write_fn: None,
            mem_size: 0,
            base_addr: 0,
            raw,
        }
    }

    // Size of memory in bytes (Automatically set if using bytes)
    #[inline]
    pub fn mem_size(mut self, mem_size: usize) -> Self {
        self.mem_size = mem_size;
        self
    }

    // The base addr displayed
    #[inline]
    pub fn base_addr(mut self, base_addr: usize) -> Self {
        self.base_addr = base_addr;
        self
    }

    // Set to false when DrawWindow() was closed. Ignore if not using DrawWindow().
    #[inline]
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
    pub fn highlight_color(mut self, color: ImColor) -> Self {
        self.raw.HighlightColor = color.into();
        self
    }
    // optional handler to read bytes.
    #[inline]
    pub fn read_fn<F>(mut self, read_fn: F) -> Self where F: FnMut(&T, usize) -> u8 + 'a {
        self.read_fn = Some(Box::new(read_fn));
        self
    }
    // optional handler to write bytes.
    #[inline]
    pub fn write_fn<F>(mut self, write_fn: F) -> Self where F: FnMut(&mut T, usize, u8) + 'a {
        self.write_fn = Some(Box::new(write_fn));
        self
    }

    // When drawing, create a window with this name
    #[inline]
    pub fn draw_window(mut self, window_name: &'a ImStr) -> Self {
        self.window_name = Some(window_name);
        self
    }
    // No longer create a window when drawing
    #[inline]
    pub fn no_window(mut self) -> Self {
        self.window_name = None;
        self
    }

    // Draw the memory editor with read and write functions set
    #[inline]
    pub fn draw(&mut self, _: &Ui, user_data: &mut T) {
        self.raw.ReadFn = Some(read_wrapper::<T>);
        self.raw.WriteFn = Some(write_wrapper::<T>);

        let mut data = (&mut self.read_fn, &mut self.write_fn, user_data);
        let mem_data = &mut data as *mut MemData<T> as *mut c_void;
        if let Some(title) = self.window_name {
            unsafe {
                sys::Editor_DrawWindow(
                    &mut self.raw,
                    title.as_ptr(),
                    mem_data,
                    self.mem_size,
                    self.base_addr,
                );
            };
        } else {
            unsafe {
                sys::Editor_DrawContents(
                    &mut self.raw,
                    mem_data,
                    self.mem_size,
                    self.base_addr,
                );
            };
        }
    }
}

unsafe extern "C" fn read_wrapper<'a, T>(data: *const u8, off: usize) -> u8 {
    let (read_fn, _, user_data) = &mut *(data as *mut MemData<T>);

    if let Some(read_fn) = read_fn {
        read_fn(user_data, off)
    } else { panic!("No Read Handler Set!") }
}
unsafe extern "C" fn write_wrapper<'a, T>(data: *mut u8, off: usize, d: u8) {
    let (_, write_fn, user_data) = &mut *(data as *mut MemData<T>);

    if let Some(write_fn) = write_fn {
        write_fn(user_data, off, d)
    } else { panic!("No Write Handler Set!") }
}
