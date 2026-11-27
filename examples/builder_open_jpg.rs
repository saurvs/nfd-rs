//! Opens a single-file dialog that only shows JPEG files.

use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    let result = nfd::dialog()
        .add_filter("JPEG images", "jpg,jpeg")
        .title("Pick a photo")
        .open()?;

    match result {
        Response::Okay(file_path) => println!("File path = {file_path:?}"),
        Response::Cancel => println!("User canceled"),
        Response::OkayMultiple(_) => unreachable!("single-file dialogs return one path"),
    }
    Ok(())
}
