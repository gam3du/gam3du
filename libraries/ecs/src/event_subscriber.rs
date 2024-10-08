use super::state::State;

pub trait EventSubscriber: Send + Sync {
    fn start(&mut self, _state: &mut State) {}
    fn update(&mut self, _state: &mut State) {}
    fn fixed_update(&mut self, _state: &mut State) {}
    fn render(&mut self, _state: &mut State) {}
    fn stop(&mut self, _state: &mut State) {}
}
