use super::state::State;
use std::any::Any;

pub trait Component: Send + Sync + Any {
    fn start(&mut self, _state: &mut State) {}

    fn update(&mut self, _state: &mut State) {}

    fn fixed_update(&mut self, _state: &mut State) {}

    fn render(&mut self, _state: &mut State) {}

    fn stop(&mut self, _state: &mut State) {}
}

/*
impl<T> Component for T where T: Component
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}*/
