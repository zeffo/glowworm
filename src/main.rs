use std::time::Duration;

use colorgrad::Color;
mod gamma;
mod modes;
mod adalight;

use gamma::GammaLookup;
use modes::DynamicGradient;
use adalight::Adalight;

struct GlowColor {
    r: u8,
    g: u8,
    b: u8,
}

impl GlowColor {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        GlowColor{r, g, b}
    }
    fn to_color(&self) -> Color {
        Color::from_rgba8(self.r, self.g, self.b, 255)
    }
}

fn main() {
    let start = GlowColor::from_rgb(200, 60, 70);
    let end = GlowColor::from_rgb(250, 110, 151);
    let colors = vec![start.to_color(), end.to_color(), end.to_color(), start.to_color()];
    const LEDS: u16 = 120;
    let mut mode = DynamicGradient::from_colors(colors, LEDS);
    let mut ada = Adalight::new("/dev/ttyACM0", 115200, LEDS, Duration::from_millis(1000), &mut mode); 
    ada.start();
}
