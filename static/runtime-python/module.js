// this module is responsible for starting and managing a WebWorker running a PythonRuntime
// it is supposed to be loaded by the main thread within the index.html

console.info("main: starting Python Worker");
let worker = new Worker("./runtime-python/worker.js", { type: "module" });
// new Worker("worker.js", { type: "module" });
console.info("main: Python Worker started");
console.info(worker);
