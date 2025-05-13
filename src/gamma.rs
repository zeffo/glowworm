struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

pub struct GammaMap {
    map: Vec<Pixel>,
}

impl GammaMap {
    pub fn new() -> Self {
        Self {
            map: (0_u8..=255)
                .map(|idx| {
                    let f = ((idx as f64) / 255.0).powf(2.8);
                    Pixel {
                        r: (f * 255.0) as u8,
                        g: (f * 240.0) as u8,
                        b: (f * 220.0) as u8,
                    }
                })
                .collect(),
        }
    }

    pub fn red(&self, r: u8) -> u8 {
        self.map[usize::from(r)].r
    }

    pub fn green(&self, g: u8) -> u8 {
        self.map[usize::from(g)].g
    }

    pub fn blue(&self, b: u8) -> u8 {
        self.map[usize::from(b)].b
    }

    pub fn correct_rgb(&self, rgb: &mut [u8]) {
        rgb[0] = self.red(rgb[0]);
        rgb[1] = self.blue(rgb[1]);
        rgb[2] = self.green(rgb[2]);
    }
}

impl Default for GammaMap {
    fn default() -> Self {
        Self::new()
    }
}
