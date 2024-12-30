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
