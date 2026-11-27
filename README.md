# nfd-rs

[![crates.io](https://img.shields.io/crates/v/nfd.svg)](https://crates.io/crates/nfd)
[![docs.rs](https://img.shields.io/docsrs/nfd)](https://docs.rs/nfd)
[![CI](https://github.com/sauravsc/nfd-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/sauravsc/nfd-rs/actions/workflows/ci.yml)
[![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-informational.svg)](#minimum-supported-rust-version)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

`nfd` opens the platform's own file dialogs from Rust: open one or more files, save a file, or pick
a folder on Windows, macOS, Linux and the BSDs. It wraps the bundled
[Native File Dialog Extended](https://github.com/btzy/nativefiledialog-extended) C library
(1.4.1), so the dialogs are the native ones: `IFileDialog` on Windows, `NSOpenPanel`/`NSSavePanel`
on macOS, and the XDG desktop portal (or GTK 3) on Linux.

## Usage

```toml
[dependencies]
nfd = "0.1"
```

Open a single file:

```rust,no_run
use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    match nfd::open_file_dialog(None, None)? {
        Response::Okay(file_path) => println!("File path = {file_path:?}"),
        Response::OkayMultiple(files) => println!("Files = {files:?}"),
        Response::Cancel => println!("User canceled"),
    }
    Ok(())
}
```

Open several images with the builder:

```rust,no_run
use nfd::Response;

fn main() -> Result<(), nfd::NFDError> {
    let result = nfd::dialog_multiple()
        .title("Choose photos")
        .add_filter("Images", "png,jpg,jpeg")
        .open()?;

    match result {
        Response::OkayMultiple(files) => println!("Files = {files:?}"),
        Response::Cancel => println!("User canceled"),
        Response::Okay(_) => unreachable!(),
    }
    Ok(())
}
```

Save a file, suggesting a folder and a name:

```rust,no_run
fn main() -> Result<(), nfd::NFDError> {
    let response = nfd::dialog_save()
        .add_filter("Text", "txt")
        .default_path("/home/me/Documents")
        .default_name("notes.txt")
        .open()?;
    println!("{response:?}");
    Ok(())
}
```

Pick a folder with `nfd::open_pick_folder(None)` or `nfd::dialog_pick_folder().open()`.

Filters can also be given as filter-list strings, as in earlier releases: commas separate
extensions and semicolons separate filters, so `"png,jpg;pdf"` offers a PNG/JPEG filter and a PDF
filter. More examples are in [`examples/`](examples).

## Platform requirements

| Platform | Build requirements | Notes |
| --- | --- | --- |
| Windows | MSVC or MinGW-w64 (`x86_64-pc-windows-gnu`) toolchain | Links `ole32`, `shell32` and `uuid`. |
| macOS | Xcode or the Command Line Tools | Dialogs must be opened from the main thread. |
| Linux, BSD | `pkg-config` and the D-Bus development files: `libdbus-1-dev` (Debian/Ubuntu), `dbus-devel` (Fedora), `dbus` (Arch) | At runtime a desktop portal with a file chooser must be running (GNOME, KDE and most desktops ship one; otherwise install `xdg-desktop-portal-gtk`). |
| Linux, BSD with `gtk3` | `pkg-config` and `libgtk-3-dev` (Debian/Ubuntu), `gtk3-devel` (Fedora), `gtk3` (Arch) | Dialogs must always be opened from the same thread. |

If you build from a git checkout, fetch the bundled C library first:
`git submodule update --init` (or clone with `--recursive`). Cargo does this automatically for git
dependencies.

On macOS the C code is compiled for the same deployment target as your Rust code
(`MACOSX_DEPLOYMENT_TARGET`, or rustc's default). With a target of 11.0 or later, filters use
content types; extensions specific to your application may need to be declared in your app's
`Info.plist` ([details](https://github.com/btzy/nativefiledialog-extended#macos)).

## Cargo features

- `gtk3`: on Linux and the BSDs, use GTK 3 instead of the XDG desktop portal.
- `raw-window-handle`: adds `DialogBuilder::parent(&window)`, which ties the dialog to your window
  so that it opens in front of it. It accepts anything implementing
  [`raw_window_handle::HasWindowHandle`](https://docs.rs/raw-window-handle/0.6) (winit, SDL2,
  GLFW and most windowing crates). Wayland windows are accepted, but the dialog is not parented to
  them.

## Threads

Dialogs are modal, so `nfd` opens one at a time; calls from other threads wait. macOS only allows
dialogs on the main thread and returns an error elsewhere. With the `gtk3` backend, every dialog
must come from the thread that opened the first one. Windows and the default Linux portal backend
work from any thread.

## Minimum supported Rust version

Rust 1.85 (edition 2024). Raising it is a minor-version change.

## Upgrading from 0.0.4

See [CHANGELOG.md](CHANGELOG.md#migrating-from-004). In short: the public API is unchanged
apart from some additions, but Linux builds now need the D-Bus development files instead of GTK
(or enable the `gtk3` feature), and paths that are not valid UTF-8 return an error instead of a
lossily converted string.

## Alternatives

If you would rather avoid a C dependency, or need async dialogs or WebAssembly support,
[`rfd`](https://crates.io/crates/rfd) is a good choice.

## Screenshot

![Cocoa on El Capitan](screenshots/cocoa_el_capitan.png?raw=true)

## License

The Rust code is MIT licensed (see [LICENSE.md](LICENSE.md)). The bundled Native File Dialog
Extended library is under the zlib license
([nativefiledialog-extended/LICENSE](https://github.com/btzy/nativefiledialog-extended/blob/master/LICENSE)).
