use colorgrad::{Color, CustomGradient};

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
        let colors = gradient.iter()
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
    colors: Vec<u8>,
    cursor: usize,
}
impl DynamicGradient {
    pub fn from_colors(gradient_colors: Vec<Color>, leds: u16) -> Self {
        let gradient = CustomGradient::new()
            .colors(&gradient_colors)
            .build()
            .unwrap()
            .colors(leds as usize);
        let colors = gradient.iter()
            .flat_map(|c| c.to_rgba8()[..3].to_owned())
            .collect();
        Self { colors, cursor: 0, }
    }
}

impl GlowMode for DynamicGradient {
    fn get_colors(&mut self) -> Vec<u8> {
        let ret = [&self.colors[self.cursor..self.colors.len()], &self.colors[..self.cursor]].concat();
        self.cursor = (self.cursor + 3) % self.colors.len();
        ret
    }
}


