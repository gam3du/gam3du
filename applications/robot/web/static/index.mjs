// This is the main entry point of the web-application coordinating the
// loading, initialization and start of all components

import * as RobotWebMain from "./wasm.js";

// Logging prefix to identify this thread and module
const LOG_SRC = "[main:index.mjs]";
console.info(LOG_SRC, "/--- initializing Main Module ---\\");

console.info(LOG_SRC, "loading RobotWebMain");
await RobotWebMain.default();
console.debug(LOG_SRC, "loading RobotWebMain", RobotWebMain);

console.info(LOG_SRC, "initializing RobotWebMain");
RobotWebMain.init();
console.debug(LOG_SRC, "initializing RobotWebMain", RobotWebMain);

console.info(LOG_SRC, "starting PythonRuntime worker");
let worker = new RobotWebMain.PythonWorker();

console.info(LOG_SRC, "Starting RobotWebMain");
RobotWebMain.start();
console.info(LOG_SRC, "RobotWebMain is running");

window.run_script = (source) => {
    console.info(LOG_SRC, "run_script", source);
    return worker.run(source);
};

window.reset_game = () => {
    console.info(LOG_SRC, "Resetting RobotWebMain");
    RobotWebMain.reset();
    console.info(LOG_SRC, "Killing Worker");
    worker.kill();
    console.info(LOG_SRC, "Creating new Worker");
    worker = new RobotWebMain.PythonWorker();
    console.info(LOG_SRC, "Reset complete");

}

console.info(LOG_SRC, "\\--- Main Module initialized ---/");
