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

//! Native open, save and folder dialogs for Windows, macOS and Linux.
//!
//! `nfd` wraps the bundled [Native File Dialog Extended] C library, so every dialog is the one
//! the platform itself provides: `IFileDialog` on Windows, `NSOpenPanel`/`NSSavePanel` on macOS,
//! and the XDG desktop portal (or GTK 3, with the `gtk3` feature) on Linux and the BSDs.
//!
//! ```no_run
//! use nfd::Response;
//!
//! match nfd::open_file_dialog(Some("png,jpg;pdf"), None)? {
//!     Response::Okay(path) => println!("selected {path}"),
//!     Response::OkayMultiple(paths) => println!("selected {paths:?}"),
//!     Response::Cancel => println!("cancelled"),
//! }
//! # Ok::<(), nfd::NFDError>(())
//! ```
//!
//! The builder exposes everything else:
//!
//! ```no_run
//! let response = nfd::dialog_save()
//!     .title("Export image")
//!     .add_filter("PNG image", "png")
//!     .add_filter("JPEG image", "jpg,jpeg")
//!     .default_path("/tmp")
//!     .default_name("untitled.png")
//!     .open()?;
//! # Ok::<(), nfd::NFDError>(())
//! ```
//!
//! # Filters
//!
//! [`DialogBuilder::add_filter`] takes a display name and a comma-separated list of extensions.
//! The older filter-list strings accepted by [`DialogBuilder::filter`] and the `open_*` functions
//! still work: commas separate extensions within a filter and semicolons separate filters, so
//! `"png,jpg;pdf"` offers one filter for PNG and JPEG files and another for PDF files.
//!
//! # Threads
//!
//! Dialogs are modal, so this crate opens one at a time: concurrent calls from several threads
//! wait for each other. In addition:
//!
//! * **macOS** only allows dialogs on the main thread. Calls from any other thread return an
//!   error instead of crashing.
//! * **Linux with the `gtk3` feature**: GTK may only be used from one thread, so every dialog must
//!   be opened from the thread that opened the first one; other threads get an error. The default
//!   portal backend has no such restriction.
//! * **Windows**: any thread works; COM is initialised for the duration of each dialog.
//!
//! # Cargo features
//!
//! * `gtk3`: on Linux and the BSDs, use GTK 3 instead of the XDG desktop portal.
//! * `raw-window-handle`: adds [`DialogBuilder::parent`], which ties a dialog to one of your
//!   windows so that it opens in front of it.
//!
//! [Native File Dialog Extended]: https://github.com/btzy/nativefiledialog-extended

pub mod error;
pub mod ffi;

pub use error::NFDError;

use ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

/// Result of opening a file dialog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Response {
    /// The user picked a file or folder. The `String` is its full path.
    Okay(String),
    /// The user picked one or more files in a [`DialogType::MultipleFiles`] dialog.
    /// Each `String` is a full path; they need not share a parent folder.
    OkayMultiple(Vec<String>),
    /// The user cancelled the dialog.
    Cancel,
}

/// The kind of dialog to open.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DialogType {
    /// Pick one existing file.
    SingleFile,
    /// Pick one or more existing files.
    MultipleFiles,
    /// Choose where to save a file.
    SaveFile,
    /// Pick one folder.
    PickFolder,
}

pub type Result<T> = std::result::Result<T, NFDError>;

/// Configures and opens a dialog.
///
/// Create one with [`dialog`], [`dialog_multiple`], [`dialog_save`], [`dialog_pick_folder`] or
/// [`DialogBuilder::new`], chain the options you need, then call [`open`](DialogBuilder::open).
#[derive(Debug, Clone)]
pub struct DialogBuilder<'a> {
    dialog_type: DialogType,
    filters: Vec<Filter>,
    default_path: Option<&'a Path>,
    default_name: Option<&'a str>,
    title: Option<&'a str>,
    #[cfg(feature = "raw-window-handle")]
    parent: Option<std::result::Result<parent::Parent, String>>,
}

impl<'a> DialogBuilder<'a> {
    pub fn new(dialog_type: DialogType) -> Self {
        DialogBuilder {
            dialog_type,
            filters: Vec::new(),
            default_path: None,
            default_name: None,
            title: None,
            #[cfg(feature = "raw-window-handle")]
            parent: None,
        }
    }

    /// Adds filters from a filter-list string such as `"png,jpg;pdf"` (commas separate
    /// extensions, semicolons separate filters). Each filter is named after its extensions.
    ///
    /// Ignored by folder dialogs.
    pub fn filter(&mut self, filter_list: &str) -> &mut Self {
        self.filters.extend(parse_filter_list(filter_list));
        self
    }

    /// Adds a filter with a display name and a comma-separated list of extensions, e.g.
    /// `add_filter("Images", "png,jpg,jpeg")`. Leading `*.` or `.` on extensions is ignored.
    ///
    /// Ignored by folder dialogs. macOS does not show filter names.
    pub fn add_filter(&mut self, name: &str, extensions: &str) -> &mut Self {
        if let Some(spec) = normalize_spec(extensions) {
            let name = if name.trim().is_empty() {
                generated_name(&spec)
            } else {
                name.to_owned()
            };
            self.filters.push(Filter { name, spec });
        }
        self
    }

    /// The folder the dialog starts in. Use an absolute path; if it is omitted, the platform
    /// chooses (often the most recently used folder).
    pub fn default_path<P: AsRef<Path> + ?Sized>(&mut self, path: &'a P) -> &mut Self {
        self.default_path = Some(path.as_ref());
        self
    }

    /// The file name a save dialog suggests. Ignored by other dialogs.
    pub fn default_name(&mut self, name: &'a str) -> &mut Self {
        self.default_name = Some(name);
        self
    }

    /// The dialog's title, where the platform shows one.
    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.title = Some(title);
        self
    }

    /// Ties the dialog to `window`, so it opens in front of it instead of behind. Supported for
    /// Win32, AppKit (the window regains focus when the dialog closes), Xlib and XCB windows;
    /// other handles, including Wayland, are accepted but the dialog is opened unparented.
    ///
    /// Requires the `raw-window-handle` feature. If the handle cannot be obtained,
    /// [`open`](DialogBuilder::open) returns an error.
    #[cfg(feature = "raw-window-handle")]
    pub fn parent<W: raw_window_handle::HasWindowHandle + ?Sized>(
        &mut self,
        window: &'a W,
    ) -> &mut Self {
        self.parent = Some(parent::Parent::from_window(window));
        self
    }

    /// Opens the dialog and waits for the user to close it.
    pub fn open(&self) -> Result<Response> {
        let request = Request::new(self)?;
        run(&request)
    }
}

/// A builder for a single-file open dialog.
pub fn dialog<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::SingleFile)
}

/// A builder for a multiple-file open dialog.
pub fn dialog_multiple<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::MultipleFiles)
}

/// A builder for a save dialog.
pub fn dialog_save<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::SaveFile)
}

/// A builder for a folder picker.
pub fn dialog_pick_folder<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::PickFolder)
}

/// Opens a single-file dialog. See the [crate docs](crate#filters) for the filter-list format.
pub fn open_file_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::SingleFile)
}

/// Opens a multiple-file dialog. See the [crate docs](crate#filters) for the filter-list format.
pub fn open_file_multiple_dialog(
    filter_list: Option<&str>,
    default_path: Option<&str>,
) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::MultipleFiles)
}

/// Opens a save dialog. See the [crate docs](crate#filters) for the filter-list format.
pub fn open_save_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::SaveFile)
}

/// Opens a folder picker.
pub fn open_pick_folder(default_path: Option<&str>) -> Result<Response> {
    open_dialog(None, default_path, DialogType::PickFolder)
}

/// Opens a dialog of any type.
pub fn open_dialog(
    filter_list: Option<&str>,
    default_path: Option<&str>,
    dialog_type: DialogType,
) -> Result<Response> {
    let mut builder = DialogBuilder::new(dialog_type);
    if let Some(filter_list) = filter_list {
        builder.filter(filter_list);
    }
    if let Some(default_path) = default_path {
        builder.default_path(default_path);
    }
    builder.open()
}

// ---------------------------------------------------------------------------------------------
// Filters

#[derive(Debug, Clone, PartialEq, Eq)]
struct Filter {
    name: String,
    spec: String,
}

/// Parses a filter-list string (`"png,jpg;pdf"`) into named filters, skipping empty ones.
fn parse_filter_list(list: &str) -> Vec<Filter> {
    list.split(';')
        .filter_map(normalize_spec)
        .map(|spec| Filter {
            name: generated_name(&spec),
            spec,
        })
        .collect()
}

/// Turns `" *.png, .JPG ,,"` into `"png,JPG"`: trims, strips `*.`/`.` prefixes and drops empty
/// and wildcard-only entries (every platform already offers an "All files" choice). Returns
/// `None` if nothing is left, because the C library must never get an empty spec.
fn normalize_spec(spec: &str) -> Option<String> {
    let extensions: Vec<&str> = spec
        .split([',', ';'])
        .map(|ext| {
            let ext = ext.trim();
            let ext = ext.strip_prefix('*').unwrap_or(ext);
            ext.strip_prefix('.').unwrap_or(ext).trim()
        })
        .filter(|ext| !ext.is_empty() && *ext != "*")
        .collect();
    if extensions.is_empty() {
        None
    } else {
        Some(extensions.join(","))
    }
}

/// A display name for a filter without one: `"png,jpg"` becomes `"PNG, JPG files"`.
fn generated_name(spec: &str) -> String {
    let extensions: Vec<String> = spec.split(',').map(str::to_uppercase).collect();
    format!("{} files", extensions.join(", "))
}

// ---------------------------------------------------------------------------------------------
// Talking to the C library

/// The builder's options as C strings. Everything that can fail on bad input fails here, before
/// any native code runs.
struct Request {
    dialog_type: DialogType,
    // `filter_items` points into `filter_strings`; CString data does not move when the Vec does.
    _filter_strings: Vec<(CString, CString)>,
    filter_items: Vec<nfdu8filteritem_t>,
    filter_count: nfdfiltersize_t,
    default_path: Option<CString>,
    default_name: Option<CString>,
    title: Option<CString>,
    #[cfg(feature = "raw-window-handle")]
    parent: Option<parent::Parent>,
}

impl Request {
    fn new(builder: &DialogBuilder<'_>) -> Result<Request> {
        let folder = builder.dialog_type == DialogType::PickFolder;
        let filter_strings = if folder {
            Vec::new()
        } else {
            builder
                .filters
                .iter()
                .map(|f| {
                    Ok((
                        CString::new(f.name.as_str())?,
                        CString::new(f.spec.as_str())?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?
        };
        let filter_items: Vec<_> = filter_strings
            .iter()
            .map(|(name, spec)| nfdu8filteritem_t {
                name: name.as_ptr(),
                spec: spec.as_ptr(),
            })
            .collect();
        let filter_count = nfdfiltersize_t::try_from(filter_items.len())
            .map_err(|_| NFDError::Error("too many filters".to_owned()))?;
        let default_path = builder
            .default_path
            .map(|path| {
                let path = std::str::from_utf8(path.as_os_str().as_encoded_bytes())?;
                Ok::<_, NFDError>(CString::new(path)?)
            })
            .transpose()?;
        let default_name = match builder.dialog_type {
            DialogType::SaveFile => builder.default_name.map(CString::new).transpose()?,
            _ => None,
        };
        let title = builder.title.map(CString::new).transpose()?;

        Ok(Request {
            dialog_type: builder.dialog_type,
            _filter_strings: filter_strings,
            filter_items,
            filter_count,
            default_path,
            default_name,
            title,
            #[cfg(feature = "raw-window-handle")]
            parent: builder
                .parent
                .clone()
                .transpose()
                .map_err(NFDError::Error)?,
        })
    }
}

fn opt_ptr(s: Option<&CStr>) -> *const c_char {
    s.map_or(std::ptr::null(), CStr::as_ptr)
}

/// Native dialogs are modal and the C library keeps global state, so only one may be open at a
/// time in the whole process.
static DIALOG_LOCK: Mutex<()> = Mutex::new(());

fn run(request: &Request) -> Result<Response> {
    platform::check_thread()?;
    let _lock = DIALOG_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let _session = Session::start()?;

    #[cfg(feature = "raw-window-handle")]
    let parent_window = request
        .parent
        .map(parent::Parent::to_native)
        .unwrap_or_default();
    #[cfg(not(feature = "raw-window-handle"))]
    let parent_window = nfdwindowhandle_t::default();

    let filter_count = request.filter_count;
    let filter_list = if request.filter_items.is_empty() {
        std::ptr::null()
    } else {
        request.filter_items.as_ptr()
    };

    let mut out_path: *mut nfdu8char_t = std::ptr::null_mut();
    let mut out_paths: *const nfdpathset_t = std::ptr::null();

    // SAFETY: NFD is initialised on this thread (`_session`) and no other dialog is running
    // (`_lock`). Every pointer in the argument structs points into `request`, which outlives the
    // call, or is null.
    let result = unsafe {
        match request.dialog_type {
            DialogType::SingleFile | DialogType::MultipleFiles => {
                let args = nfdopendialogu8args_t {
                    filterList: filter_list,
                    filterCount: filter_count,
                    defaultPath: opt_ptr(request.default_path.as_deref()),
                    parentWindow: parent_window,
                    title: opt_ptr(request.title.as_deref()),
                    ..Default::default()
                };
                if request.dialog_type == DialogType::SingleFile {
                    NFD_OpenDialogU8_With_Impl(NFD_INTERFACE_VERSION, &mut out_path, &args)
                } else {
                    NFD_OpenDialogMultipleU8_With_Impl(NFD_INTERFACE_VERSION, &mut out_paths, &args)
                }
            }
            DialogType::SaveFile => {
                let args = nfdsavedialogu8args_t {
                    filterList: filter_list,
                    filterCount: filter_count,
                    defaultPath: opt_ptr(request.default_path.as_deref()),
                    defaultName: opt_ptr(request.default_name.as_deref()),
                    parentWindow: parent_window,
                    title: opt_ptr(request.title.as_deref()),
                    ..Default::default()
                };
                NFD_SaveDialogU8_With_Impl(NFD_INTERFACE_VERSION, &mut out_path, &args)
            }
            DialogType::PickFolder => {
                let args = nfdpickfolderu8args_t {
                    defaultPath: opt_ptr(request.default_path.as_deref()),
                    parentWindow: parent_window,
                    title: opt_ptr(request.title.as_deref()),
                    ..Default::default()
                };
                NFD_PickFolderU8_With_Impl(NFD_INTERFACE_VERSION, &mut out_path, &args)
            }
        }
    };

    match result {
        // SAFETY: on NFD_OKAY the dialog filled in the output for its type, which we now own.
        NFD_OKAY if request.dialog_type == DialogType::MultipleFiles => {
            unsafe { take_path_set(out_paths) }.map(Response::OkayMultiple)
        }
        NFD_OKAY => unsafe { take_path(out_path) }.map(Response::Okay),
        NFD_CANCEL => Ok(Response::Cancel),
        // SAFETY: still inside the session.
        _ => Err(unsafe { take_error() }),
    }
}

/// Keeps NFD initialised on the current thread while alive.
struct Session(());

impl Session {
    fn start() -> Result<Session> {
        // SAFETY: always safe to call; balanced by `NFD_Quit` in `drop` on success.
        if unsafe { NFD_Init() } == NFD_OKAY {
            Ok(Session(()))
        } else {
            Err(unsafe { take_error() })
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: `NFD_Init` succeeded on this thread.
        unsafe { NFD_Quit() }
    }
}

/// Reads and clears NFD's last error.
unsafe fn take_error() -> NFDError {
    unsafe {
        let message = NFD_GetError();
        let message = if message.is_null() {
            "unknown error".to_owned()
        } else {
            CStr::from_ptr(message).to_string_lossy().into_owned()
        };
        NFD_ClearError();
        NFDError::Error(message)
    }
}

/// Copies and frees a path returned by a single-selection dialog.
unsafe fn take_path(path: *mut nfdu8char_t) -> Result<String> {
    if path.is_null() {
        return Err(NFDError::Error("the dialog returned no path".to_owned()));
    }
    unsafe {
        let copy = CStr::from_ptr(path).to_str().map(str::to_owned);
        NFD_FreePathU8(path);
        Ok(copy?)
    }
}

/// Copies and frees a path set returned by a multiple-selection dialog.
unsafe fn take_path_set(set: *const nfdpathset_t) -> Result<Vec<String>> {
    struct Guard(*const nfdpathset_t);
    impl Drop for Guard {
        fn drop(&mut self) {
            // SAFETY: the set came from a successful dialog and is freed exactly once.
            unsafe { NFD_PathSet_Free(self.0) }
        }
    }

    if set.is_null() {
        return Err(NFDError::Error("the dialog returned no paths".to_owned()));
    }
    let _guard = Guard(set);
    unsafe {
        let mut count: nfdpathsetsize_t = 0;
        if NFD_PathSet_GetCount(set, &mut count) != NFD_OKAY {
            return Err(take_error());
        }
        let mut paths = Vec::new();
        for index in 0..count {
            let mut path: *mut nfdu8char_t = std::ptr::null_mut();
            if NFD_PathSet_GetPathU8(set, index, &mut path) != NFD_OKAY {
                return Err(take_error());
            }
            let copy = CStr::from_ptr(path).to_str().map(str::to_owned);
            NFD_PathSet_FreePathU8(path);
            paths.push(copy?);
        }
        Ok(paths)
    }
}

// ---------------------------------------------------------------------------------------------
// Platform rules

#[cfg(target_os = "macos")]
mod platform {
    use crate::{NFDError, Result};

    pub(crate) fn is_main_thread() -> bool {
        unsafe extern "C" {
            safe fn pthread_main_np() -> std::os::raw::c_int;
        }
        pthread_main_np() == 1
    }

    pub(crate) fn check_thread() -> Result<()> {
        if is_main_thread() {
            Ok(())
        } else {
            Err(NFDError::Error(
                "macOS only allows file dialogs on the main thread".to_owned(),
            ))
        }
    }
}

#[cfg(all(feature = "gtk3", unix, not(target_os = "macos")))]
mod platform {
    use crate::{NFDError, Result};
    use std::sync::OnceLock;
    use std::thread::{self, ThreadId};

    static GTK_THREAD: OnceLock<ThreadId> = OnceLock::new();

    pub(crate) fn check_thread() -> Result<()> {
        let current = thread::current().id();
        if *GTK_THREAD.get_or_init(|| current) == current {
            Ok(())
        } else {
            Err(NFDError::Error(
                "GTK is not thread-safe: open every dialog from the thread that opened the first one"
                    .to_owned(),
            ))
        }
    }
}

#[cfg(not(any(target_os = "macos", all(feature = "gtk3", unix))))]
mod platform {
    pub(crate) fn check_thread() -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "raw-window-handle")]
mod parent {
    use crate::ffi::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    /// A parent window, stored as plain integers so that builders stay `Send` and `Sync`. The
    /// builder borrows the window, which keeps the handle valid.
    #[derive(Debug, Copy, Clone)]
    pub(crate) enum Parent {
        None,
        Win32(isize),
        AppKitView(usize),
        X11(std::os::raw::c_ulong),
    }

    impl Parent {
        pub(crate) fn from_window<W: HasWindowHandle + ?Sized>(
            window: &W,
        ) -> Result<Parent, String> {
            let handle = window
                .window_handle()
                .map_err(|err| format!("could not get the parent window handle: {err}"))?;
            Ok(match handle.as_raw() {
                RawWindowHandle::Win32(h) => Parent::Win32(h.hwnd.get()),
                RawWindowHandle::AppKit(h) => Parent::AppKitView(h.ns_view.as_ptr() as usize),
                RawWindowHandle::Xlib(h) => Parent::X11(h.window),
                RawWindowHandle::Xcb(h) => Parent::X11(h.window.get().into()),
                _ => Parent::None,
            })
        }

        /// Must be called on the thread that opens the dialog (the main thread on macOS).
        pub(crate) fn to_native(self) -> nfdwindowhandle_t {
            let (type_, handle) = match self {
                Parent::None => return nfdwindowhandle_t::default(),
                Parent::Win32(hwnd) => (NFD_WINDOW_HANDLE_TYPE_WINDOWS, hwnd as usize),
                Parent::AppKitView(view) => (NFD_WINDOW_HANDLE_TYPE_COCOA, window_of_view(view)),
                Parent::X11(window) => (NFD_WINDOW_HANDLE_TYPE_X11, window as usize),
            };
            if handle == 0 {
                return nfdwindowhandle_t::default();
            }
            nfdwindowhandle_t {
                type_,
                handle: handle as *mut std::os::raw::c_void,
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn window_of_view(view: usize) -> usize {
        unsafe extern "C" {
            fn nfd_rs_window_of_view(view: *mut std::os::raw::c_void) -> *mut std::os::raw::c_void;
        }
        // SAFETY: `view` is a live NSView (the builder borrows its window) and we are on the
        // main thread (checked before the dialog lock is taken).
        unsafe { nfd_rs_window_of_view(view as *mut _) as usize }
    }

    #[cfg(not(target_os = "macos"))]
    fn window_of_view(_view: usize) -> usize {
        0
    }
}

/// Compiles the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(name: &str, spec: &str) -> Filter {
        Filter {
            name: name.to_owned(),
            spec: spec.to_owned(),
        }
    }

    #[test]
    fn filter_list_is_split_into_named_filters() {
        assert_eq!(
            parse_filter_list("png,jpg;pdf"),
            vec![
                filter("PNG, JPG files", "png,jpg"),
                filter("PDF files", "pdf")
            ]
        );
        assert_eq!(parse_filter_list("jpg"), vec![filter("JPG files", "jpg")]);
    }

    #[test]
    fn filter_list_tolerates_spaces_dots_and_empty_entries() {
        assert_eq!(
            parse_filter_list(" *.png , .jpg ;; ,; tar.gz ;"),
            vec![
                filter("PNG, JPG files", "png,jpg"),
                filter("TAR.GZ files", "tar.gz")
            ]
        );
    }

    #[test]
    fn empty_and_wildcard_filters_are_dropped() {
        assert!(parse_filter_list("").is_empty());
        assert!(parse_filter_list(" ; , ;").is_empty());
        assert!(parse_filter_list("*;*.*").is_empty());
        assert_eq!(normalize_spec("*,png"), Some("png".to_owned()));
    }

    #[test]
    fn add_filter_keeps_the_name_and_normalizes_extensions() {
        let mut builder = dialog();
        builder
            .add_filter("Images", "png, *.jpg")
            .add_filter("", "txt")
            .add_filter("Nothing", " , ")
            .filter("pdf");
        assert_eq!(
            builder.filters,
            vec![
                filter("Images", "png,jpg"),
                filter("TXT files", "txt"),
                filter("PDF files", "pdf"),
            ]
        );
    }

    #[test]
    fn builder_can_be_configured_in_steps() {
        let dir = std::path::PathBuf::from("/tmp");
        let mut builder = dialog_save();
        builder.default_path(&dir);
        builder.default_name("a.txt");
        builder.title("Save");
        assert_eq!(builder.default_path, Some(dir.as_path()));
        assert_eq!(builder.default_name, Some("a.txt"));
        assert_eq!(builder.title, Some("Save"));
        // `&str` paths still work, as in 0.0.x.
        builder.default_path("/");
        assert_eq!(builder.default_path, Some(Path::new("/")));
    }

    #[test]
    fn request_rejects_nul_bytes_before_any_native_call() {
        let err = open_dialog(Some("png\0"), None, DialogType::SingleFile).unwrap_err();
        assert!(matches!(err, NFDError::NulError(_)), "{err:?}");
        let err = open_pick_folder(Some("/tmp\0")).unwrap_err();
        assert!(matches!(err, NFDError::NulError(_)), "{err:?}");
        let err = dialog().title("a\0b").open().unwrap_err();
        assert!(matches!(err, NFDError::NulError(_)), "{err:?}");
    }

    #[cfg(unix)]
    #[test]
    fn request_rejects_non_utf8_paths_before_any_native_call() {
        use std::os::unix::ffi::OsStrExt;
        let path = Path::new(std::ffi::OsStr::from_bytes(b"/tmp/\xff"));
        let err = dialog().default_path(path).open().unwrap_err();
        assert!(matches!(err, NFDError::Utf8Error(_)), "{err:?}");
    }

    #[test]
    fn request_builds_c_filter_items() {
        let mut builder = dialog();
        builder.filter("png,jpg;pdf");
        let request = Request::new(&builder).unwrap();
        assert_eq!(request.filter_items.len(), 2);
        let spec = unsafe { CStr::from_ptr(request.filter_items[0].spec) };
        let name = unsafe { CStr::from_ptr(request.filter_items[1].name) };
        assert_eq!(spec.to_str(), Ok("png,jpg"));
        assert_eq!(name.to_str(), Ok("PDF files"));
    }

    #[test]
    fn folder_and_open_requests_drop_irrelevant_options() {
        let mut builder = dialog_pick_folder();
        builder.filter("png").default_name("x");
        let request = Request::new(&builder).unwrap();
        assert!(request.filter_items.is_empty());
        assert!(request.default_name.is_none());
    }

    #[test]
    fn builders_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DialogBuilder<'static>>();
        assert_send_sync::<Response>();
        assert_send_sync::<NFDError>();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_refuses_dialogs_off_the_main_thread() {
        let (main, check) =
            std::thread::spawn(|| (platform::is_main_thread(), platform::check_thread()))
                .join()
                .unwrap();
        assert!(!main);
        assert!(check.is_err());
    }

    #[cfg(all(feature = "gtk3", unix, not(target_os = "macos")))]
    #[test]
    fn gtk_is_pinned_to_one_thread() {
        let first = std::thread::spawn(platform::check_thread).join().unwrap();
        let second = std::thread::spawn(platform::check_thread).join().unwrap();
        assert!(first.is_ok());
        assert!(second.is_err());
    }
}
