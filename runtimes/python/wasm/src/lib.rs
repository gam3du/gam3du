//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::print_stdout,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unwrap_in_result,
    unsafe_code,
    reason = "just a demo"
)]

use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
// use wasm_bindgen::prelude::*;
use wasm_rs_dbg::dbg;
use wasm_rs_shared_channel::spsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Init,
    Done { count: u32 },
}

#[wasm_bindgen]
pub struct Channel {
    sender: Option<spsc::Sender<Request>>,
    receiver: spsc::Receiver<Request>,
}

#[wasm_bindgen]
impl Channel {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default, reason = "TODO")]
    #[must_use]
    pub fn new() -> Channel {
        let (sender, receiver) = spsc::channel::<Request>(1024).split();
        Channel {
            sender: Some(sender),
            receiver,
        }
    }
    #[must_use]
    pub fn from(val: JsValue) -> Self {
        let (sender, receiver) = spsc::SharedChannel::from(val).split();
        Channel {
            sender: Some(sender),
            receiver,
        }
    }

    #[must_use]
    pub fn replica(&self) -> JsValue {
        self.receiver.0.clone().into()
    }

    pub fn run(&mut self) -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let sender = self.sender()?;
        sender.init()?;

        loop {
            dbg!("waiting for messages for 10 seconds");
            match self
                .receiver
                .recv(Some(std::time::Duration::from_secs(1)))?
            {
                None => {}
                Some(request) => {
                    dbg!(&request);
                    if let Request::Done { .. } = request {
                        dbg!("received `Done`, terminating the runner");
                        break;
                    }
                }
            }
            sender.done(3)?;
        }
        Ok(())
    }

    pub fn sender(&mut self) -> Result<Sender, JsValue> {
        match self.sender.take() {
            Some(sender) => Ok(Sender(sender)),
            None => Err("sender is already taken".to_owned().into()),
        }
    }
}

#[wasm_bindgen]
pub struct Sender(spsc::Sender<Request>);

#[wasm_bindgen]
impl Sender {
    pub fn init(&self) -> Result<(), JsValue> {
        self.0.send(&Request::Init)
    }

    pub fn done(&self, count: u32) -> Result<(), JsValue> {
        self.0.send(&Request::Done { count })
    }
}

// #[wasm_bindgen]
// extern "C" {
//     // #[wasm_bindgen(js_namespace = window)]
//     fn callback(s: &str) -> u32;
// }

// #[wasm_bindgen]
// pub fn greet(name: &str) {
//     let mut i = 0;
//     loop {
//         let callback = callback(&format!("Hello, {name}!"));
//         println!("{i} callback returned {callback}");
//         if callback > 0 {
//             return;
//         }
//         let time = web_time::Instant::now();
//         while time.elapsed() < web_time::Duration::from_secs(1) {
//             // burn cycles
//         }
//         i += 1;
//     }
// }
