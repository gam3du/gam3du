use crate::GameState;

mod native;
mod python;

pub use native::NativePlugin;
pub use python::PythonPlugin;

pub trait Plugin {
    fn init(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>);

    fn update(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>);
}
