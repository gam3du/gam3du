// this module is responsible for starting and managing a WebWorker running the engine and application
// it is supposed to be loaded by the main thread within the index.html/index.js

import * as Application from "./wasm.js";

const LOG_SRC = "[main:application/module.js]";
console.info(LOG_SRC, "/--- initializing Application Module ---\\");

// console.info(LOG_SRC, "initializing Application");
// await Application.default();
// console.debug(LOG_SRC, "initialized Application", Application);

console.info(LOG_SRC, "\\--- Application Module initialized ---/");
