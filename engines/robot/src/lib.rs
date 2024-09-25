#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::indexing_slicing,
    // clippy::todo,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::missing_panics_doc,
    reason = "TODO remove and fix before release"
)]

mod camera;
mod game_loop;
mod game_state;
mod projection;
mod render_state;
mod renderer;
mod scripting;
mod tile;

pub use game_loop::GameLoop;
pub use game_state::GameState;
pub use render_state::RenderState;
pub use renderer::{Renderer, RendererBuilder};

use rustpython_vm::{builtins::PyModule, pymodule, PyRef, VirtualMachine};

pub fn make_module(vm: &VirtualMachine) -> PyRef<PyModule> {
    engine_api::make_module(vm)
}

#[pymodule]
pub mod engine_api {

    #[pyfunction]
    fn get_current_fps() {
        // just forward to a location outside of this macro so that the IDE can assist us
        // super::message(name, args, kwargs, vm)
    }
}
