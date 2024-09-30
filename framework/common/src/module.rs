pub trait Module {
    fn enter_main(&mut self);
    fn wake(&mut self);
}
