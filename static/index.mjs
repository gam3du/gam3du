// This is the main entry point of the web-application coordinating the
// loading, initialization and start of all components

import SharedChannel from "./shared-channel.mjs";
import * as PythonRuntime from "./runtime-python/module.mjs";
import * as Application from "./application/module.mjs";

// Logging prefix to identify this thread and module
const LOG_SRC = "[main:index.html]";
console.info(LOG_SRC, "/--- initializing Main Module ---\\");

// Capacity of the buffer backing the message channel between the PythonRuntime and the engine
const CHANNEL_CAPACITY = 65536;
console.info(LOG_SRC, "creating buffers backing the message channel");
const channel = new SharedChannel(CHANNEL_CAPACITY, "[main:SharedChannel]");
console.debug(LOG_SRC, "SharedChannel buffers", channel.buffers());

console.info(LOG_SRC, "Setting channel buffers for application");
Application.connect_api_client(channel.buffers());
console.info(LOG_SRC, "channel buffers for application were set");

console.info(LOG_SRC, "Starting Application");
Application.start();
console.info(LOG_SRC, "Application is running");


export function run_script(source) {
    console.info(LOG_SRC, "starting PythonRuntime worker");
    PythonRuntime.start_worker(channel.buffers(), Application.on_python_request).then(() => {
        console.info(LOG_SRC, "PythonRuntime.run()");
        PythonRuntime.run(source);
    });
    console.info(LOG_SRC, "PythonRuntime worker started");
}

window.run_script = run_script;

window.reset_game = () => {
    console.info(LOG_SRC, "Resetting Application");
    Application.reset();
}

// document.getElementById()

// alert("index loaded");
// window.document.addEventListener("DOMContentLoaded", function () {
//     alert("DOM loaded");
// });

console.info(LOG_SRC, "\\--- Main Module initialized ---/");
