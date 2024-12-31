#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    reason = "TODO remove before release"
)]

use gam3du_framework_common::api::ApiDescriptor;
use runtime_python_bindgen::{Config, PyIdentifier};
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

const API_DESCRIPTOR: &str = "control.api.json";

fn main() {
    println!("cargo::rerun-if-changed={API_DESCRIPTOR}");

    // TODO make the engine a command line parameter
    let api_json = std::fs::read_to_string(API_DESCRIPTOR).unwrap();
    let api: ApiDescriptor = serde_json::from_str(&api_json).unwrap();
    let api_name = api.name.file();

    // Generate sync api
    {
        let api_bindings = format!("python/control/{api_name}_api.py");
        let mut out = new_out_file(api_bindings);
        write_header(&mut out);
        runtime_python_bindgen::generate(&mut out, &api, &Config { sync: true }).unwrap();
    }

    // Generate async api
    {
        let api_bindings = format!("python/control/{api_name}_api_async.py");
        let mut out = new_out_file(api_bindings);
        write_header(&mut out);
        runtime_python_bindgen::generate(&mut out, &api, &Config { sync: false }).unwrap();
    }
}

fn new_out_file(api_bindings: String) -> BufWriter<File> {
    let out_file = File::create(api_bindings).unwrap();
    BufWriter::new(out_file)
}

fn write_header(out: &mut BufWriter<File>) {
    writeln!(
        out,
        "# This file has been generated automatically and shall not be edited by hand!"
    )
    .unwrap();
    writeln!(out, "# generator: applications/robot/build.rs").unwrap();
    writeln!(out, "# api descriptor: {API_DESCRIPTOR}").unwrap();
    writeln!(out).unwrap();
}
