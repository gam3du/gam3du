// This is the main entry point of the web-application coordinating the
// loading, initialization and start of all components

import * as PythonRuntime from "./runtime-python/module.js";
import * as Application from "./application/module.js";

// Logging prefix to identify this thread and module
const LOG_SRC = "[main:index.html]";
console.info(LOG_SRC, "/--- initializing Main Module ---\\");

// Capacity of the buffer backing the message channel between the PythonRuntime and the engine
const CHANNEL_CAPACITY = 65536;

console.info(LOG_SRC, "creating buffers backing the message channel");
let channel_buffers = [new SharedArrayBuffer(4 * 4), new SharedArrayBuffer(CHANNEL_CAPACITY)];
console.debug(LOG_SRC, "buffers", channel_buffers);

console.info(LOG_SRC, "starting PythonRuntime worker");
await PythonRuntime.start_worker(channel_buffers, Application.on_python_request);
console.info(LOG_SRC, "PythonRuntime worker started");

console.info(LOG_SRC, "Setting channel buffers for application");
Application.connect_api_client(channel_buffers);
console.info(LOG_SRC, "channel buffers for application were set");

console.info(LOG_SRC, "Starting Application");
Application.start();
console.info(LOG_SRC, "Application is running");

console.info(LOG_SRC, "\\--- Main Module initialized ---/");
