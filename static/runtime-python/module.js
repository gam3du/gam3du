// this module is responsible for starting and managing a WebWorker running a PythonRuntime
// it is supposed to be loaded by the main thread within the index.html

console.info("main: starting Python Worker");
let worker = new Worker("./runtime-python/worker.js", { type: "module" });
// new Worker("worker.js", { type: "module" });
console.info("main: Python Worker started");
console.info(worker);

worker.onmessage = (message_event) => {
    console.log("message from RuntimePython worker → main", message_event.data);
}

export function set_channel_buffers(buffers) {
    console.log("xxx", buffers);

    let message = {
        type: "set_channel_buffers",
        buffers: buffers,
    };

    console.log("sending message", message);
    worker.postMessage(message);
}

export function run(receiver) {
    let message = {
        type: "run",
    };

    console.log("sending message", message);
    worker.postMessage(message);
}
