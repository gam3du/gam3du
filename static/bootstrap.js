import("./runtime_python_wasm_bg.wasm")
    .then(wasm => import("./index.js"))
    .catch(e => console.error("Error importing `index.js`:", e));
