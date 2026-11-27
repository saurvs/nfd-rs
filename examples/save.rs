//! Opens a save dialog that suggests a file name.

use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from);

    let mut dialog = nfd::dialog_save();
    dialog
        .title("Save notes")
        .add_filter("Text files", "txt,md")
        .default_name("notes.txt");
    if let Some(home) = &home {
        dialog.default_path(home);
    }

    match dialog.open()? {
        Response::Okay(path) => println!("Save to {path:?}"),
        Response::Cancel => println!("User canceled"),
        Response::OkayMultiple(_) => unreachable!("save dialogs return one path"),
    }
    Ok(())
}
