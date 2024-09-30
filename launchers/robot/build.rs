#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    reason = "TODO remove before release"
)]

use gam3du_framework_common::api::ApiDescriptor;
use runtime_python_bindgen::PyIdentifier;
use std::io::{BufWriter, Write};

const API_DESCRIPTOR: &str = "control.api.json";

fn main() {
    println!("cargo::rerun-if-changed={API_DESCRIPTOR}");

    // TODO make the engine a command line parameter
    let api_json = std::fs::read_to_string(API_DESCRIPTOR).unwrap();
    let api: ApiDescriptor = serde_json::from_str(&api_json).unwrap();

    let api_bindings = format!("python/control/{}_api.py", api.name.file());

    let out_file = std::fs::File::create(api_bindings).unwrap();
    let mut out = BufWriter::new(out_file);
    writeln!(
        out,
        "# This file has been generated automatically and shall not be edited by hand!"
    )
    .unwrap();
    writeln!(out, "# generator: {}", file!()).unwrap();
    writeln!(out, "# api descriptor: {API_DESCRIPTOR}").unwrap();
    writeln!(out).unwrap();
    runtime_python_bindgen::generate(&mut out, &api).unwrap();
}
