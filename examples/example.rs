//! Opens a single-file dialog with no filter.

use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    match nfd::open_file_dialog(None, None)? {
        Response::Okay(file_path) => println!("File path = {file_path:?}"),
        Response::OkayMultiple(files) => println!("Files = {files:?}"),
        Response::Cancel => println!("User canceled"),
    }
    Ok(())
}
