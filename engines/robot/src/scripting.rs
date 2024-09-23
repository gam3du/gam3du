use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use gam3du_framework::module::Module;
use glam::Vec3;
use runtimes::{
    api::{ApiClient, ApiServer, Identifier, Value},
    message::{ClientToServerMessage, MessageId, RequestMessage, ServerToClientMessage},
};
use rustpython_vm::{pymodule, VirtualMachine};

use crate::{api::EngineApi, GameState};

// this will be needed when running a python VM
// thread_local! {
//     pub(crate) static LOCKED_GAME_STATE: RefCell<GameState> = RefCell::default();
// }

const ROBOT_API_NAME: Identifier = Identifier(Cow::Borrowed("robot"));

pub struct Plugin {
    robot_controllers: Vec<Box<dyn ApiServer>>,
    current_command: Option<(MessageId, usize)>,
}

impl Plugin {
    #[must_use]
    pub fn new() -> Self {
        Self {
            robot_controllers: Vec::new(),
            current_command: None,
        }
    }

    pub fn add_robot_controller(&mut self, robot_controller: Box<dyn ApiServer>) {
        let api_name = robot_controller.api_name();
        assert_eq!(
            api_name, &ROBOT_API_NAME,
            "expected api server for the 'robot' api, but '{api_name}' was given"
        );
        self.robot_controllers.push(robot_controller);
    }

    pub(crate) fn init(&mut self) {
        //
    }

    pub(crate) fn update(
        &mut self,
        game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>,
    ) {
        // check whether the engine it still animating something
        if !game_state.is_idle() {
            return;
        }

        if let Some((command_id, controller_index)) = self.current_command {
            self.robot_controllers[controller_index]
                .send_response(command_id, serde_json::Value::Null);
            self.current_command.take();
        }

        // this will be needed when running a python VM
        // LOCKED_GAME_STATE.with_borrow_mut(|locked_state| {
        //     mem::swap(locked_state, game_state.as_mut());
        // });
        // CAUTION: `game_state` contains placeholder data until we swap it back

        'next_endpoint: for (endpoint_index, robot_api_endpoint) in
            self.robot_controllers.iter_mut().enumerate()
        {
            'next_robot_api_event: loop {
                match robot_api_endpoint.poll_request() {
                    Some(ClientToServerMessage::Request(request)) => {
                        let RequestMessage {
                            // TODO remember id to send a matching response once the command completed
                            id,
                            command,
                            arguments,
                        } = request;

                        let command_result =
                            Self::process_command(game_state, &command, &arguments);
                        if let Err(error) = command_result {
                            robot_api_endpoint.send_error(id, error);
                        } else {
                            self.current_command = Some((id, endpoint_index));
                            break 'next_endpoint;
                        }
                    }
                    None => break 'next_robot_api_event,
                }
            }
        }

        // this will be needed when running a python VM
        // // swap back the _real_ game state into `game_state`
        // LOCKED_GAME_STATE.with_borrow_mut(|locked_state| {
        //     mem::swap(locked_state, game_state.as_mut());
        // });
    }

    pub(crate) fn process_command(
        game_state: &mut GameState,
        command: &Identifier,
        arguments: &[Value],
    ) -> Result<(), String> {
        match (command.0.as_ref(), arguments) {
            ("draw forward", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.draw_forward(duration as u64)?;
            }
            ("move forward", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.move_forward(duration as u64)?;
            }
            ("turn left", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.turn_left(duration as u64);
            }
            ("turn right", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.turn_right(duration as u64);
            }
            ("color rgb", [red, green, blue]) => {
                let &Value::Float(red) = red else {
                    panic!("wrong argument");
                };
                let &Value::Float(green) = green else {
                    panic!("wrong argument");
                };
                let &Value::Float(blue) = blue else {
                    panic!("wrong argument");
                };
                game_state.color_rgb(red, green, blue);
            }
            (other, _) => {
                return Err(format!("Unknown Command: {other}"));
            }
        };
        Ok(())
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct ScriptingModule {
    api_clients: HashMap<Identifier, Box<dyn ApiClient>>,
}

impl Module for ScriptingModule {
    // fn add_api_client(&mut self, api_client: Box<dyn ApiClient>) {
    //     self.api_clients
    //         .insert(api_client.api_name().clone(), api_client);
    // }

    fn enter_main(&self) {
        //
    }

    fn wake(&self) {
        //
    }
}

const API_NAME: Identifier = Identifier(Cow::Borrowed("robot"));

pub(crate) struct EngineApiClient {
    response: Option<ServerToClientMessage>,
    game_state: Arc<RwLock<GameState>>,
}

impl EngineApiClient {
    pub(crate) fn new(game_state: &Arc<RwLock<GameState>>) -> Self {
        Self {
            response: None,
            game_state: Arc::clone(game_state),
        }
    }
}

// impl ApiClient for EngineApiClient {
//     fn api(&self) -> &runtimes::api::ApiDescriptor {
//         todo!()
//     }

//     fn api_name(&self) -> &Identifier {
//         &API_NAME
//     }

//     fn send_command(
//         &mut self,
//         command: Identifier,
//         _arguments: Vec<runtimes::api::Value>,
//     ) -> MessageId {
//         let command_id = rand::thread_rng().r#gen();
//         let mut game_state = self.game_state.write().unwrap();

//         match command.0.as_ref() {
//             "draw forward" => {
//                 game_state.move_forward(command_id, true).unwrap();
//             }
//             "move forward" => {
//                 game_state.move_forward(command_id, false).unwrap();
//             }
//             "turn left" => {
//                 game_state.turn_left(command_id);
//             }
//             "turn right" => {
//                 game_state.turn_right(command_id);
//             }
//             "color black" => {
//                 game_state.color(command_id, Vec3::new(0.2, 0.2, 0.2));
//             }
//             "color red" => {
//                 game_state.color(command_id, Vec3::new(0.8, 0.2, 0.2));
//             }
//             "color green" => {
//                 game_state.color(command_id, Vec3::new(0.2, 0.8, 0.2));
//             }
//             "color yellow" => {
//                 game_state.color(command_id, Vec3::new(0.8, 0.8, 0.2));
//             }
//             "color blue" => {
//                 game_state.color(command_id, Vec3::new(0.2, 0.2, 0.8));
//             }
//             "color magenta" => {
//                 game_state.color(command_id, Vec3::new(0.8, 0.0, 0.8));
//             }
//             "color cyan" => {
//                 game_state.color(command_id, Vec3::new(0.2, 0.8, 0.8));
//             }
//             "color white" => {
//                 game_state.color(command_id, Vec3::new(0.8, 0.8, 0.8));
//             }
//             other => {
//                 panic!("Unknown Command: {other}");
//             }
//         };

//         command_id
//     }

//     fn poll_response(&mut self) -> Option<ServerToClientMessage> {
//         self.response.take()
//     }
// }

pub fn make_module(vm: &VirtualMachine) -> rustpython_vm::PyRef<rustpython_vm::builtins::PyModule> {
    engine_api::make_module(vm)
}

#[pymodule]
pub mod engine_api {

    #[pyfunction]
    fn get_current_fps() {
        // just forward to a location outside of this macro so that the IDE can assist us
        // super::message(name, args, kwargs, vm)
    }
}
