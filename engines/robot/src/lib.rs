// TODO remove and fix before release
#![allow(missing_docs)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::todo)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unwrap_in_result)]
// TODO remove ASAP!
#![allow(dead_code)]

mod camera;
mod game_loop;
mod game_state;
mod projection;
mod render_state;
mod renderer;
mod tile;
mod robot_renderer;
mod floor_renderer;

pub use game_loop::GameLoop;
pub use renderer::Renderer;
