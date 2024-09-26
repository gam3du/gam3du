pub trait EngineApi {
    fn move_forward(&mut self, duration: u64) -> bool;
    fn draw_forward(&mut self, duration: u64) -> bool;
    fn turn_left(&mut self, duration: u64);
    fn turn_right(&mut self, duration: u64);
    fn robot_color_rgb(&mut self, red: f32, green: f32, blue: f32);
    fn paint_tile(&mut self);
}
