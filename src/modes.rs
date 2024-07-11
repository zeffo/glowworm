use colorgrad::{Color, CustomGradient};
use libwayshot::{output::OutputInfo, WayshotConnection};
use serde::{Deserialize, Serialize};

pub trait GlowMode {
    fn get_colors(&mut self) -> Vec<u8>;
}

pub struct StaticGradient {
    colors: Vec<u8>,
}

impl StaticGradient {
    pub fn from_colors(gradient_colors: Vec<Color>, leds: u16) -> Self {
        let gradient = CustomGradient::new()
            .colors(&gradient_colors)
            .build()
            .unwrap()
            .colors(leds as usize);
        let colors = gradient
            .iter()
            .flat_map(|c| c.to_rgba8()[..3].to_owned())
            .collect();
        Self { colors }
    }
}

impl GlowMode for StaticGradient {
    fn get_colors(&mut self) -> Vec<u8> {
        self.colors.to_vec()
    }
}

pub struct DynamicGradient {
    colors: Vec<u8>, // can we use a reference here?
    cursor: usize,
}
impl DynamicGradient {
    pub fn from_colors(gradient_colors: Vec<Color>, leds: u16) -> Self {
        let gradient = CustomGradient::new()
            .colors(&gradient_colors)
            .build()
            .unwrap()
            .colors(leds as usize);
        let colors = gradient
            .iter()
            .flat_map(|c| c.to_rgba8()[..3].to_owned())
            .collect();
        Self { colors, cursor: 0 }
    }
}

impl GlowMode for DynamicGradient {
    // TODO: try to return a reference here instead of allocating a new vec on every call
    fn get_colors(&mut self) -> Vec<u8> {
        let ret = [
            &self.colors[self.cursor..self.colors.len()],
            &self.colors[..self.cursor],
        ]
        .concat();
        self.cursor = (self.cursor + 3) % self.colors.len();
        ret
    }
}

#[derive(Serialize, Deserialize)]
pub struct LEDConfig {
    leds: Vec<(u16, u16, u16, u16)>, // vec of capture areas for each LED
}

// TODO: benchmark libwayshot vs other wayland screenshotters
pub struct Ambient<'a> {
    wl_connection: WayshotConnection,
    displays: &'a Vec<OutputInfo>,
    config: &'a LEDConfig,
}

impl<'a> Ambient<'a> {
    pub fn new(displays: &'a Vec<OutputInfo>, config: &'a LEDConfig) -> Self {
        let wl_connection = WayshotConnection::new().unwrap();
        Self {
            wl_connection,
            displays,
            config,
        }
    }
}

impl<'a> GlowMode for Ambient<'a> {
    fn get_colors(&mut self) -> Vec<u8> {
        let screen = self
            .wl_connection
            .screenshot_outputs(self.displays, false)
            .unwrap();
        vec![1, 2, 3]
    }
}
