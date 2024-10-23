use std::fs::read_to_string;
use std::time::Duration;

use colorgrad::Color;
mod adalight;
mod gamma;
mod modes;

use adalight::Adalight;
use gamma::GammaLookup;
use modes::{Ambient, AmbientAlgorithm, LEDConfig};

struct GlowColor {
    r: u8,
    g: u8,
    b: u8,
}

#[allow(dead_code)]
impl GlowColor {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        GlowColor { r, g, b }
    }
    fn to_color(&self) -> Color {
        Color::from_rgba8(self.r, self.g, self.b, 255)
    }
}

fn main() {
    // let start = GlowColor::from_rgb(119, 142, 217);
    // let end = GlowColor::from_rgb(219, 99, 235);
    // let colors = vec![start.to_color(), end.to_color(), end.to_color(), start.to_color()];
    const LEDS: u16 = 120;
    // let mut mode = DynamicGradient::from_colors(colors, LEDS);

    let conf = read_to_string("/home/aman/.config/glowworm/config.json").unwrap();
    let config: LEDConfig = serde_json::from_str(&conf).unwrap();
    let mut ada = Adalight::new(
        "/dev/ttyACM0",
        115200,
        LEDS,
        Duration::from_millis(1000),
        180,
    );
    let mut mode = Ambient::new(config, AmbientAlgorithm::Samples, ada);
    mode.start();
}
