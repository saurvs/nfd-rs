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

//! Compiles the bundled nativefiledialog-extended sources for the target platform.

use std::env;
use std::path::PathBuf;
use std::process::Command;

const NFD_DIR: &str = "nativefiledialog-extended";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=csrc");
    println!("cargo:rerun-if-changed={NFD_DIR}/src");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    // docs.rs only runs rustdoc, which needs neither the C library nor its system dependencies.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let src = manifest_dir.join(NFD_DIR).join("src");
    let include = src.join("include");
    if !include.join("nfd.h").is_file() {
        panic!(
            "nfd: {} is missing. If you are building from a git checkout, run \
             `git submodule update --init`.",
            include.display()
        );
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("set by cargo");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let gtk3 = env::var_os("CARGO_FEATURE_GTK3").is_some();
    let raw_window_handle = env::var_os("CARGO_FEATURE_RAW_WINDOW_HANDLE").is_some();

    let mut nfd = cc::Build::new();
    nfd.include(&include);

    // Our own small C helpers (see csrc/), always compiled as C.
    let mut glue = cc::Build::new();
    glue.include(&include).file("csrc/nfd_rs_glue.c");

    // System libraries are emitted after the static libraries that use them, because GNU ld and
    // MinGW resolve symbols in command-line order.
    let mut system_libs: Vec<&str> = Vec::new();
    let mut pkg_config_package = None;

    match target_os.as_str() {
        "macos" => {
            // Without this, cc compiles for the installed SDK's macOS version, which is usually
            // newer than the version rustc targets.
            let deployment_target = macos_deployment_target();
            nfd.env("MACOSX_DEPLOYMENT_TARGET", &deployment_target);
            glue.env("MACOSX_DEPLOYMENT_TARGET", &deployment_target);
            // allowedContentTypes (macOS 11+) replaces allowedFileTypes (deprecated in 12).
            let major: u32 = deployment_target
                .split('.')
                .next()
                .and_then(|major| major.parse().ok())
                .unwrap_or(10);
            let content_types = major >= 11;
            nfd.file(src.join("nfd_cocoa.m")).define(
                "NFD_MACOS_ALLOWEDCONTENTTYPES",
                if content_types { "1" } else { "0" },
            );
            system_libs.push("framework=AppKit");
            if content_types {
                system_libs.push("framework=UniformTypeIdentifiers");
            }
            if raw_window_handle {
                glue.file("csrc/nfd_rs_cocoa.m");
            }
        }
        "windows" => {
            nfd.cpp(true).std("c++17").file(src.join("nfd_win.cpp"));
            if target_env != "msvc" {
                // The library needs neither exceptions, RTTI nor libstdc++.
                nfd.flag("-fno-exceptions")
                    .flag("-fno-rtti")
                    .cpp_link_stdlib(None);
            }
            system_libs.extend(["ole32", "shell32", "uuid"]);
        }
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" => {
            nfd.cpp(true)
                .std("c++17")
                .flag("-fno-exceptions")
                .flag("-fno-rtti")
                .cpp_link_stdlib(None)
                // Lets callers parent dialogs to X11 windows; needs no extra libraries.
                .define("NFD_X11", None);

            let (source, package, hint) = if gtk3 {
                (
                    "nfd_gtk.cpp",
                    "gtk+-3.0",
                    "Debian/Ubuntu: `libgtk-3-dev`, Fedora: `gtk3-devel`, Arch: `gtk3`",
                )
            } else {
                (
                    "nfd_portal.cpp",
                    "dbus-1",
                    "Debian/Ubuntu: `libdbus-1-dev`, Fedora: `dbus-devel`, Arch: `dbus`",
                )
            };
            let library = pkg_config::Config::new()
                .cargo_metadata(false)
                .probe(package)
                .unwrap_or_else(|err| {
                    panic!(
                        "nfd: could not find `{package}` with pkg-config. Install pkg-config and \
                         the development files ({hint}).\n{err}"
                    )
                });
            nfd.includes(&library.include_paths).file(src.join(source));
            pkg_config_package = Some(package);
            if !gtk3 {
                nfd.define("NFD_PORTAL", None);
                glue.define("NFD_PORTAL", None);
            }
        }
        other => panic!(
            "nfd: unsupported target OS `{other}`. Supported: macOS, Windows, Linux and the BSDs."
        ),
    }

    nfd.compile("nfd");
    glue.compile("nfd_rs_glue");

    for lib in system_libs {
        println!("cargo:rustc-link-lib={lib}");
    }
    if let Some(package) = pkg_config_package {
        // Probe again, this time letting pkg-config emit the link flags.
        pkg_config::Config::new()
            .probe(package)
            .expect("pkg-config succeeded above");
    }
}

/// The macOS version to compile the C code for: `MACOSX_DEPLOYMENT_TARGET` if set, otherwise
/// whatever rustc targets by default.
fn macos_deployment_target() -> String {
    if let Ok(target) = env::var("MACOSX_DEPLOYMENT_TARGET") {
        if !target.trim().is_empty() {
            return target.trim().to_owned();
        }
    }
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let target = env::var("TARGET").expect("set by cargo");
    let printed = Command::new(rustc)
        .args(["--print", "deployment-target", "--target", &target])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok());
    if let Some(version) = printed
        .as_deref()
        .and_then(|out| out.trim().strip_prefix("MACOSX_DEPLOYMENT_TARGET="))
    {
        return version.to_owned();
    }
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") {
        "11.0".to_owned()
    } else {
        "10.12".to_owned()
    }
}
