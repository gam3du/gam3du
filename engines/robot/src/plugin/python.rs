use std::cell::RefCell;

use rustpython_vm::{builtins::PyModule, pymodule, VirtualMachine};

use crate::GameState;

use super::Plugin;

// this will be needed when running a python VM
thread_local! {
    pub(crate) static LOCKED_GAME_STATE: RefCell<GameState> = RefCell::default();
}

#[derive(Default)]
pub struct PythonPlugin {}

impl Plugin for PythonPlugin {
    fn init(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        todo!()
    }

    fn update(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        todo!()
    }
}

pub fn make_module(vm: &VirtualMachine) -> rustpython_vm::PyRef<PyModule> {
    engine_api::make_module(vm)
}

#[pymodule]
mod engine_api {

    #[pyfunction]
    fn get_current_fps() {
        // just forward to a location outside of this macro so that the IDE can assist us
        // super::message(name, args, kwargs, vm)
    }
}
