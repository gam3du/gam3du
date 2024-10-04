use crate::{api::EngineApi, events::GameEvent, GameState};
use gam3du_framework_common::{
    api::{ApiServerEndpoint, Identifier, Value},
    message::{ClientToServerMessage, PendingResult, RequestId, RequestMessage},
};
use glam::Vec3;
use rand::{thread_rng, Rng};
use std::{
    fmt::{self, Display},
    num::NonZeroU128,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use super::Plugin;

const ROBOT_API_NAME: &str = "robot";
const CMD_MOVE_FORWARD: &str = "move forward";
const CMD_DRAW_FORWARD: &str = "draw forward";
const CMD_TURN: &str = "turn";
const CMD_PAINT_TILE: &str = "paint tile";
const CMD_ROBOT_COLOR_RGB: &str = "robot color rgb";

enum CommandError {
    UnknownCommand(Identifier),
    MissingArgument(Identifier, &'static str),
    WrongArgumentType(Identifier, &'static str),
}

impl Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(command) => {
                write!(formatter, "Unknown Command: {command}")
            }
            Self::MissingArgument(command, parameter) => {
                write!(
                    formatter,
                    "Missing `{parameter}` argument for command `{command}`"
                )
            }
            Self::WrongArgumentType(command, parameter) => {
                write!(
                    formatter,
                    "Argument `{parameter}` for command `{command}` has wrong type"
                )
            }
        }
    }
}

impl<T> From<CommandError> for PendingResult<T, CommandError> {
    fn from(value: CommandError) -> Self {
        Self::Error(value)
    }
}

pub struct NativePlugin {
    id: NonZeroU128,
    robot_controllers: Vec<ApiServerEndpoint>,
    current_command: Option<(RequestId, usize)>,
    sender: Sender<GameEvent>,
    receiver: Receiver<GameEvent>,
}

impl NativePlugin {
    #[must_use]
    #[expect(
        clippy::new_without_default,
        reason = "the parameters are unlikely to remain empty"
    )]
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
        let api_name = robot_controller.api().name.0.as_ref();
        assert_eq!(
            api_name, ROBOT_API_NAME,
            "expected api server for the 'robot' api, but '{api_name}' was given"
        );
        self.robot_controllers.push(robot_controller);
    }
}

impl Plugin for NativePlugin {
    fn init(&mut self, game_state: &mut GameState) {
        game_state
            .event_registries
            .robot_stopped
            .subscribe(self.id, self.sender.clone());
    }

    fn update(&mut self, game_state: &mut GameState) {
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

        'next_endpoint: for (endpoint_index, robot_api_endpoint) in
            self.robot_controllers.iter_mut().enumerate()
        {
            'next_robot_api_event: loop {
                match robot_api_endpoint.poll_request() {
                    Some(ClientToServerMessage::Request(request)) => {
                        let RequestMessage {
                            id,
                            command,
                            arguments,
                        } = request;

                        match run_command(game_state, command, arguments) {
                            PendingResult::Ok(value) => {
                                robot_api_endpoint.send_response(id, value);
                            }
                            PendingResult::Pending => {
                                self.current_command = Some((id, endpoint_index));
                                break 'next_endpoint;
                            }
                            PendingResult::Error(error) => {
                                robot_api_endpoint.send_error(id, error.to_string());
                            }
                        }
                    }
                    None => break 'next_robot_api_event,
                }
            }
        }
    }
}

fn run_command(
    game_state: &mut GameState,
    command: Identifier,
    mut arguments: Vec<Value>,
) -> PendingResult<Value, CommandError> {
    let mut arguments = arguments.drain(..);

    match command.0.as_ref() {
        CMD_DRAW_FORWARD => {
            let Some(duration) = arguments.next() else {
                return CommandError::MissingArgument(command, "duration").into();
            };

            let Value::Integer(duration) = duration else {
                return CommandError::WrongArgumentType(command, "duration").into();
            };

            if game_state.move_forward(true, duration.try_into().unwrap()) {
                PendingResult::Pending
            } else {
                PendingResult::Ok(Value::Boolean(false))
            }
        }
        CMD_MOVE_FORWARD => {
            let Some(duration) = arguments.next() else {
                return CommandError::MissingArgument(command, "duration").into();
            };

            let Value::Integer(duration) = duration else {
                return CommandError::WrongArgumentType(command, "duration").into();
            };

            if game_state.move_forward(false, duration.try_into().unwrap()) {
                PendingResult::Pending
            } else {
                PendingResult::Ok(Value::Boolean(false))
            }
        }
        CMD_TURN => {
            let Some(steps_ccw) = arguments.next() else {
                return CommandError::MissingArgument(command, "steps_ccw").into();
            };

            let Value::Integer(steps_ccw) = steps_ccw else {
                return CommandError::WrongArgumentType(command, "steps_ccw").into();
            };

            let Ok(steps_ccw) = steps_ccw.try_into() else {
                return CommandError::WrongArgumentType(command, "steps_ccw").into();
            };

            let Some(duration) = arguments.next() else {
                return CommandError::MissingArgument(command, "duration").into();
            };

            let Value::Integer(duration) = duration else {
                return CommandError::WrongArgumentType(command, "duration").into();
            };

            game_state.turn(steps_ccw, duration.try_into().unwrap());
            PendingResult::Pending
        }
        CMD_ROBOT_COLOR_RGB => {
            let Some(red) = arguments.next() else {
                return CommandError::MissingArgument(command, "red").into();
            };

            let Value::Float(red) = red else {
                return CommandError::WrongArgumentType(command, "red").into();
            };

            let Some(green) = arguments.next() else {
                return CommandError::MissingArgument(command, "green").into();
            };

            let Value::Float(green) = green else {
                return CommandError::WrongArgumentType(command, "green").into();
            };

            let Some(blue) = arguments.next() else {
                return CommandError::MissingArgument(command, "blue").into();
            };

            let Value::Float(blue) = blue else {
                return CommandError::WrongArgumentType(command, "blue").into();
            };

            game_state.robot.color = Vec3::new(red, green, blue);
            PendingResult::Ok(Value::Unit)
        }
        CMD_PAINT_TILE => {
            game_state.paint_tile();
            PendingResult::Ok(Value::Unit)
        }
        _ => CommandError::UnknownCommand(command).into(),
    }
}
