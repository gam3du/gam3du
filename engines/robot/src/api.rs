pub trait EngineApi {
    fn set_height(&mut self, height: f32);
    fn move_forward(&mut self, draw: bool, duration: u64) -> bool;
    fn jump(&mut self, duration: u64) -> bool;
    fn turn(&mut self, steps_ccw: i8, duration: u64);
    fn robot_color_rgb(&mut self, red: f32, green: f32, blue: f32);
    fn paint_tile(&mut self);
}
