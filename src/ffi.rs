/*
Copyright (c) 2016 Saurav Sachidanand

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/

//! Raw bindings to the bundled [Native File Dialog Extended] C library.
//!
//! These mirror the UTF-8 (`U8`) half of `nfd.h` from the bundled version (see
//! [`NFD_INTERFACE_VERSION`]). They are provided as an escape hatch for features that the safe
//! API does not wrap yet. Everything here is `unsafe` and follows the C library's rules: call
//! [`NFD_Init`] on the current thread before any other function, call [`NFD_Quit`] afterwards,
//! free every returned path with the matching free function, and do not open dialogs from more
//! than one thread at a time.
//!
//! The safe API in the crate root already does all of this; prefer it when you can. These
//! bindings follow the bundled C library, so they may change whenever it is upgraded.
//!
//! [Native File Dialog Extended]: https://github.com/btzy/nativefiledialog-extended

#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

/// A UTF-8 encoded C character.
pub type nfdu8char_t = c_char;

/// Number of filter items passed to a dialog.
pub type nfdfiltersize_t = c_uint;

/// Interface version passed to the `*_With_Impl` functions.
pub type nfdversion_t = usize;

/// Number of entries in a path set.
#[cfg(any(windows, target_vendor = "apple"))]
pub type nfdpathsetsize_t = std::os::raw::c_ulong;

/// Number of entries in a path set.
#[cfg(not(any(windows, target_vendor = "apple")))]
pub type nfdpathsetsize_t = c_uint;

/// Opaque path set returned by the multiple-selection dialogs.
pub type nfdpathset_t = c_void;

/// Result of an NFD call. One of [`NFD_ERROR`], [`NFD_OKAY`] or [`NFD_CANCEL`].
///
/// This is a plain integer rather than a Rust `enum` so that an unexpected value coming from C can
/// never cause undefined behaviour.
pub type nfdresult_t = c_int;

/// Programmatic error; see [`NFD_GetError`].
pub const NFD_ERROR: nfdresult_t = 0;
/// The user pressed okay, or the call succeeded.
pub const NFD_OKAY: nfdresult_t = 1;
/// The user pressed cancel.
pub const NFD_CANCEL: nfdresult_t = 2;

/// The version of the `*_With_Impl` argument structs these bindings describe.
pub const NFD_INTERFACE_VERSION: nfdversion_t = 2;

/// A filter: a display `name` and a comma-separated list of extensions in `spec` (e.g. `"png,jpg"`).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nfdu8filteritem_t {
    pub name: *const nfdu8char_t,
    pub spec: *const nfdu8char_t,
}

/// No parent window.
pub const NFD_WINDOW_HANDLE_TYPE_UNSET: usize = 0;
/// `handle` is a Win32 `HWND`.
pub const NFD_WINDOW_HANDLE_TYPE_WINDOWS: usize = 1;
/// `handle` is an `NSWindow*`.
pub const NFD_WINDOW_HANDLE_TYPE_COCOA: usize = 2;
/// `handle` is an X11 `Window` (an XID, cast to a pointer).
pub const NFD_WINDOW_HANDLE_TYPE_X11: usize = 3;
/// `handle` is a `wl_surface*`. The bundled build does not enable Wayland parenting, so this
/// is accepted but ignored.
pub const NFD_WINDOW_HANDLE_TYPE_WAYLAND: usize = 4;

/// A native parent window handle.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nfdwindowhandle_t {
    /// One of the `NFD_WINDOW_HANDLE_TYPE_*` constants.
    pub type_: usize,
    pub handle: *mut c_void,
}

impl Default for nfdwindowhandle_t {
    fn default() -> Self {
        nfdwindowhandle_t {
            type_: NFD_WINDOW_HANDLE_TYPE_UNSET,
            handle: std::ptr::null_mut(),
        }
    }
}

/// Arguments for [`NFD_OpenDialogU8_With_Impl`] and [`NFD_OpenDialogMultipleU8_With_Impl`].
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nfdopendialogu8args_t {
    pub filterList: *const nfdu8filteritem_t,
    pub filterCount: nfdfiltersize_t,
    pub defaultPath: *const nfdu8char_t,
    pub parentWindow: nfdwindowhandle_t,
    pub title: *const nfdu8char_t,
    pub acceptLabel: *const nfdu8char_t,
    pub cancelLabel: *const nfdu8char_t,
}

/// Arguments for [`NFD_SaveDialogU8_With_Impl`].
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nfdsavedialogu8args_t {
    pub filterList: *const nfdu8filteritem_t,
    pub filterCount: nfdfiltersize_t,
    pub defaultPath: *const nfdu8char_t,
    pub defaultName: *const nfdu8char_t,
    pub parentWindow: nfdwindowhandle_t,
    pub title: *const nfdu8char_t,
    pub acceptLabel: *const nfdu8char_t,
    pub cancelLabel: *const nfdu8char_t,
}

/// Arguments for [`NFD_PickFolderU8_With_Impl`] and [`NFD_PickFolderMultipleU8_With_Impl`].
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nfdpickfolderu8args_t {
    pub defaultPath: *const nfdu8char_t,
    pub parentWindow: nfdwindowhandle_t,
    pub title: *const nfdu8char_t,
    pub acceptLabel: *const nfdu8char_t,
    pub cancelLabel: *const nfdu8char_t,
}

macro_rules! zeroed_default {
    ($($ty:ty),*) => {$(
        impl Default for $ty {
            fn default() -> Self {
                // SAFETY: every field is a raw pointer, an integer or a `nfdwindowhandle_t`,
                // for all of which all-zeroes (null / 0 / UNSET) is a valid value.
                unsafe { std::mem::zeroed() }
            }
        }
    )*};
}

zeroed_default!(
    nfdopendialogu8args_t,
    nfdsavedialogu8args_t,
    nfdpickfolderu8args_t
);

unsafe extern "C" {
    /// Free a path returned by a single-selection dialog.
    pub fn NFD_FreePathU8(filePath: *mut nfdu8char_t);
    /// Initialise NFD on the current thread.
    pub fn NFD_Init() -> nfdresult_t;
    /// De-initialise NFD, if [`NFD_Init`] returned [`NFD_OKAY`].
    pub fn NFD_Quit();

    pub fn NFD_OpenDialogU8(
        outPath: *mut *mut nfdu8char_t,
        filterList: *const nfdu8filteritem_t,
        filterCount: nfdfiltersize_t,
        defaultPath: *const nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_OpenDialogU8_With_Impl(
        version: nfdversion_t,
        outPath: *mut *mut nfdu8char_t,
        args: *const nfdopendialogu8args_t,
    ) -> nfdresult_t;

    pub fn NFD_OpenDialogMultipleU8(
        outPaths: *mut *const nfdpathset_t,
        filterList: *const nfdu8filteritem_t,
        filterCount: nfdfiltersize_t,
        defaultPath: *const nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_OpenDialogMultipleU8_With_Impl(
        version: nfdversion_t,
        outPaths: *mut *const nfdpathset_t,
        args: *const nfdopendialogu8args_t,
    ) -> nfdresult_t;

    pub fn NFD_SaveDialogU8(
        outPath: *mut *mut nfdu8char_t,
        filterList: *const nfdu8filteritem_t,
        filterCount: nfdfiltersize_t,
        defaultPath: *const nfdu8char_t,
        defaultName: *const nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_SaveDialogU8_With_Impl(
        version: nfdversion_t,
        outPath: *mut *mut nfdu8char_t,
        args: *const nfdsavedialogu8args_t,
    ) -> nfdresult_t;

    pub fn NFD_PickFolderU8(
        outPath: *mut *mut nfdu8char_t,
        defaultPath: *const nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_PickFolderU8_With_Impl(
        version: nfdversion_t,
        outPath: *mut *mut nfdu8char_t,
        args: *const nfdpickfolderu8args_t,
    ) -> nfdresult_t;

    pub fn NFD_PickFolderMultipleU8(
        outPaths: *mut *const nfdpathset_t,
        defaultPath: *const nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_PickFolderMultipleU8_With_Impl(
        version: nfdversion_t,
        outPaths: *mut *const nfdpathset_t,
        args: *const nfdpickfolderu8args_t,
    ) -> nfdresult_t;

    /// The last error, or null. The string is owned by NFD.
    pub fn NFD_GetError() -> *const c_char;
    /// Clear the last error (and free any memory held for it).
    pub fn NFD_ClearError();

    pub fn NFD_PathSet_GetCount(
        pathSet: *const nfdpathset_t,
        count: *mut nfdpathsetsize_t,
    ) -> nfdresult_t;
    /// Get the path at `index`; free it with [`NFD_PathSet_FreePathU8`].
    pub fn NFD_PathSet_GetPathU8(
        pathSet: *const nfdpathset_t,
        index: nfdpathsetsize_t,
        outPath: *mut *mut nfdu8char_t,
    ) -> nfdresult_t;
    pub fn NFD_PathSet_FreePathU8(filePath: *const nfdu8char_t);
    pub fn NFD_PathSet_Free(pathSet: *const nfdpathset_t);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    const LAYOUT_LEN: usize = 29;

    unsafe extern "C" {
        static nfd_rs_layout: [usize; LAYOUT_LEN];
        static nfd_rs_layout_len: usize;
    }

    /// The bindings must describe the same ABI as the bundled `nfd.h`. The C side of this
    /// table is `csrc/nfd_rs_glue.c`, compiled against that header by `build.rs`.
    #[test]
    fn layouts_match_the_c_header() {
        let expected: [usize; LAYOUT_LEN] = [
            NFD_INTERFACE_VERSION,
            size_of::<nfdresult_t>(),
            size_of::<nfdpathsetsize_t>(),
            size_of::<nfdfiltersize_t>(),
            size_of::<nfdu8filteritem_t>(),
            offset_of!(nfdu8filteritem_t, spec),
            size_of::<nfdwindowhandle_t>(),
            offset_of!(nfdwindowhandle_t, handle),
            size_of::<nfdopendialogu8args_t>(),
            offset_of!(nfdopendialogu8args_t, filterCount),
            offset_of!(nfdopendialogu8args_t, defaultPath),
            offset_of!(nfdopendialogu8args_t, parentWindow),
            offset_of!(nfdopendialogu8args_t, title),
            offset_of!(nfdopendialogu8args_t, acceptLabel),
            offset_of!(nfdopendialogu8args_t, cancelLabel),
            size_of::<nfdsavedialogu8args_t>(),
            offset_of!(nfdsavedialogu8args_t, filterCount),
            offset_of!(nfdsavedialogu8args_t, defaultPath),
            offset_of!(nfdsavedialogu8args_t, defaultName),
            offset_of!(nfdsavedialogu8args_t, parentWindow),
            offset_of!(nfdsavedialogu8args_t, title),
            offset_of!(nfdsavedialogu8args_t, acceptLabel),
            offset_of!(nfdsavedialogu8args_t, cancelLabel),
            size_of::<nfdpickfolderu8args_t>(),
            offset_of!(nfdpickfolderu8args_t, parentWindow),
            offset_of!(nfdpickfolderu8args_t, title),
            offset_of!(nfdpickfolderu8args_t, acceptLabel),
            offset_of!(nfdpickfolderu8args_t, cancelLabel),
            (NFD_ERROR * 100 + NFD_OKAY * 10 + NFD_CANCEL) as usize,
        ];
        // SAFETY: both statics are immutable arrays/integers defined in C.
        let (actual, len) = unsafe { (nfd_rs_layout, nfd_rs_layout_len) };
        assert_eq!(len, LAYOUT_LEN);
        assert_eq!(actual, expected);
    }
}
