// This is the main entry point of the web-application coordinating the
// loading, initialization and start of all components

import * as Application from "./application/module.mjs";
import * as Wasm from "./wasm.js";

// Logging prefix to identify this thread and module
const LOG_SRC = "[main:index.mjs]";
console.info(LOG_SRC, "/--- initializing Main Module ---\\");

console.info(LOG_SRC, "loading Wasm");
await Wasm.default();
console.debug(LOG_SRC, "loading Wasm", Wasm);

console.info(LOG_SRC, "starting PythonRuntime worker");
let worker = new Wasm.PythonWorker(Application.client_port);

console.info(LOG_SRC, "Starting Application");
Application.start();
console.info(LOG_SRC, "Application is running");

export async function run_script(source) {
    console.info(LOG_SRC, "run_script", source);
    return worker.run(source);
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

console.info(LOG_SRC, "\\--- Main Module initialized ---/");
