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

mod ffi;
mod error;

use ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use error::NFDError;

/// Result of opening a file dialog
pub enum Response {
    /// User pressed okay. `String` is the file path selected
    Okay(String),
    /// User pressed okay on mupliple selections. Result contains a Vec of all the files
    OkayMultiple(Vec<String>),
    /// User pressed cancel
    Cancel,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DialogType {
    SingleFile,
    MultipleFiles,
    SaveFile,
    PickFolder,
}

pub struct DialogBuilder<'a> {
    filter: Option<&'a str>,
    default_path: Option<&'a str>,
    dialog_type: DialogType,
}

impl<'a> DialogBuilder<'a> {

    pub fn new(dialog_type: DialogType) -> Self {
        DialogBuilder {
            filter: None,
            default_path: None,
            dialog_type: dialog_type,
        }
    }

    pub fn filter(&'a mut self, filter: &'a str) -> &mut DialogBuilder {
        self.filter = Some(filter);
        self
    }

    pub fn default_path(&'a mut self, path: &'a str) -> &mut DialogBuilder {
        self.default_path = Some(path);
        self
    }

    pub fn open(&self) -> Result<Response> {
        open_dialog(self.filter, self.default_path, self.dialog_type.clone())
    }
}

pub fn dialog<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::SingleFile)
}

pub fn dialog_multiple<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::MultipleFiles)
}

pub fn dialog_save<'a>() -> DialogBuilder<'a> {
    DialogBuilder::new(DialogType::SaveFile)
}

pub type Result<T> = std::result::Result<T, NFDError>;

/// Open single file dialog
pub fn open_file_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::SingleFile)
}

/// Open mulitple file dialog
pub fn open_file_multiple_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::MultipleFiles)
}

/// Open save dialog
pub fn open_save_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> Result<Response> {
    open_dialog(filter_list, default_path, DialogType::SaveFile)
}

/// Open save dialog
pub fn open_pick_folder(default_path: Option<&str>) -> Result<Response> {
    open_dialog(None, default_path, DialogType::PickFolder)
}

pub fn open_dialog(filter_list: Option<&str>, default_path: Option<&str>, dialog_type: DialogType) -> Result<Response> {
    let result;
    let filter_list_cstring;
    let default_path_cstring;

    let filter_list_ptr = match filter_list {
        Some(fl_str) => {
            filter_list_cstring = CString::new(fl_str)?;
            filter_list_cstring.as_ptr()
        }
        None => std::ptr::null()
    };

    let default_path_ptr = match default_path {
        Some(dp_str) => {
            default_path_cstring = CString::new(dp_str)?;
            default_path_cstring.as_ptr()
        }
        None => std::ptr::null()
    };

    let mut out_path: *mut c_char = std::ptr::null_mut();
    let ptr_out_path = &mut out_path as *mut *mut c_char;

    let mut out_multiple = nfdpathset_t::default();
    let ptr_out_multiple = &mut out_multiple as *mut nfdpathset_t;

    unsafe {
        result = match dialog_type {
            DialogType::SingleFile => {
                NFD_OpenDialog(filter_list_ptr, default_path_ptr, ptr_out_path)
            },

            DialogType::MultipleFiles => {
                NFD_OpenDialogMultiple(filter_list_ptr, default_path_ptr, ptr_out_multiple)
            },

            DialogType::SaveFile => {
                NFD_SaveDialog(filter_list_ptr, default_path_ptr, ptr_out_path)
            },

            DialogType::PickFolder => {
                NFD_PickFolder(default_path_ptr, ptr_out_path)
            },
        };

        match result {
            nfdresult_t::NFD_OKAY =>{
                if dialog_type == DialogType::MultipleFiles {
                    let count = NFD_PathSet_GetCount(&out_multiple);
                    let mut res = Vec::with_capacity(count);
                    for i in 0..count {
                        let path = CStr::from_ptr(NFD_PathSet_GetPath(&out_multiple, i)).to_string_lossy().into_owned();
                        res.push(path)

                    }

                    NFD_PathSet_Free(ptr_out_multiple);

                    Ok(Response::OkayMultiple(res))
                } else {
                    Ok(Response::Okay(CStr::from_ptr(out_path).to_string_lossy().into_owned()))
                }
            },

            nfdresult_t::NFD_CANCEL => Ok(Response::Cancel),
            nfdresult_t::NFD_ERROR => Err(NFDError::Error(CStr::from_ptr(NFD_GetError()).to_string_lossy().into_owned())),
        }
    }
}
