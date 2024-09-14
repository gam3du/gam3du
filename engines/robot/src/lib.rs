#![expect(
    missing_docs,
    clippy::indexing_slicing,
    // clippy::todo,
    // clippy::panic,
    clippy::unwrap_used,
    // clippy::unwrap_in_result,
    clippy::missing_panics_doc,
    reason = "TODO remove and fix before release"
)]

mod camera;
mod game_loop;
mod game_state;
mod projection;
mod render_state;
mod renderer;
mod tile;

pub use game_loop::GameLoop;
pub use game_state::GameState;
pub use render_state::RenderState;
pub use renderer::{Renderer, RendererBuilder};
