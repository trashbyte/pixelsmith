use std::ops::{Index, IndexMut, Range};
use crate::{slice_max, slice_min};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    RGB, RGBA, HSL, HSLA, HSV, HSVA, Lab, LabA,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    components: [f32; 4],
    space: ColorSpace
}

macro_rules! const_color_fn {
    ($name:ident => RGB($r:literal, $g:literal, $b:literal) HSL($h:literal, $s:literal, $l:literal) HSV($h2:literal, $s2:literal, $v:literal) Lab($lab_l:literal, $lab_a:literal, $lab_b:literal)) => {
        pub const fn $name(space: ColorSpace) -> Color {
            match space {
                ColorSpace::RGB  => Color::from_rgb ($r, $g, $b),
                ColorSpace::RGBA => Color::from_rgba($r, $g, $b, 1.0),
                ColorSpace::HSL  => Color::from_hsl ($h, $s, $l),
                ColorSpace::HSLA => Color::from_hsla($h, $s, $l, 1.0),
                ColorSpace::HSV  => Color::from_hsv ($h2, $s2, $v),
                ColorSpace::HSVA => Color::from_hsva($h2, $s2, $v, 1.0),
                ColorSpace::Lab  => Color::from_lab ($lab_l, $lab_a, $lab_b),
                ColorSpace::LabA => Color::from_laba($lab_l, $lab_a, $lab_b, 1.0),
            }
        }
    }
}


#[rustversion::attr(nightly, feature(split_array))]
impl Color {
    const_color_fn! { black => RGB(0.0, 0.0, 0.0) HSL(0.0, 0.0, 0.0) HSV(0.0, 0.0, 0.0) Lab(0.0, 0.0, 0.0) }
    const_color_fn! { white => RGB(1.0, 1.0, 1.0) HSL(0.0, 0.0, 1.0) HSV(0.0, 0.0, 1.0) Lab(1.0, 0.0, 0.0) }

    pub const fn from_rgb (r: f32, g: f32, b: f32) -> Color { Color { components: [r, g, b, 1.0], space: ColorSpace::RGB } }
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color { Color { components: [r, g, b, a], space: ColorSpace::RGBA } }
    pub const fn from_hsl (h: f32, s: f32, l: f32) -> Color { Color { components: [h, s, l, 1.0], space: ColorSpace::HSL } }
    pub const fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Color { Color { components: [h, s, l, a], space: ColorSpace::HSLA } }
    pub const fn from_hsv (h: f32, s: f32, v: f32) -> Color { Color { components: [h, s, v, 1.0], space: ColorSpace::HSV } }
    pub const fn from_hsva(h: f32, s: f32, v: f32, a: f32) -> Color { Color { components: [h, s, v, a], space: ColorSpace::HSVA } }
    pub const fn from_lab (l: f32, a: f32, b: f32) -> Color { Color { components: [l, a, b, 1.0], space: ColorSpace::Lab } }
    pub const fn from_laba(l: f32, a: f32, b: f32, alpha: f32) -> Color { Color { components: [l, a, b, alpha], space: ColorSpace::LabA } }

    pub fn with_alpha(self, alpha: f32) -> Color {
        let [a, b, c, _] = self.components;
        Self { components: [a, b, c, alpha], ..self }
    }

    pub fn opaque(self) -> Color { self.with_alpha(1.0) }
    pub fn transparent(self) -> Color { self.with_alpha(0.0) }

    pub fn check_alpha(self) -> Option<f32> {
        match self.space {
            ColorSpace::RGB | ColorSpace::HSL | ColorSpace::HSV | ColorSpace::Lab => None,
            ColorSpace::RGBA | ColorSpace::HSLA | ColorSpace::HSVA | ColorSpace::LabA => Some(self.components[3])
        }
    }

    pub fn as_bytes(self) -> [u8; 4] {
        let [a, b, c, d] = self.components;
        [(a*256.0).floor() as u8, (b*256.0).floor() as u8, (c*256.0).floor() as u8, (d*256.0).floor() as u8]
    }

    // stable: unsafe implementation
    #[rustversion::not(nightly)]
    pub fn components_3(&self) -> &[f32; 3] { unsafe { &*(self.components.as_ptr() as *const [f32; 3]) } }
    #[rustversion::not(nightly)]
    pub fn components_3_mut(&mut self) -> &mut [f32; 3] { unsafe { &mut *(self.components.as_mut_ptr() as *mut [f32; 3]) } }

    // nightly: safer implementation with feature(split_array)
    #[rustversion::nightly]
    pub fn components_3(&self) -> &[f32; 3] { &self.components[0..=2].split_array_ref().0 }
    #[rustversion::nightly]
    pub fn components_3_mut(&mut self) -> &mut [f32; 3] { &self.components[0..=2].split_array_mut().0 }

    pub fn components_4(&self) -> &[f32; 4] { &self.components }
    pub fn components_4_mut(&mut self) -> &mut [f32; 4] { &mut self.components }

    /// Converts this Color into a different ColorSpace *in-place*.
    pub fn convert(&mut self, space: ColorSpace) {
        match space {
            ColorSpace::RGB | ColorSpace::RGBA => *self = self.to_rgb(),
            ColorSpace::HSL | ColorSpace::HSLA => *self = self.to_hsl(),
            ColorSpace::HSV | ColorSpace::HSVA => *self = self.to_hsv(),
            ColorSpace::Lab | ColorSpace::LabA => *self = self.to_lab(),
        }
    }

    pub fn to_rgb(&self) -> Color {
        match self.space {
            ColorSpace::RGB | ColorSpace::RGBA => { *self }
            ColorSpace::HSL | ColorSpace::HSLA => {
                let [hue, saturation, lightness, alpha] = self.components;

                let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
                let h_prime = hue * 6.0;
                let x = chroma * (1.0 - (h_prime % 2.0 - 1.0).abs());

                let (r1, b1, g1) = {
                    if      h_prime <  1.0 { (chroma, x, 0.0) }
                    else if h_prime <  2.0 { (x, chroma, 0.0) }
                    else if h_prime <  3.0 { (0.0, chroma, x) }
                    else if h_prime <  4.0 { (0.0, x, chroma) }
                    else if h_prime <  5.0 { (x, 0.0, chroma) }
                    else if h_prime <= 6.0 { (chroma, 0.0, x) }
                    else { unreachable!() }
                };
                let m = lightness - (chroma / 2.0);
                Color::from_rgba(r1 + m, g1 + m, b1 + m, alpha)
            }
            ColorSpace::HSV | ColorSpace::HSVA => {
                let [hue, saturation, value, alpha] = self.components;

                let chroma = value * saturation;
                let h_prime = hue * 6.0;
                let x = chroma * (1.0 - (h_prime % 2.0 - 1.0).abs());

                let (r1, b1, g1) = {
                    if      h_prime <  1.0 { (chroma, x, 0.0) }
                    else if h_prime <  2.0 { (x, chroma, 0.0) }
                    else if h_prime <  3.0 { (0.0, chroma, x) }
                    else if h_prime <  4.0 { (0.0, x, chroma) }
                    else if h_prime <  5.0 { (x, 0.0, chroma) }
                    else if h_prime <= 6.0 { (chroma, 0.0, x) }
                    else { unreachable!() }
                };
                let m = value - chroma;
                Color::from_rgba(r1 + m, g1 + m, b1 + m, alpha)
            }
            ColorSpace::Lab | ColorSpace::LabA => {
                todo!()
            }
        }
    }

    pub fn to_hsv(&self) -> Color {
        match self.space {
            ColorSpace::HSL | ColorSpace::HSLA => { todo!() }
            ColorSpace::HSV | ColorSpace::HSVA => { *self }
            ColorSpace::RGB | ColorSpace::RGBA => {
                let [r, g, b, alpha] = self.components;

                let max = slice_max(&[r, g, b]);
                let min = slice_min(&[r, g, b]);
                let chroma = max - min;

                let value = max;

                let hue = if chroma < f32::EPSILON { 0.0 } else {
                    if      max == r { (g - b) / chroma       }
                    else if max == g { (b - r) / chroma + 2.0 }
                    else if max == b { (r - g) / chroma + 4.0 }
                    else { unreachable!() }
                };

                let saturation = if value < f32::EPSILON { 0.0 } else { chroma / value };

                Color::from_hsva(hue / 6.0, saturation, value, alpha)
            }
            ColorSpace::Lab | ColorSpace::LabA => { todo!() }
        }
    }

    pub fn to_hsl(&self) -> Color {
        match self.space {
            ColorSpace::HSL | ColorSpace::HSLA => { *self }
            ColorSpace::HSV | ColorSpace::HSVA => {
                let [hue, s_hsv, value, alpha] = self.components;
                let lightness = value * (1.0 - (s_hsv / 2.0));
                let s_hsl = if lightness.min(1.0 - lightness) < f32::EPSILON { 0.0 }
                else { (value - lightness) / (lightness.min(1.0 - lightness)) };
                Color::from_hsla(hue, s_hsl, lightness, alpha)
            }
            ColorSpace::RGB | ColorSpace::RGBA => {
                let [r, g, b, alpha] = self.components;

                let max = slice_max(&[r, g, b]);
                let min = slice_min(&[r, g, b]);
                let chroma = max - min;

                let lightness = (max - min) / 2.0;

                let hue = if chroma < f32::EPSILON { 0.0 } else {
                    if      max == r { (g - b) / chroma       }
                    else if max == g { (b - r) / chroma + 2.0 }
                    else if max == b { (r - g) / chroma + 4.0 }
                    else { unreachable!() }
                };

                let saturation = if lightness.min(1.0 - lightness) < f32::EPSILON { 0.0 }
                                 else { (max - lightness) / lightness.min(1.0 - lightness) };

                Color::from_hsla(hue / 6.0, saturation, lightness, alpha)
            }
            ColorSpace::Lab | ColorSpace::LabA => {
                todo!()
            }
        }
    }

    pub fn to_lab(&self) -> Color {
        todo!()
    }

    // TODO: space conversions
    // TODO: linear <-> srgb conversions
}

impl Index<usize> for Color {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 1 | 2 | 3 => {
                &self[index]
            }
            _ => panic!("Index {} out of bounds for {:?}", index, self)
        }
    }
}

impl IndexMut<usize> for Color {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 1 | 2 | 3 => {
                &mut self[index]
            }
            _ => panic!("Index {} out of bounds for {:?}", index, self)
        }
    }
}

impl Index<Range<usize>> for Color {
    type Output = f32;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        if index.start.max(index.end) <= 3 {
            &self[index]
        }
        else {
            panic!("Index {}..{} out of bounds for {:?}", index.start, index.end, self);
        }
    }
}

impl IndexMut<Range<usize>> for Color {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        if index.start.max(index.end) <= 3 {
            &mut self[index]
        }
        else {
            panic!("Index {}..{} out of bounds for {:?}", index.start, index.end, self);
        }
    }
}
