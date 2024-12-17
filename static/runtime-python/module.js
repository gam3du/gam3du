// this module is responsible for starting and managing a WebWorker running a PythonRuntime
// it is supposed to be loaded by the main thread within the index.html/index.js

const LOG_SRC = "[main:runtime-python/module.js]";
console.info(LOG_SRC, "/--- initializing Python Module ---\\");

let worker;

export function start_worker(channel_buffers, on_python_message) {
    console.info(LOG_SRC, "start_worker", channel_buffers);

    console.info(LOG_SRC, "starting Python Worker");
    worker = new Worker("./runtime-python/worker.js", { type: "module" });


    // message event handler forwarding all PythonRequests to the Application
    function forward_request(message_event) {
        let message = message_event.data;
        console.trace(LOG_SRC, "forwarding request from Python to Application", message);
        on_python_message(message);
    }

    let init_promise = new Promise((resolve, reject) => {
        function init_event_handler(message_event) {
            worker.removeEventListener("message", init_event_handler);

            let message = message_event.data;
            console.info(LOG_SRC, "message from RuntimePython worker → main", message_event.data);
            switch (message.type) {
                case "loaded":
                    console.info(LOG_SRC, "worker successfully loaded");


                    console.info(LOG_SRC, "sending the channel buffers to the PythonRuntime");
                    set_channel_buffers(channel_buffers);

                    console.info(LOG_SRC, "registering message handler for requests");
                    worker.addEventListener("message", forward_request);

                    console.info(LOG_SRC, "resolving init promise");
                    resolve();

                    break;

                // case "run":

                //     console.info(LOG_SRC, "starting PythonRuntime (this will block the containing WebWorker until the script completes)");
                //     run();

                //     console.info(LOG_SRC, "unregistering message handler for requests");
                //     worker.removeEventListener("message", forward_request);
                //     break;

                default:
                    console.info(LOG_SRC, "unknown message type from worker", message.type);
                    reject()
                    break;
            }

        }

        worker.addEventListener("message", init_event_handler);
    });

    console.info(LOG_SRC, "Python Worker started, waiting for initialization", worker);
    return init_promise;
}

function set_channel_buffers(channel_buffers) {
    let message = {
        type: "set_channel_buffers",
        buffers: channel_buffers,
    };

    console.info(LOG_SRC, "sending channel buffers to PythonWorker", message);
    worker.postMessage(message);
}

export function run() {
    console.info(LOG_SRC, "starting PythonRuntime (this will block the containing WebWorker until the script completes)");

    console.info(LOG_SRC, "run");
    let message = {
        type: "run",
    };

    console.info(LOG_SRC, "sending run-command to PythonModule WASM", message);
    worker.postMessage(message);
}

console.info(LOG_SRC, "\\--- Python Module initialized ---/");
