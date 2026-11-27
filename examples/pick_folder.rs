//! Opens a folder picker.

use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    match nfd::open_pick_folder(None)? {
        Response::Okay(folder) => println!("Folder = {folder:?}"),
        Response::Cancel => println!("User canceled"),
        Response::OkayMultiple(_) => unreachable!("folder pickers return one path"),
    }
    Ok(())
}
