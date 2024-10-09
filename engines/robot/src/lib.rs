#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::indexing_slicing,
    clippy::todo,
    clippy::panic,
    // clippy::panic_in_result_fn,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::missing_panics_doc,
    reason = "TODO remove and fix before release"
)]

pub mod api;
mod events;
mod game_loop;
mod game_state;
pub mod plugin;
mod render_state;
mod renderer;
mod tile;

pub use game_loop::GameLoop;
pub use game_state::GameState;
pub use render_state::RenderState;
pub use renderer::{Renderer, RendererBuilder};
