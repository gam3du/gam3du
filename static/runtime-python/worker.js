// This is the WebWorker hosting the Python runtime
// After initialization this will make the thread block!
// The only way to control it is through the channel backed
// by a SharedArrayBuffer which is created upon initialization.
// see https://github.com/RustPython/RustPython/issues/5435
// and https://users.rust-lang.org/t/using-async-in-a-call-back-of-a-blocking-library/121089
import * as PythonRuntime from './wasm.js';

console.info("Python Worker is loading");

await PythonRuntime.default();
console.info("Python Worker WASM initialized");

PythonRuntime.init();
console.info("PythonRuntime initialized");

self.onmessage = (message_event) => {
    let message = message_event.data;
    console.info("message received", message);
    switch (message.type) {
        case "set_channel_buffers":
            console.info("sending to WASM", message.buffers);
            PythonRuntime.set_channel_buffers(message.buffers);
            console.info("gone?", message.buffers);
            break;
        case "run":
            PythonRuntime.run();
            break;
        default:
            console.error("unknown message type: ", message.type);
    }
}

console.info("Python Worker main terminated");
