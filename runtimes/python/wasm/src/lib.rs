#![expect(missing_docs)]

use std::sync::{Mutex, MutexGuard};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {name}!"));
}

#[wasm_bindgen]
pub fn set_source_code(source_code: String) {
    application_state().set_source_code(source_code);
}

#[wasm_bindgen]
pub fn run() {
    application_state().run();
}

fn application_state() -> MutexGuard<'static, ApplicationState> {
    #[expect(
        clippy::expect_used,
        reason = "a poison error made the application unusable already"
    )]
    APPLICATION_STATE
        .lock()
        .expect("Lock for ApplicationState has been poisoned")
}

static APPLICATION_STATE: Mutex<ApplicationState> = Mutex::new(ApplicationState::new());

struct ApplicationState {
    source_code: String,
}

impl ApplicationState {
    const fn new() -> Self {
        Self {
            source_code: String::new(),
        }
    }

    fn set_source_code(&mut self, source_code: String) {
        self.source_code = source_code;
    }

    fn run(&mut self) {}
}
