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
        console.debug(this.log_src, "SharedChannel::send", bytes);

        console.debug(this.log_src, "header", this.header);
        console.debug(this.log_src, "data", this.data);

        const len = bytes.byteLength;
        // if (this.__unused() < len) {
        //     throw "not enough space";
        // }
        const b_use = Atomics.load(this.header, B_USE) == 1;
        console.debug(this.log_src, "b_use", b_use);
        const end_header = b_use ? B_END : A_END;
        console.debug(this.log_src, "end_header", end_header);

        const end = Atomics.load(this.header, end_header);
        console.debug(this.log_src, "end", end);
        this.data.set(new Uint8Array(bytes), end);
        // for i in 0..len {
        //     this.data.set_index(end + i, bytes.get_index(i));
        // }
        Atomics.store(this.header, end_header, end + len);
        Atomics.notify(this.header, end_header);
        Atomics.notify(this.header, A_START);

        console.debug(this.log_src, "sending complete");
        console.debug(this.log_src, "header", this.header);
        console.debug(this.log_src, "data", this.data);
        // this.__maybe_switch();
    }

    __unused() {
        let b_use = Atomics.load(this.header, B_USE) == 1; // as u32
        if (b_use) {
            let a_start = Atomics.load(this.header, A_START); // as u32
            let b_end = Atomics.load(this.header, B_END); // as u32
            return a_start - b_end;
        } else {
            let a_end = Atomics.load(this.header, A_END); // as u32
            return self.len - a_end;
        }
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
