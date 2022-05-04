use num::{Num, NumCast};

#[derive(Default, Clone, Copy, Debug)]
pub struct Rect<N: Num + NumCast + Copy + PartialOrd> {
    pub x: N,
    pub y: N,
    pub w: N,
    pub h: N,
}

#[inline]
fn _cast<N: NumCast>(value: f32) -> N {
    num::cast::<f32, N>(value).unwrap()
}

impl<N: Num + NumCast + Copy + PartialOrd> Rect<N> {
    pub fn test(&self, pos_x: N, pos_y: N) -> bool {
        pos_x >= self.x && pos_x <= self.x + self.w && pos_y >= self.y && pos_y <= self.y + self.h
    }

    pub fn adjusted_by(&self, dx: N, dy: N, dw: N, dh: N) -> Rect<N> {
        Rect {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w + dw,
            h: self.h + dh
        }
    }

    pub fn retracted_by(&self, retract_x: N, retract_y: N) -> Rect<N> {
        self.adjusted_by(retract_x, retract_y, retract_x * _cast(-2.0), retract_y * _cast(-2.0))
    }

    pub fn expanded_by(&self, expand_x: N, expand_y: N) -> Rect<N> {
        self.adjusted_by(expand_x * _cast(-1.0), expand_x * _cast(-1.0), expand_y * _cast(2.0), expand_y * _cast(2.0))
    }
}
