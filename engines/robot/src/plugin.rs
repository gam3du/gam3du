use crate::GameState;

mod native;
mod python;

pub use native::NativePlugin;
pub use python::PythonPlugin;

pub trait Plugin {
    // TODO try changing signature to `&mut GameState`
    fn init(&mut self, game_state: &mut GameState);

    fn update(&mut self, game_state: &mut GameState);
}
