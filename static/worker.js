worker = self;
import("./runtime_python_wasm_bg.wasm")
    .then(wasm => {
        import("./runtime_python_wasm.js").then(module => {
            postMessage("started");
            worker.onmessage = (msg) => {
                let channel = module.Channel.from(msg.data);
                while (true) {
                    channel.run();
                    console.debug("worker: runner terminated, restarting");
                }
            }
        })
    });
