struct GammaValues {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct GammaLookup {
    table: Vec<GammaValues>,
}

impl GammaLookup {
    pub fn new() -> Self {
        Self {
            table: (0_u8..=255)
                .map(|index| {
                    let f = ((index as f64) / 255.0).powf(2.8);
                    GammaValues {
                        r: (f * 255.0) as u8,
                        g: (f * 240.0) as u8,
                        b: (f * 220.0) as u8,
                    }
                })
                .collect(),
        }
    }

    pub fn red(&self, r: u8) -> u8 {
        self.table[usize::from(r)].r
    }

    pub fn green(&self, g: u8) -> u8 {
        self.table[usize::from(g)].g
    }

    pub fn blue(&self, b: u8) -> u8 {
        self.table[usize::from(b)].b
    }

    pub fn correct_rgb(&self, rgb: &mut [u8]) {
        rgb[0] = self.red(rgb[0]);
        rgb[1] = self.blue(rgb[1]);
        rgb[2] = self.green(rgb[2]);
    }

}

impl Default for GammaLookup {
    fn default() -> Self {
        Self::new()
    }
}
