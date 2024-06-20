use super::{state::State, Application};

pub trait EventSubscriber: Send + Sync {
    fn start       (&mut self, state: &mut State) { }
    fn update      (&mut self, state: &mut State) { }
    fn fixed_update(&mut self, state: &mut State) { }
    fn render      (&mut self, state: &mut State) { }
    fn stop        (&mut self, state: &mut State) { }
}