//! Opens a multiple-file dialog.

use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    match nfd::dialog_multiple().filter("png,jpg;pdf").open()? {
        Response::OkayMultiple(files) => println!("Files = {files:?}"),
        Response::Cancel => println!("User canceled"),
        Response::Okay(_) => unreachable!("multiple-file dialogs return a list"),
    }
    Ok(())
}
