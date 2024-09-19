use runtimes::api::ApiClient;

pub trait Module {
    fn add_api_client(&mut self, api_client: Box<dyn ApiClient>);
    fn enter_main(&self);
    fn wake(&self);
}
