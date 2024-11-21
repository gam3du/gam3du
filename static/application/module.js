// this module is responsible for starting and managing a WebWorker running the engine and application
// it is supposed to be loaded by the main thread within the index.html/index.js

import * as Application from "./wasm.js";

const LOG_SRC = "[main:application/module.js]";
console.info(LOG_SRC, "/--- initializing Application Module ---\\");

console.info(LOG_SRC, "initializing Application");
await Application.default();
console.debug(LOG_SRC, "initialized Application", Application);

let request_queue = new Array();

/// This will be called by the Python Runtime for each request to the engine
export function on_python_request(message) {
    request_queue.push(message);
}

/// This will be called by the engine to check for new requests sent by the Python Runtime
export function poll_api_client_request() {
    request_queue.shift()
}

export function connect_api_client(channel_buffers) {
    console.info(LOG_SRC, "setting channel buffers");
    Application.connect_api_client(channel_buffers);
    console.debug(LOG_SRC, "channel buffers were set");
}

export function start() {
    console.info(LOG_SRC, "starting application");
    Application.start();
    console.debug(LOG_SRC, "application completed");
}

console.info(LOG_SRC, "\\--- Application Module initialized ---/");
