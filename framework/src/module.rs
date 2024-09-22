pub trait Module {
    fn enter_main(&self);
    fn wake(&self);
}
