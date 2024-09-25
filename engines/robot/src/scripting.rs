use std::{
    borrow::Cow,
    num::NonZeroU128,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use rand::{thread_rng, Rng};

use runtimes::{
    api::{ApiServerEndpoint, Identifier, Value},
    message::{ClientToServerMessage, MessageId, RequestMessage},
};

use crate::{api::EngineApi, events::GameEvent, GameState};

// this will be needed when running a python VM
// thread_local! {
//     pub(crate) static LOCKED_GAME_STATE: RefCell<GameState> = RefCell::default();
// }

const ROBOT_API_NAME: Identifier = Identifier(Cow::Borrowed("robot"));

pub struct Plugin {
    id: NonZeroU128,
    robot_controllers: Vec<ApiServerEndpoint>,
    current_command: Option<(MessageId, usize)>,
    sender: Sender<GameEvent>,
    receiver: Receiver<GameEvent>,
}

impl Plugin {
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            id: thread_rng().r#gen(),
            robot_controllers: Vec::new(),
            current_command: None,
            sender,
            receiver,
        }
    }

    pub fn add_robot_controller(&mut self, robot_controller: ApiServerEndpoint) {
        let api_name = robot_controller.api_name();
        assert_eq!(
            api_name, &ROBOT_API_NAME,
            "expected api server for the 'robot' api, but '{api_name}' was given"
        );
        self.robot_controllers.push(robot_controller);
    }

    pub(crate) fn init(
        &mut self,
        game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>,
    ) {
        game_state
            .event_registries
            .robot_stopped
            .subscribe(self.id, self.sender.clone());
    }

    pub(crate) fn update(
        &mut self,
        game_state: &mut std::sync::RwLockWriteGuard<'_, Box<GameState>>,
    ) {
        'next_event: loop {
            match self.receiver.try_recv() {
                Ok(GameEvent::RobotStopped) => {
                    if let Some((command_id, controller_index)) = self.current_command.take() {
                        self.robot_controllers[controller_index]
                            .send_response(command_id, Value::Boolean(true));
                    }
                }
                Err(TryRecvError::Empty) => {
                    break 'next_event;
                }
                Err(TryRecvError::Disconnected) => {
                    todo!();
                }
            }
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

                        match Self::process_command(game_state, &command, &arguments) {
                            Ok(Some(())) => {
                                robot_api_endpoint.send_response(id, Value::Null);
                            }
                            Ok(None) => {
                                self.current_command = Some((id, endpoint_index));
                                break 'next_endpoint;
                            }
                            Err(error) => {
                                robot_api_endpoint.send_error(id, error);
                            }
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
    ) -> Result<Option<()>, String> {
        match (command.0.as_ref(), arguments) {
            ("draw forward", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.draw_forward(duration as u64)?;
                Ok(None)
            }
            ("move forward", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.move_forward(duration as u64)?;
                Ok(None)
            }
            ("turn left", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.turn_left(duration as u64);
                Ok(None)
            }
            ("turn right", [duration]) => {
                let &Value::Integer(duration) = duration else {
                    panic!("wrong argument");
                };
                game_state.turn_right(duration as u64);
                Ok(None)
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
                Ok(Some(()))
            }
            (other, _) => Err(format!("Unknown Command: {other}")),
        }
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}

// const API_NAME: Identifier = Identifier(Cow::Borrowed("robot"));

// pub(crate) struct EngineApiClient {
//     response: Option<ServerToClientMessage>,
//     game_state: Arc<RwLock<GameState>>,
// }

// impl EngineApiClient {
//     pub(crate) fn new(game_state: &Arc<RwLock<GameState>>) -> Self {
//         Self {
//             response: None,
//             game_state: Arc::clone(game_state),
//         }
//     }
// }

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

// pub fn make_module(vm: &VirtualMachine) -> rustpython_vm::PyRef<rustpython_vm::builtins::PyModule> {
//     engine_api::make_module(vm)
// }

// #[pymodule]
// pub mod engine_api {

//     #[pyfunction]
//     fn get_current_fps() {
//         // just forward to a location outside of this macro so that the IDE can assist us
//         // super::message(name, args, kwargs, vm)
//     }
// }
