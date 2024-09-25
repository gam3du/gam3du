use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use gam3du_framework::module::Module;
use glam::Vec3;
use rand::Rng;
use runtimes::{
    api::{ApiClientEndpoint, Identifier},
    message::{MessageId, ServerToClientMessage},
};

use crate::GameState;

pub(crate) struct ScriptingModule {
    api_clients: HashMap<Identifier, Box<ApiClientEndpoint>>,
}

impl Module for ScriptingModule {
    // fn add_api_client(&mut self, api_client: Box<ApiClientEndpoint>) {
    //     self.api_clients
    //         .insert(api_client.api().name.clone(), api_client);
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
}

// impl ApiClient for EngineApiClient {
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
