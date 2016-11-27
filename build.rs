// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate pkg_config;

use std::process::Command;
use std::env;
use std::fs;
use std::fs::File;

// Needs the libev-dev package

// Return true if file1 doesn't exist or is older than file2.
fn doesnt_exists_or_older(file1: &str, file2: &str) -> bool {
    let f1 = File::open(file1);
    if f1.is_err() {
        return true;
    }

    let f2 = File::open(file2);
    if f2.is_err() {
        panic!(format!("The file {} should exist!", file2));
    }

    let time1 = f1.unwrap().metadata().unwrap().modified().unwrap();
    let time2 = f2.unwrap().metadata().unwrap().modified().unwrap();
    time1 < time2
}

macro_rules! doesnt_exists_or_older (
    ($f1:expr, $f2:expr) => (
        doesnt_exists_or_older(&format!("libpagekite/{}", $f1), &format!("libpagekite/{}", $f2))
    )
);

fn build(output: &str) {
    // If configure doesn't exist or is older than configure.ac run autogen.sh
    if doesnt_exists_or_older!("configure", "configure.ac") {
        let exit_code = Command::new("./autogen.sh")
            .current_dir("libpagekite")
            .status()
            .unwrap();

        if !exit_code.success() {
            panic!("Failed to run libpagekite/autogen.sh");
        }
    }

    // If Makefile doesn't exist or is older than configure run configure
    if doesnt_exists_or_older!("Makefile", "configure") {
        let exit_code = Command::new("./configure")
            .env("CFLAGS", "-fPIC") // Needed to build the static library as PIC.
            .arg(format!("--prefix={}", output))
            .arg(format!("--without-java"))
            //.arg(format!("--host={}", env::var("TARGET_TRIPLE").unwrap_or(env::var("TARGET").unwrap())))
            .current_dir("libpagekite")
            .status().unwrap();

        if !exit_code.success() {
            panic!("Failed to run libpagekite/configure");
        }
    }

    // We don't pass -j parameter to `make` because the build fails with this option.
    let exit_code = Command::new("make")
        .arg("install")
        .current_dir("libpagekite")
        .status()
        .unwrap();

    if !exit_code.success() {
        panic!(format!("Failure running `make -C libpagekite`"));
    }
}

#[allow(unused_must_use)]
fn main() {
    let libpagekite_build_dir = format!("{}/{}", env::var("OUT_DIR").unwrap(), "libpagekite");

    fs::create_dir(libpagekite_build_dir.clone());

    build(&libpagekite_build_dir);

    pkg_config::probe_library("libssl").unwrap();

    // No pkg-config support for libev unfortunately.
    println!("cargo:rustc-link-lib=ev");

    println!("cargo:rustc-link-search=native={}/lib", libpagekite_build_dir);
    println!("cargo:rustc-link-lib=dylib=pagekite");
}
