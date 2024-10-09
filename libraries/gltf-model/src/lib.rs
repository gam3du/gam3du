#![allow(missing_docs, reason = "TODO add later")]
#![expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

mod model;
mod renderer;

pub use model::Vertex;
pub use renderer::Renderer as GltfModelRenderer;
