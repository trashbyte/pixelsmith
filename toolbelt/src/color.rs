#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl SrgbColor {
    pub const BLACK:       Self = Self { r:   0, g:   0, b:   0, a: 255 };
    pub const WHITE:       Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r:   0, g:   0, b:   0, a:   0 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl LinearColor {
    pub const BLACK:       Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE:       Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}
