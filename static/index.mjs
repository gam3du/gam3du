// This is the main entry point of the web-application coordinating the
// loading, initialization and start of all components

// import SharedChannel from "./shared-channel.mjs";
// import PythonRuntime from "./runtime-python/module.mjs";
import * as Application from "./application/module.mjs";
import * as Wasm from "./wasm.js";

// Logging prefix to identify this thread and module
const LOG_SRC = "[main:index.mjs]";
console.info(LOG_SRC, "/--- initializing Main Module ---\\");

console.info(LOG_SRC, "loading Wasm");
await Wasm.default();
console.debug(LOG_SRC, "loading Wasm", Wasm);


// // Capacity of the buffer backing the message channel between the PythonRuntime and the engine
// const CHANNEL_CAPACITY = 65536;
// console.info(LOG_SRC, "creating buffers backing the message channel");
// const channel = new SharedChannel(CHANNEL_CAPACITY, "[main:SharedChannel]");
// console.debug(LOG_SRC, "SharedChannel buffers", channel.buffers());

console.info(LOG_SRC, "starting PythonRuntime worker");
// const message_channel = new MessageChannel();
// let scripting_runtime = new PythonRuntime(Application.client_port);
let worker = new Wasm.PythonWorker(Application.client_port);

// console.info(LOG_SRC, "Setting channel buffers for application");
// Application.connect_api_client(message_channel.port2);
// console.info(LOG_SRC, "channel buffers for application were set");

console.info(LOG_SRC, "Starting Application");
Application.start();
console.info(LOG_SRC, "Application is running");

// console.info(LOG_SRC, "Initializing scripting runtime");
// await scripting_runtime.init();
// console.info(LOG_SRC, "Scripting has been initialized");

export async function run_script(source) {
    console.info(LOG_SRC, "run_script", source);
    return worker.run(source);
    // PythonRuntime.start_worker(channel.buffers(), Application.on_python_request).then(() => {
    //     console.info(LOG_SRC, "PythonRuntime.run()");
    //     PythonRuntime.run(source);
    // });
    // console.info(LOG_SRC, "PythonRuntime worker started");
}

window.run_script = run_script;

window.reset_game = () => {
    console.info(LOG_SRC, "Resetting Application");
    Application.reset();
    console.info(LOG_SRC, "Killing Worker");
    worker.kill();
    console.info(LOG_SRC, "Creating new Worker");
    worker = new Wasm.PythonWorker(Application.client_port);
    console.info(LOG_SRC, "Reset complete");

}

// document.getElementById()

// alert("index loaded");
// window.document.addEventListener("DOMContentLoaded", function () {
//     alert("DOM loaded");
// });

console.info(LOG_SRC, "\\--- Main Module initialized ---/");
