// This is the WebWorker hosting the Python runtime
// After initialization this will make the thread block!
// The only way to control it is through the channel backed
// by a SharedArrayBuffer which is created upon initialization.
// see https://github.com/RustPython/RustPython/issues/5435
// and https://users.rust-lang.org/t/using-async-in-a-call-back-of-a-blocking-library/121089
import * as Python from './wasm.js';

console.info("Python Worker is loading");

await Python.default();

console.info("Python Worker WASM initialized");

let channel = new Python.Channel();
console.info("Python Worker Channel created");

let sender = channel.run();
console.info("Python Worker end of blocking");


// // worker = self;
// // import("./runtime_python_wasm_bg.wasm")
// //     .then(wasm => {
// //         import("./runtime_python_wasm.js").then(module => {
// //             postMessage("started");
// //             worker.onmessage = (msg) => {
// //                 let channel = module.Channel.from(msg.data);
// //                 while (true) {
// //                     channel.run();
// //                     console.debug("worker: runner terminated, restarting");
// //                 }
// //             }
// //         })
// //     });

console.info("Python Worker main terminated");
