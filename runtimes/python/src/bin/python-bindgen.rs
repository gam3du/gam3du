#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    unused_crate_dependencies,
    clippy::print_stdout,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

use gam3du_framework::api::ApiDescriptor;
use runtime_python::{bindgen, Config};
use std::io::BufWriter;

fn main() {
    // let move_forward = FunctionDescriptor {
    //     name: Identifier("move forward".into()),
    //     caption: RichText(
    //         "Makes the robot move to the next tile in its current orientation".into(),
    //     ),
    //     description: RichText(
    //         "Makes the robot move to the next tile in its current orientation".into(),
    //     ),
    //     parameters: Vec::new(),
    //     returns: None,
    // };

    // let turn_left = FunctionDescriptor {
    //     name: Identifier("turn left".into()),
    //     caption: RichText("Turns the robot 45° in a counter-clockwise direction".into()),
    //     description: RichText("Turns the robot 45° in a counter-clockwise direction".into()),
    //     parameters: Vec::new(),
    //     returns: None,
    // };

    // let turn_right = FunctionDescriptor {
    //     name: Identifier("turn right".into()),
    //     caption: RichText("Turns the robot 45° in a clockwise direction".into()),
    //     description: RichText("Turns the robot 45° in a clockwise direction".into()),
    //     parameters: Vec::new(),
    //     returns: None,
    // };

    // let api = Api {
    //     name: Identifier("robot".into()),
    //     caption: RichText(
    //         "A simple robot that can be moved across a 2D-plane and draw lines".into(),
    //     ),
    //     description: RichText(
    //         "Once upon a time there was a lonely robot with the serial number `#C0D1E`. …".into(),
    //     ),
    //     functions: vec![move_forward, turn_left, turn_right],
    // };

    // let json_string = serde_json::to_string_pretty(&api).unwrap();
    // println!("{json_string}");

    // let json_string = json5::to_string(&api).unwrap();

    // TODO make the engine a command line parameter
    let api_json = std::fs::read_to_string("engines/robot/api.json").unwrap();
    let api: ApiDescriptor = serde_json::from_str(&api_json).unwrap();
    println!("{api:#?}");

    // Generate sync api
    let out_file = std::fs::File::create("python/robot_api.py").unwrap();
    let mut out = BufWriter::new(out_file);
    bindgen::generate(&mut out, &api, &Config { sync: true }).unwrap();

    let out_file = std::fs::File::create("python/robot_api_async.py").unwrap();
    let mut out = BufWriter::new(out_file);
    bindgen::generate(&mut out, &api, &Config { sync: false }).unwrap();

    // println!("{out}");
}
