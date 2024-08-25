// TODO remove and fix before release
#![allow(missing_docs)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::todo)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unwrap_in_result)]
#![allow(clippy::missing_panics_doc)]
// TODO remove ASAP!
#![allow(dead_code)]

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
