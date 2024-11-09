//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(clippy::print_stdout, reason = "just a demo")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = window)]
    fn callback(s: &str) -> u32;
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    let mut i = 0;
    loop {
        let callback = callback(&format!("Hello, {name}!"));
        println!("{i} callback returned {callback}");
        if callback > 0 {
            return;
        }
        let time = web_time::Instant::now();
        while time.elapsed() < web_time::Duration::from_secs(1) {
            // burn cycles
        }
        i += 1;
    }
}
