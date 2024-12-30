// this module is responsible for starting and managing a WebWorker running a PythonRuntime
// it is supposed to be loaded by the main thread within the index.html/index.mjs

// import SharedChannel from "../shared-channel.mjs";

const LOG_SRC = "[main:runtime-python/module.mjs]";
console.info(LOG_SRC, "/--- loading Python Module ---\\");

// // Capacity of the buffer backing the message channel between the PythonRuntime and the engine
// const CHANNEL_CAPACITY = 65536;

// export default class PythonRuntime {

//     command_port = null;
//     shared_channel = null;
//     worker = null;

//     constructor(command_port) {
//         console.info(LOG_SRC, "creating new Python runtime");

//         console.info(LOG_SRC, "creating buffers backing the message channel");
//         this.shared_channel = new SharedChannel(CHANNEL_CAPACITY, "[main:SharedChannel]");
//         console.debug(LOG_SRC, "SharedChannel buffers", this.shared_channel.buffers());

//         console.debug(LOG_SRC, "command port", command_port);
//         this.command_port = command_port;
//         this.command_port.onmessage = this.on_server_response.bind(this);
//     }

//     async init() {
//         console.info(LOG_SRC, "starting Python Worker");
//         this.worker = new Worker("./runtime-python/worker.mjs", { type: "module" });
//         // this.worker.onmessage = this.on_worker_loaded;

//         console.info(LOG_SRC, "Waiting for PythonWorker to be loaded");
//         await this.send_async(null, "loading");
//         console.info(LOG_SRC, "PythonWorker has been loaded successfully");

//         const message = {
//             type: "set_channel_buffers",
//             buffers: this.shared_channel.buffers(),
//         };
//         console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
//         await this.send_async(message, "setting channel buffers");

//         console.info(LOG_SRC, "PythonWorker has been configured", message);
//     }

//     /// Sends a message to the worker and waits for a response.
//     ///
//     /// This is a convenience function to simplify the initial setup and will not work if the runtime
//     /// is blocked.
//     async send_async(payload, topic) {
//         return new Promise((resolve, reject) => {

//             const worker = this.worker;

//             function unexpected(message_event) {
//                 let message = message_event.data;
//                 console.error(LOG_SRC, "unexpected message from worker while no handler was registered:", message);
//             }

//             // message event handler forwarding all PythonRequests to the Application
//             function on_worker_response(message_event) {
//                 let message = message_event.data;
//                 console.trace(LOG_SRC, "response from worker", message);
//                 resolve(message);
//                 worker.onmessage = unexpected;
//             }

//             worker.onmessage = on_worker_response;

//             if (payload !== null) {
//                 worker.postMessage(payload);
//             }
//         });
//     }

//     // on_worker_loaded(message_event) {
//     //     console.info(LOG_SRC, "worker successfully loaded");

//     //     const message = {
//     //         type: "set_channel_buffers",
//     //         buffers: this.shared_channel.buffers(),
//     //     };
//     //     console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
//     //     worker.postMessage(message);

//     //     console.info(LOG_SRC, "registering message handler for requests");
//     //     worker.addEventListener("message", forward_request);

//     //     console.info(LOG_SRC, "resolving init promise");
//     //     resolve();

//     //     this.worker.onmessage = this.on_worker_configured;
//     // }

//     // on_worker_configured(message_event) {
//     //     const message = message_event.data;
//     //     console.info(LOG_SRC, "message from RuntimePython worker → main", message_event.data);
//     //     switch (message.type) {
//     //         case "loaded":
//     //             console.info(LOG_SRC, "worker successfully loaded");

//     //             const message = {
//     //                 type: "set_channel_buffers",
//     //                 buffers: this.shared_channel.buffers(),
//     //             };
//     //             console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
//     //             worker.postMessage(message);

//     //             console.info(LOG_SRC, "registering message handler for requests");
//     //             worker.addEventListener("message", forward_request);

//     //             console.info(LOG_SRC, "resolving init promise");
//     //             resolve();

//     //             break;

//     //         default:
//     //             console.info(LOG_SRC, "unknown message type from worker", message.type);
//     //             reject()
//     //             break;
//     //     }

//     // }

//     // on_worker_message(message_event) {
//     //     worker.removeEventListener("message", init_event_handler);

//     //     const message = message_event.data;
//     //     console.info(LOG_SRC, "message from RuntimePython worker → main", message_event.data);
//     //     switch (message.type) {
//     //         case "loaded":
//     //             console.info(LOG_SRC, "worker successfully loaded");

//     //             const message = {
//     //                 type: "set_channel_buffers",
//     //                 buffers: this.shared_channel.buffers(),
//     //             };
//     //             console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
//     //             worker.postMessage(message);

//     //             console.info(LOG_SRC, "registering message handler for requests");
//     //             worker.addEventListener("message", forward_request);

//     //             console.info(LOG_SRC, "resolving init promise");
//     //             resolve();

//     //             break;

//     //         default:
//     //             console.info(LOG_SRC, "unknown message type from worker", message.type);
//     //             reject()
//     //             break;
//     //     }

//     // }

//     // called whenever the engine send a response message back to the python runtime
//     // the message will be forwarded to the shared channel of the blocked worker 
//     on_server_response(message_event) {
//         const payload = message_event.data;
//         console.debug(LOG_SRC, "on_server_response: ", payload);
//         this.shared_channel.send(payload);
//     }

//     async run(source) {
//         // TODO error if `init` didn't complete successfully
//         console.info(LOG_SRC, "starting PythonRuntime (this will block the containing WebWorker until the script completes)");

//         console.info(LOG_SRC, "run");
//         let message = {
//             type: "run",
//             source: source,
//         };

//         console.info(LOG_SRC, "sending run-command to PythonModule WASM", message);
//         await this.send_async(message);

//         this.worker.onmessage = (message_event) => {
//             this.command_port.postMessage(message_event.data);
//         };
//     }

//     kill() {
//         this.worker.terminate()
//     }

// }

// let worker;

// export function start_worker(channel_buffers, on_python_message) {
//     console.info(LOG_SRC, "start_worker", channel_buffers);


//     // message event handler forwarding all PythonRequests to the Application
//     function forward_request(message_event) {
//         let message = message_event.data;
//         console.trace(LOG_SRC, "forwarding request from Python to Application", message);
//         on_python_message(message);
//     }

//     let init_promise = new Promise((resolve, reject) => {
//         function init_event_handler(message_event) {


//         }

//     });

//     console.info(LOG_SRC, "Python Worker started, waiting for initialization", worker);
//     return init_promise;
// }

// function set_channel_buffers(channel_buffers) {
//     let message = {
//         type: "set_channel_buffers",
//         buffers: channel_buffers,
//     };

//     console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
//     worker.postMessage(message);
// }

// export function run(source) {
//     console.info(LOG_SRC, "starting PythonRuntime (this will block the containing WebWorker until the script completes)");

//     console.info(LOG_SRC, "run");
//     let message = {
//         type: "run",
//         source: source,
//     };

//     console.info(LOG_SRC, "sending run-command to PythonModule WASM", message);
//     worker.postMessage(message);
// }

console.info(LOG_SRC, "\\--- Python Module loaded ---/");
