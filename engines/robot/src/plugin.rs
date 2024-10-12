use crate::GameState;

mod python;

pub use python::PythonPlugin;

pub trait Plugin {
    // TODO try changing signature to `&mut GameState`
    fn init(&mut self, game_state: &mut GameState);

    fn update(&mut self, game_state: &mut GameState);
}
