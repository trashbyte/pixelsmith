//! Axis-aligned bounding box.
//!
//! Represented by a cuboid defined by two points. As long as the
//! `set_*` functions are used, the `lower` point will be less than or equal to the `upper` point
//! for any axis.


use cgmath::Point3;


// local min/max funcs for f32 since it isn't Ord and doesn't work with std::min/max
fn float_min(a: f32, b: f32) -> f32 { if a < b { a } else { b } }
fn float_max(a: f32, b: f32) -> f32 { if a > b { a } else { b } }


/// An axis-aligned bounding box. Represented by a cuboid defined by two points. As long as the
/// `set_*` functions are used, the `lower` point will be less than or equal to the `upper` point
/// for any axis.
pub struct AABB {
    lower: Point3<f32>,
    upper: Point3<f32>,
}


#[allow(dead_code)]
impl AABB {
    /// Constructs a new AABB of size zero.
    pub fn new() -> AABB {
        AABB {
            lower: Point3::new(0.0, 0.0, 0.0),
            upper: Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Constructs a new AABB with the given points. This method does not ensure `lower` <= `upper`
    /// for all axes.
    pub fn from(lower: Point3<f32>, upper: Point3<f32>) -> AABB {
        AABB { lower, upper }
    }

    /// Returns the length of the AABB in the x dimension.
    pub fn size_x(&self) -> f32 { self.upper.x - self.lower.x }
    /// Returns the length of the AABB in the y dimension.
    pub fn size_y(&self) -> f32 { self.upper.y - self.lower.y }
    /// Returns the length of the AABB in the z dimension.
    pub fn size_z(&self) -> f32 { self.upper.z - self.lower.z }

    /// Returns the x coordinate of the lower point, representing the left side of the AABB.
    pub fn left(&self) -> f32 { self.lower.x }
    /// Returns the x coordinate of the upper point, representing the right side of the AABB.
    pub fn right(&self) -> f32 { self.upper.x }
    /// Returns the y coordinate of the lower point, representing the top side of the AABB.
    pub fn top(&self) -> f32 { self.lower.y }
    /// Returns the y coordinate of the upper point, representing the bottom side of the AABB.
    pub fn bottom(&self) -> f32 { self.upper.y }
    /// Returns the z coordinate of the lower point, representing the front side of the AABB.
    pub fn front(&self) -> f32 { self.lower.z }
    /// Returns the z coordinate of the upper point, representing the back side of the AABB.
    pub fn back(&self) -> f32 { self.upper.z }


    /// Updates the lower point. Rearranges the coordinates to assure that `lower` <= `upper` for
    /// all axes.
    pub fn set_lower(&mut self, lower: Point3<f32>) {
        let (x1, y1, z1) = lower.into();
        let (x2, y2, z2) = self.upper.into();
        self.lower = Point3::new(float_min(x1, x2), float_min(y1, y2), float_min(z1, z2));
        self.upper = Point3::new(float_max(x1, x2), float_max(y1, y2), float_max(z1, z2));
    }
    /// Updates the upper point. Rearranges the coordinates to assure that `lower` <= `upper` for
    /// all axes.
    pub fn set_upper(&mut self, upper: Point3<f32>) {
        let (x1, y1, z1) = self.lower.into();
        let (x2, y2, z2) = upper.into();
        self.lower = Point3::new(float_min(x1, x2), float_min(y1, y2), float_min(z1, z2));
        self.upper = Point3::new(float_max(x1, x2), float_max(y1, y2), float_max(z1, z2));
    }


    /// Updates the x coordinate of the lower point (the left side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_left(&mut self, left: f32) {
        let x1 = left;
        let x2 = self.upper.x;
        self.lower.x = float_min(x1, x2);
        self.upper.x = float_max(x1, x2);
    }
    /// Updates the x coordinate of the upper point (the right side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_right(&mut self, right: f32) {
        let x1 = self.lower.x;
        let x2 = right;
        self.lower.x = float_min(x1, x2);
        self.upper.x = float_max(x1, x2);
    }
    /// Updates the y coordinate of the lower point (the bottom side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_bottom(&mut self, bottom: f32) {
        let y1 = bottom;
        let y2 = self.upper.y;
        self.lower.y = float_min(y1, y2);
        self.upper.y = float_max(y1, y2);
    }
    /// Updates the y coordinate of the upper point (the top side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_top(&mut self, top: f32) {
        let y1 = self.lower.y;
        let y2 = top;
        self.lower.y = float_min(y1, y2);
        self.upper.y = float_max(y1, y2);
    }
    /// Updates the z coordinate of the lower point (the front side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_front(&mut self, front: f32) {
        let z1 = front;
        let z2 = self.upper.z;
        self.lower.z = float_min(z1, z2);
        self.upper.z = float_max(z1, z2);
    }
    /// Updates the z coordinate of the upper point (the back side of the AABB). Ensures that
    /// `lower` <= `upper` for all axes.
    pub fn set_back(&mut self, back: f32) {
        let z1 = self.lower.z;
        let z2 = back;
        self.lower.z = float_min(z1, z2);
        self.upper.z = float_max(z1, z2);
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            lower: Point3::new(0.0, 0.0, 0.0),
            upper: Point3::new(0.0, 0.0, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AABB;
    use cgmath::Point3;

    macro_rules! assert_eq_float {
        ($a:expr, $b:expr) => { assert!((($a) - ($b)).abs() < std::f32::EPSILON) }
    }

    #[test]
    fn test_aabb_from() {
        let b = AABB::from(Point3::new(1.0, 3.0, 5.0), Point3::new(6.0, 4.0, 2.0));
        assert_eq!(b.lower, Point3::new(1.0, 3.0, 5.0));
        assert_eq!(b.upper, Point3::new(6.0, 4.0, 2.0));
        assert_eq_float!(b.left(), 1.0);
        assert_eq_float!(b.right(), 6.0);
        assert_eq_float!(b.top(), 3.0);
        assert_eq_float!(b.bottom(), 4.0);
        assert_eq_float!(b.front(), 5.0); // note that these are in the wrong order
        assert_eq_float!(b.back(), 2.0); // AABB::from() does not check point ordering
    }
}
