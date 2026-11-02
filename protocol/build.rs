//! Provides the `wifi_config` module (audit Phase 2E, PIN-TPL-015).
//!
//! The local, git-ignored `src/wifi_config.rs` holds Wi-Fi credentials and the
//! development token. When it is absent (a fresh clone, CI), the placeholder values of
//! `src/wifi_config.example.rs` are used instead, with a build warning, so every crate
//! that depends on the protocol still builds. Before 2E a fresh clone failed to build
//! until the file was created by hand.

use std::{env, fs, path::PathBuf};

fn main() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by Cargo")).join("src");
    let local = src.join("wifi_config.rs");
    let example = src.join("wifi_config.example.rs");
    println!("cargo:rerun-if-changed={}", local.display());
    println!("cargo:rerun-if-changed={}", example.display());

    let source = if local.exists() {
        local
    } else {
        println!(
            "cargo:warning=protocol/src/wifi_config.rs not found: using the placeholders from wifi_config.example.rs (the Wi-Fi transport will not connect). Copy the example and fill it in to use Wi-Fi."
        );
        example
    };
    let text = fs::read_to_string(&source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()));
    // `include!` cannot contain inner doc comments (`//!`).
    let body: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//!"))
        .map(|line| format!("{line}\n"))
        .collect();
    let out = PathBuf::from(env::var("OUT_DIR").expect("set by Cargo")).join("wifi_config.rs");
    fs::write(&out, body).unwrap_or_else(|error| panic!("cannot write {}: {error}", out.display()));
}
