// This is the WebWorker hosting the Python runtime
// After initialization this will make the thread block!
// The only way to control it is through the channel backed
// by a SharedArrayBuffer which is created upon initialization.
// see https://github.com/RustPython/RustPython/issues/5435
// and https://users.rust-lang.org/t/using-async-in-a-call-back-of-a-blocking-library/121089
import * as PythonRuntime from './wasm.js';

export function send_api_client_request(request_bytes) {
    console.debug("forwarding", request_bytes.length, "bytes");
    self.postMessage(request_bytes);
}

const LOG_SRC = "[python-worker:runtime-python/worker.mjs]";
console.info(LOG_SRC, "/--- initializing Python Worker ---\\");

await PythonRuntime.default();
console.info(LOG_SRC, "Python Worker WASM initialized");

PythonRuntime.init();
console.info(LOG_SRC, "PythonRuntime initialized");

self.onmessage = (message_event) => {
    let message = message_event.data;
    console.info(LOG_SRC, "message received", message);
    switch (message.type) {
        case "set_channel_buffers":
            console.info(LOG_SRC, "sending to WASM", message.buffers);
            PythonRuntime.set_channel_buffers(message.buffers);
            break;
        case "run":
            console.debug(LOG_SRC, "calling PythonRuntime.run()");
            PythonRuntime.run(message.source);
            console.debug(LOG_SRC, "PythonRuntime.run() completed");
            break;
        default:
            console.error(LOG_SRC, "unknown message type: ", message.type);
    }
}

console.info(LOG_SRC, "notifying module about completion");
self.postMessage({ type: "loaded" });

console.info(LOG_SRC, "\\--- Python Worker initialized ---/");