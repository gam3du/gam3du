// this module is responsible for starting and managing a WebWorker running the engine and application
// it is supposed to be loaded by the main thread within the index.html/index.mjs

import * as Application from "./wasm.js";

const LOG_SRC = "[main:application/module.mjs]";
console.info(LOG_SRC, "/--- loading Application Module ---\\");

console.info(LOG_SRC, "loading Application");
await Application.default();
console.debug(LOG_SRC, "loading Application", Application);

console.info(LOG_SRC, "initializing Application");
export const client_port = Application.init();
console.debug(LOG_SRC, "initializing Application", Application);

// let request_queue = new Array();

// /// This will be called by the Python Runtime for each request to the engine
// export function on_python_request(message) {
//     console.info(LOG_SRC, "on_python_message", message);
//     request_queue.push(message);
// }

// /// This will be called by the engine to check for new requests sent by the Python Runtime
// export function poll_api_client_request() {
//     // console.trace(LOG_SRC, "poll_api_client_request");
//     return request_queue.shift()
// }

// export function connect_api_client(message_port) {
//     console.info(LOG_SRC, "setting message port");
//     Application.connect_api_client(message_port);
//     console.debug(LOG_SRC, "message port was set");
// }

export function start() {
    console.info(LOG_SRC, "starting application");
    Application.start();
    console.debug(LOG_SRC, "application completed");
}

export function reset() {
    console.info(LOG_SRC, "resetting application");
    Application.reset();
    console.debug(LOG_SRC, "resetting completed");
}

console.info(LOG_SRC, "\\--- Application Module loaded ---/");
