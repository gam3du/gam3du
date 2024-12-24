// translated from crate `wasm_rs_shared_channel::spsc` (Sender part)

const A_START = 0;
const A_END = 1;
const B_END = 2;
const B_USE = 3;

export default class SharedChannel {

    // len: Capacity of the buffer backing the message channel between the PythonRuntime and the engine
    constructor(len, log_src) {
        console.info(log_src, "creating new shared channel");
        this.log_src = log_src;
        this.len = len;
        this.header_bytes = new SharedArrayBuffer(4 * 4);
        this.header = new Int32Array(this.header_bytes);
        this.data_bytes = new SharedArrayBuffer(len);
        this.data = new Uint8Array(this.data_bytes);
    }

    buffers() {
        return [this.header_bytes, this.data_bytes];
    }

    /// Sends a sequence of bytes (ArrayBuffer or TypedArrayBuffer) into the channel
    ///
    /// If there isn't enough space currently in the channel to accommodate
    /// the value, it'll throw a JavaScript exception (`"not enough space"`)
    send(bytes) {
        const len = bytes.byteLength;
        if (this.unused() < len) {
            throw "not enough space";
        }
        const b_use = Atomics.load(this.header, B_USE) == 1;
        const end_header = b_use ? B_END : A_END;

        const end = Atomics.load(this.header, end_header);
        this.data.set(new Uint8Array(bytes), end);
        // for i in 0..len {
        //     this.data.set_index(end + i, bytes.get_index(i));
        // }
        Atomics.store(this.header, end_header, end + len);
        Atomics.notify(this.header, end_header);
        Atomics.notify(this.header, A_START);

        this.__maybe_switch();
    }

    __maybe_switch() {
        const a_start = Atomics.load(this.header, A_START);
        const a_end = Atomics.load(this.header, A_END);
        const b_end = Atomics.load(this.header, B_END);
        if (this.len - a_end < a_start - b_end) {
            Atomics.store(this.header, B_USE, 1);
        }
    }

};
