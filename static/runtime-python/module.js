// this module is responsible for starting and managing a WebWorker running a PythonRuntime
// it is supposed to be loaded by the main thread within the index.html

const LOG_SRC = "[main:runtime-python/module.js]";
console.info(LOG_SRC, "/--- initializing Python Module ---\\");

console.info(LOG_SRC, "starting Python Worker");
let worker = new Worker("./runtime-python/worker.js", { type: "module" });
console.info(LOG_SRC, "Python Worker started", worker);

worker.onmessage = (message_event) => {
    console.info(LOG_SRC, "message from RuntimePython worker → main", message_event.data);
}

export function set_channel_buffers(buffers) {
    console.info(LOG_SRC, "set_channel_buffers", buffers);

    let message = {
        type: "set_channel_buffers",
        buffers: buffers,
    };

    console.info(LOG_SRC, "sending channel buffers to PythonModule WASM", message);
    worker.postMessage(message);
}

export function run() {
    console.info(LOG_SRC, "run");
    let message = {
        type: "run",
    };

    console.info(LOG_SRC, "sending run-command to PythonModule WASM", message);
    worker.postMessage(message);
}

console.info(LOG_SRC, "\\--- Python Module initialized ---/");
