use std::collections::HashMap;

use gam3du_framework::module::Module;
use runtimes::api::{ApiClient, Identifier};

pub(crate) struct ScriptingModule {
    api_clients: HashMap<Identifier, Box<dyn ApiClient>>,
}

impl Module for ScriptingModule {
    fn add_api_client(&mut self, api_client: Box<dyn ApiClient>) {
        self.api_clients
            .insert(api_client.api_name().clone(), api_client);
    }

    fn enter_main(&self) {
        //
    }

    fn wake(&self) {
        //
    }
}
