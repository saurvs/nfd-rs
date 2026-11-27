# Changelog

## 0.1.0 (2026-11-27)

The first release since 0.0.4 (October 2016). Thank you to everyone who reported issues and
sent fixes while the crate went unreleased, and sorry for the long wait.

### Changed

- The bundled C library is now [Native File Dialog Extended](https://github.com/btzy/nativefiledialog-extended)
  1.4.1, the maintained successor of the 2016 nativefiledialog snapshot used before
  ([#22](https://github.com/sauravsc/nfd-rs/issues/22); supersedes
  [#21](https://github.com/sauravsc/nfd-rs/pull/21) by @repi and
  [#25](https://github.com/sauravsc/nfd-rs/pull/25) by @lockieluke).
- Linux and the BSDs use the XDG desktop portal by default, so GTK is no longer needed to build or
  run ([#19](https://github.com/sauravsc/nfd-rs/issues/19)). Building needs `pkg-config` and the
  D-Bus development files instead. The new `gtk3` feature selects the GTK 3 backend.
- Windows builds with both MSVC and MinGW-w64 toolchains
  ([#18](https://github.com/sauravsc/nfd-rs/issues/18), [#20](https://github.com/sauravsc/nfd-rs/issues/20)).
- The build script uses `cc` instead of the deprecated `gcc` crate, which also removes the
  future-incompatibility warning ([#28](https://github.com/sauravsc/nfd-rs/pull/28) by @ctrlcctrlv).
- The `error` and `ffi` modules are public ([#10](https://github.com/sauravsc/nfd-rs/pull/10) by
  @ashkitten), and `NFDError` is re-exported from the crate root. `ffi` now contains raw bindings to
  Native File Dialog Extended.
- Builder methods take `&mut self` rather than `&'a mut self`, so a builder can be configured over
  several statements and then opened.
- `DialogBuilder::default_path` accepts any `AsRef<Path>` (`&str` still works).
- Paths are returned exactly as the platform reports them. A path that is not valid UTF-8 returns
  `Err(NFDError::Utf8Error)` instead of a lossily converted string that names no real file.
- `NFDError` has descriptive `Display` messages, implements `Error::source`, and derives `Clone`,
  `PartialEq` and `Eq`. `Response` and `DialogType` derive `Debug`, `Clone`, `PartialEq`, `Eq` and
  `Hash`.
- Edition 2024; the minimum supported Rust version is 1.85.

### Added

- `DialogBuilder::add_filter(name, extensions)` for filters with your own display names.
- `DialogBuilder::title` and `DialogBuilder::default_name` (suggested file name for save dialogs).
- `DialogBuilder::parent` (feature `raw-window-handle`) to keep a dialog in front of your window
  ([#16](https://github.com/sauravsc/nfd-rs/issues/16)).
- `dialog_pick_folder()` builder shortcut.

### Fixed

- Paths returned by single-selection dialogs were never freed.
- Only one dialog is open at a time across the process, and dialogs are refused (with an error)
  where the platform would crash: off the main thread on macOS, and on a second thread with GTK
  ([#27](https://github.com/sauravsc/nfd-rs/issues/27)).
- Windows: the `SHCreateItemFromParsingName` link error from 0.0.4
  ([#14](https://github.com/sauravsc/nfd-rs/issues/14), [#18](https://github.com/sauravsc/nfd-rs/issues/18)),
  fixed on master by @raphlinus in [#15](https://github.com/sauravsc/nfd-rs/pull/15), is now released
  ([#17](https://github.com/sauravsc/nfd-rs/issues/17)).
- The default Linux backend no longer prints "GtkDialog mapped without a transient parent"
  ([#8](https://github.com/sauravsc/nfd-rs/issues/8)).
- Empty or wildcard-only filters (`""`, `";"`, `"*"`) are ignored instead of reaching the C
  library.

### Migrating from 0.0.4

- **Linux build dependencies**: install `pkg-config` and `libdbus-1-dev` (Debian/Ubuntu),
  `dbus-devel` (Fedora) or `dbus` (Arch). To keep using GTK, enable the `gtk3` feature
  (`nfd = { version = "0.1", features = ["gtk3"] }`) and keep `libgtk-3-dev` installed. The portal
  backend needs a running desktop portal at runtime, which common desktops provide.
- **Threads**: on macOS, open dialogs from the main thread. With `gtk3`, open them all from one
  thread. Calls that break these rules now return an error instead of crashing.
- **Non-UTF-8 paths** now produce `NFDError::Utf8Error`.
- **Error messages**: `NFDError`'s variants are unchanged, but its `Display` text is new.
- **Filter names**: filters from filter-list strings get names such as "PNG, JPG files". Use
  `add_filter` to choose your own.
- **Windows default folder**: as is usual on Windows, `default_path` is only used when the system
  has no recently used folder for your application.
- **Git dependencies**: the submodule moved from `nativefiledialog` to `nativefiledialog-extended`.
  Cargo fetches it automatically; in an existing checkout run `git submodule update --init`.
- Old example code such as `panic!(e)` no longer compiles on edition 2021 and later; use
  `panic!("{e}")` or `?`.

## 0.0.4 (2016-10-19)

- Folder picker (`open_pick_folder`), save dialog fix, MinGW support.

## 0.0.3 and earlier

- Initial releases.
