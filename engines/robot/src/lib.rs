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

pub mod api;
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
pub use scripting::Plugin;
