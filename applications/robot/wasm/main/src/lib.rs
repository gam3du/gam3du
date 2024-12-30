#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    // clippy::todo,
    // clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    // clippy::unwrap_in_result,

    reason = "TODO remove before release"
)]

use std::{cell::RefCell, mem, rc::Rc};

use gam3du_framework_common::message::ServerToClientMessage;
// use serde::Deserialize;
use tracing::{debug, info, trace};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use wasm_rs_shared_channel::spsc;
use web_sys::{
    js_sys::{self, Uint8Array},
    MessageEvent, MessagePort, Worker, WorkerOptions,
};

const CHANNEL_CAPACITY: u32 = 0x1_0000;

// #[cfg(target_family = "wasm")]
// pub use wasm::{init, start};

#[wasm_bindgen(start)]
fn main() -> Result<(), wasm_bindgen::JsValue> {
    gam3du_framework::init_logger();
    tracing::info!("application loaded");
    Ok(())
}

// #[wasm_bindgen]
// pub struct Foo {
//     internal: i32,
// }

// #[wasm_bindgen]
// impl Foo {
//     #[wasm_bindgen(constructor)]
//     pub fn new(val: i32) -> Foo {
//         Foo { internal: val }
//     }

//     pub fn get(&self) -> i32 {
//         self.internal
//     }

//     pub fn set(&mut self, val: i32) {
//         self.internal = val;
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerState {
    Loading,
    Initializing,
    Ready,
    Running,
}

#[wasm_bindgen]
pub struct PythonWorker {
    worker_state: Rc<RefCell<WorkerState>>,
    // command_port: MessagePort,
    worker: Rc<RefCell<Worker>>,
    // /// used to send messages to the possibly blocked Python Worker
    // sender: spsc::Sender<ServerToClientMessage>,
    // /// used to receive messages within the possibly blocked Python Worker
    // receiver: Option<spsc::Receiver<ServerToClientMessage>>,
}

#[wasm_bindgen]
impl PythonWorker {
    #[wasm_bindgen(constructor)]
    pub fn new(command_port: MessagePort) -> Self {
        info!("creating new Python runtime");

        info!("creating buffers backing the message channel");
        let (sender, receiver) = spsc::channel::<ServerToClientMessage>(CHANNEL_CAPACITY).split();

        debug!("command port {command_port:?}");

        debug!("setting onmessage handler for server messages on command port");
        let server_message_handler = Self::get_on_server_message(sender);
        command_port.set_onmessage(Some(server_message_handler.as_ref().unchecked_ref()));
        mem::forget(server_message_handler);

        let worker_state = Rc::new(RefCell::new(WorkerState::Loading));

        debug!("creating new WebWorker");
        let worker_options = WorkerOptions::new();
        worker_options.set_type(web_sys::WorkerType::Module);
        let worker =
            Worker::new_with_options("./runtime-python/worker.mjs", &worker_options).unwrap();
        let worker = Rc::new(RefCell::new(worker));
        debug!("registering handler for incoming worker messages");
        let worker_message_handler = Self::get_on_worker_message(
            receiver,
            command_port,
            Rc::clone(&worker_state),
            Rc::clone(&worker),
        );
        worker
            .borrow_mut()
            .set_onmessage(Some(worker_message_handler.as_ref().unchecked_ref()));
        mem::forget(worker_message_handler);

        debug!("PythonWorker successfully created");
        Self {
            worker_state,
            worker,
        }
    }

    /// Create a closure to act on the messages sent by the server.
    ///
    /// called whenever the engine sends a response message back to the python runtime
    /// the message will be forwarded to the shared channel of the blocked worker
    fn get_on_server_message(
        sender: spsc::Sender<ServerToClientMessage>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::new(move |message_event: MessageEvent| {
            let data = message_event.data();
            debug!("on_server_message: {data:?}");

            let bytes = Uint8Array::new(&data).to_vec();
            let message = bincode::deserialize(&bytes).unwrap();

            sender.send(&message).unwrap();
        })
    }

    // #[wasm_bindgen]
    // pub async fn init(&mut self) {
    //     info!("starting Python Worker");
    //     let worker = Worker::new("./runtime-python/worker.mjs"); //, { type: "module" });
    //                                                              // self.worker.onmessage = self.on_worker_loaded;

    //     info!("Waiting for PythonWorker to be loaded");
    //     self.send_async(null, "loading").await;
    //     info!("PythonWorker has been loaded successfully");

    //     // let message = {
    //     //     type: "set_channel_buffers",
    //     //     buffers: self.shared_channel.buffers(),
    //     // };
    //     info!("sending channel buffers to PythonWorker", message);
    //     self.send_async(message, "setting channel buffers").await;

    //     assert!(
    //         self.worker.replace(worker).is_none(),
    //         "worker has already been created"
    //     );

    //     info!("PythonWorker has been configured", message);
    // }

    // /// Sends a message to the worker and waits for a response.
    // ///
    // /// This is a convenience function to simplify the initial setup and will not work if the runtime
    // /// is blocked.
    // async fn send_async(payload: &[u8], topic: &str) {
    //     return Promise::new(|resolve, reject| {
    //         let worker = self.worker;

    //         fn unexpected(message_event: MessageEvent) {
    //             let message = message_event.data;
    //             error!(
    //                 "unexpected message from worker while no handler was registered:",
    //                 message
    //             );
    //         }

    //         // message event handler forwarding all PythonRequests to the Application
    //         fn on_worker_response(message_event: MessageEvent) {
    //             let message = message_event.data;
    //             trace!("response from worker", message);
    //             resolve(message);
    //             worker.onmessage = unexpected;
    //         }

    //         worker.onmessage = on_worker_response;

    //         if (payload != null) {
    //             worker.postMessage(payload);
    //         }
    //     });
    // }

    // message event handler forwarding all PythonRequests to the Application

    fn get_on_worker_message(
        receiver: spsc::Receiver<ServerToClientMessage>,
        command_port: MessagePort,
        worker_state: Rc<RefCell<WorkerState>>,
        worker: Rc<RefCell<Worker>>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::new(move |message_event: MessageEvent| {
            let message = message_event.data();
            let mut worker_state = worker_state.borrow_mut();
            trace!("message from worker (state: {worker_state:?}): {message:?}");

            match *worker_state {
                WorkerState::Loading => {
                    let receiver = receiver.0.clone().into();

                    let message = js_sys::Object::new();
                    assert!(
                        js_sys::Reflect::set(&message, &"type".into(), &"init".into()).unwrap(),
                        "failed to set `type` field of message"
                    );
                    assert!(
                        js_sys::Reflect::set(&message, &"receiver".into(), &receiver).unwrap(),
                        "failed to set `receiver` field of message"
                    );

                    info!("sending shared channel to PythonWorker: {message:?}");
                    worker.borrow().post_message(&message).unwrap();

                    info!("Loading: loading acknowledged by Worker; switch mode to `Initializing`");
                    *worker_state = WorkerState::Initializing;
                }
                WorkerState::Initializing => {
                    info!("Initializing: initialization acknowledged by Worker; switch mode to `Ready`");
                    *worker_state = WorkerState::Ready;
                }
                WorkerState::Ready => {
                    // panic!("No messages are expected in ready state");
                    // switch to `running` state so that all further messages from the Worker will be blindly
                    // forwarded to the engine
                    info!("Ready: run command acknowledged by Worker; switch mode to `Running`");
                    *worker_state = WorkerState::Running;
                }
                WorkerState::Running => {
                    info!("Running: forwarding worker command to engine server");
                    command_port.post_message(&message).unwrap();
                }
            }
        })
    }

    // fn on_worker_message(&mut self, message_event: MessageEvent) {

    // }

    pub fn run(&mut self, source: &str) {
        info!("starting PythonRuntime (this will block the containing WebWorker until the script completes)");

        let worker_state = self.worker_state.borrow_mut();
        assert_eq!(
            *worker_state,
            WorkerState::Ready,
            "Worker needs to be ready to run"
        );

        let message = js_sys::Object::new();
        assert!(
            js_sys::Reflect::set(&message, &"type".into(), &"run".into()).unwrap(),
            "failed to set `type` field of message"
        );
        assert!(
            js_sys::Reflect::set(&message, &"source".into(), &source.into()).unwrap(),
            "failed to set `source` field of message"
        );

        info!("sending run-command to PythonModule WASM: {message:?}");
        self.worker.borrow().post_message(&message).unwrap();
    }

    #[wasm_bindgen]
    pub fn kill(self) {
        self.worker.borrow().terminate();
    }
}
