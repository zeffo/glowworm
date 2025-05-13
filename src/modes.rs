pub trait Mode {

    /// "Render" a new frame on the adalight device.
    fn render(&mut self) -> Vec<u8>;

}
