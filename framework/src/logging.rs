use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_LEVEL: Level = Level::TRACE;

fn target_filter() -> filter::Targets {
    filter::Targets::new()
        .with_target("cranelift_codegen", Level::INFO)
        .with_target("wasmtime_cranelift", Level::INFO)
        .with_target("wasmtime", Level::INFO)
        .with_target("wgpu_core", Level::WARN)
        // Workaround for https://github.com/gfx-rs/wgpu/issues/6043
        .with_target("wgpu_core::device::resource", Level::WARN)
        .with_target("wgpu_hal", Level::WARN)
        .with_target("naga", Level::INFO)
        .with_target("calloop", Level::INFO)
        .with_target("rustpython_codegen", Level::DEBUG)
        .with_target("rustpython_parser", Level::DEBUG)
        .with_target("runtime_python", Level::DEBUG)
        .with_target("rustpython_vm", Level::DEBUG)
        .with_target("rustpython_vm::frame", Level::WARN)
        .with_target("engine_robot::model", Level::INFO)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_logger() {
    // A layer that logs events to stdout using the human-readable "pretty" format.
    let logger = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(logger)
        .with(target_filter())
        .with(LevelFilter::from_level(DEFAULT_LEVEL))
        .init();
}

#[cfg(target_arch = "wasm32")]
pub fn init_logger() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers, see note below
        .with_writer(tracing_web::MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = tracing_web::performance_layer()
        .with_details_from_fields(tracing_subscriber::fmt::format::Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events

    // let console_writer = tracing_web::MakeWebConsoleWriter::new().with_pretty_level();

    // let logger = tracing_subscriber::fmt::layer()
    //     .with_ansi(true)
    //     .without_time()
    //     .with_writer(console_writer)
    //     .with_level(false);

    // tracing_subscriber::registry()
    //     .with(logger)
    //     .with(target_filter())
    //     .with(LevelFilter::from_level(DEFAULT_LEVEL))
    //     .init();
}
