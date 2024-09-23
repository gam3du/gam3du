pub trait EngineApi {
    fn move_forward(&mut self, duration: u64) -> Result<(), String>;
    fn draw_forward(&mut self, duration: u64) -> Result<(), String>;
    fn turn_left(&mut self, duration: u64);
    fn turn_right(&mut self, duration: u64);
    fn color_rgb(&mut self, red: f32, green: f32, blue: f32);
}
