use std::{
    cell::RefCell,
    mem,
    num::NonZeroU128,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use super::Plugin;
use crate::{events::GameEvent, GameState};
use gam3du_framework::module::Module;
use log::debug;
use rand::{thread_rng, Rng};
use runtime_python::{PythonRuntime, PythonRuntimeBuilder};
use rustpython_vm::pymodule;

thread_local! {
    pub(crate) static VM_GAME_STATE: RefCell<GameState> = RefCell::default();
}

pub struct PythonPlugin {
    id: NonZeroU128,
    runtime: PythonRuntime,
    // TODO use this with a watchdog to kill a blocking Python VM
    _user_signal: rustpython_vm::signal::UserSignalSender,

    sender: Sender<GameEvent>,
    receiver: Receiver<GameEvent>,
}

impl PythonPlugin {
    #[must_use]
    pub fn new(mut runtime_builder: PythonRuntimeBuilder) -> Self {
        let (sender, receiver) = channel();

        runtime_builder.add_native_module("robot_plugin_api", || Box::new(plugin_api::make_module));
        let user_signal = runtime_builder.enable_user_signals();

        Self {
            id: thread_rng().r#gen(),
            _user_signal: user_signal,
            runtime: runtime_builder.build(),
            sender,
            receiver,
        }
    }

    fn pre_init_vm(&mut self, game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>) {
        debug!("registering `robot_stopped` event");
        game_state
            .event_registries
            .robot_stopped
            .subscribe(self.id, self.sender.clone());
    }

    fn init_vm(&mut self) {
        self.runtime.enter_main();
    }

    fn update_vm(&mut self) {
        'next_event: loop {
            match self.receiver.try_recv() {
                Ok(GameEvent::RobotStopped) => {
                    debug!("robot stopped");
                    self.runtime.interpreter.enter(|vm| {
                        let module = self.runtime.module.as_ref().unwrap();
                        let callback = module.get_attr("on_robot_stopped", vm).unwrap();
                        callback.call((), vm).unwrap_or_else(|exception| {
                            vm.print_exception(exception);
                            panic!("on_robot_stopped not callable");
                        });
                    });
                }
                Err(TryRecvError::Empty) => {
                    break 'next_event;
                }
                Err(TryRecvError::Disconnected) => {
                    todo!();
                }
            }
        }

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
        self.pre_init_vm(game_state);
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
    #![expect(
        clippy::needless_pass_by_value,
        reason = "cannot pass &str in this macro"
    )]

    use super::VM_GAME_STATE;
    use crate::api::EngineApi;
    use log::{debug, error, info, trace, warn};
    use rustpython_vm::VirtualMachine;

    #[pyfunction]
    fn move_forward(duration: u64, _vm: &VirtualMachine) -> bool {
        trace!("pyfunction: move_forward({duration})");
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.move_forward(duration))
    }

    #[pyfunction]
    fn draw_forward(duration: u64, _vm: &VirtualMachine) -> bool {
        trace!("pyfunction: draw_forward({duration})");
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.draw_forward(duration))
    }

    #[pyfunction]
    fn turn_left(duration: u64, _vm: &VirtualMachine) {
        trace!("pyfunction: turn_left({duration})");
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.turn_left(duration));
    }

    #[pyfunction]
    fn turn_right(duration: u64, _vm: &VirtualMachine) {
        trace!("pyfunction: turn_right({duration})");
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.turn_right(duration));
    }

    #[pyfunction]
    fn paint_tile(_vm: &VirtualMachine) {
        trace!("pyfunction: paint_tile()");
        VM_GAME_STATE.with_borrow_mut(EngineApi::paint_tile);
    }

    #[pyfunction]
    fn robot_color_rgb(red: f32, green: f32, blue: f32, _vm: &VirtualMachine) {
        trace!("pyfunction: robot_color_rgb({red}, {green}, {blue})");
        VM_GAME_STATE.with_borrow_mut(|game_state| game_state.robot_color_rgb(red, green, blue));
    }

    #[pyfunction]
    fn log_error(message: String) {
        error!("Python plugin: {message}");
    }

    #[pyfunction]
    fn log_warn(message: String) {
        warn!("Python plugin: {message}");
    }

    #[pyfunction]
    fn log_info(message: String) {
        info!("Python plugin: {message}");
    }

    #[pyfunction]
    fn log_debug(message: String) {
        debug!("Python plugin: {message}");
    }

    #[pyfunction]
    fn log_trace(message: String) {
        trace!("Python plugin: {message}");
    }
}
