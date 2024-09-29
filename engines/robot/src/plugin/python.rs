use std::{cell::RefCell, mem};

use super::Plugin;
use crate::GameState;
use gam3du_framework::module::Module;
use runtime_python::{PythonRuntime, PythonRuntimeBuilder};
use rustpython_vm::pymodule;

// this will be needed when running a python VM
thread_local! {
    pub(crate) static VM_GAME_STATE: RefCell<GameState> = RefCell::default();
}

pub struct PythonPlugin {
    runtime: PythonRuntime,
    // TODO use this with a watchdog to kill a blocking Python VM
    user_signal: rustpython_vm::signal::UserSignalSender,
}

impl PythonPlugin {
    #[must_use]
    pub fn new(mut runtime_builder: PythonRuntimeBuilder) -> Self {
        runtime_builder
            .add_native_module("robot_control_api", || Box::new(plugin_api::make_module));
        let user_signal = runtime_builder.enable_user_signals();

        Self {
            user_signal,
            runtime: runtime_builder.build(),
        }
    }

    fn init_vm(&mut self) {
        self.runtime.enter_main();
    }

    fn update_vm(&mut self) {
        self.runtime.wake();
    }

    fn swap_vm_game_state(game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        VM_GAME_STATE.with_borrow_mut(|locked_state| {
            mem::swap(locked_state, game_state.as_mut());
        });
    }
}

impl Plugin for PythonPlugin {
    fn init(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        Self::swap_vm_game_state(game_state);
        self.init_vm();
        Self::swap_vm_game_state(game_state);
    }

    fn update(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        Self::swap_vm_game_state(game_state);
        self.update_vm();
        Self::swap_vm_game_state(game_state);
    }
}

#[pymodule]
mod plugin_api {
    use super::VM_GAME_STATE;
    use crate::api::EngineApi;
    use rustpython_vm::{builtins::PyBool, PyResult, VirtualMachine};

    #[pyfunction]
    fn move_forward(duration: u64, _vm: &VirtualMachine) -> bool {
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.move_forward(duration))
    }

    #[pyfunction]
    fn draw_forward(duration: u64, _vm: &VirtualMachine) -> bool {
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.draw_forward(duration))
    }

    #[pyfunction]
    fn turn_left(duration: u64, _vm: &VirtualMachine) {
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.turn_left(duration));
    }

    #[pyfunction]
    fn turn_right(duration: u64, _vm: &VirtualMachine) {
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.turn_right(duration));
    }

    #[pyfunction]
    fn paint_tile(_vm: &VirtualMachine) {
        VM_GAME_STATE.with_borrow_mut(EngineApi::paint_tile);
    }

    #[pyfunction]
    fn robot_color_rgb(red: f32, green: f32, blue: f32, _vm: &VirtualMachine) {
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.robot_color_rgb(red, green, blue));
    }
}
