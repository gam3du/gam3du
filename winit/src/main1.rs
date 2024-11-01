fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // env_logger::init();
        // pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        // wasm_bindgen_futures::spawn_local(run(event_loop, window));
        panic!("AAAAAAAAAAAAAhhh!");
    }
}
