use cgmath::{Vector3, Matrix4, Deg, Point3, dot, EuclideanSpace, Transform as CgTransform};
use num::traits::real::Real;

pub mod aabb;

pub mod color;
pub use color::{Color, ColorSpace};

pub mod noise;

pub mod transform;
pub use transform::Transform;

pub mod paths;

pub mod curve;

pub mod rect;
pub use rect::Rect;

pub mod once;



#[derive(Copy, Clone, Debug)]
pub struct Plane {
    n: Vector3<f32>,
    d: f32
}

#[derive(Clone, Debug)]
pub struct FrustumPlanes {
    pub left: Plane,
    pub right: Plane,
    pub bottom: Plane,
    pub top: Plane,
    pub front: Plane,
    pub rear: Plane,
}


pub fn view_to_frustum(pitch: f32, yaw: f32, fov: Deg<f32>, aspect: f32, near_z: f32, far_z: f32) -> FrustumPlanes {
    let forward = Vector3::new(0.0, 0.0, 1.0);

    let neg_yaw = -yaw + 180.0 - 90.0 - fov.0 + 5.0;
    let pos_yaw = -yaw + 180.0 + 90.0 + fov.0 - 5.0;
    let neg_pitch = -pitch - 90.0 - (fov.0 / aspect) + 5.0;
    let pos_pitch = -pitch + 90.0 + (fov.0 / aspect) - 5.0;

    let norm_left = (Matrix4::from_angle_y(Deg(pos_yaw)) * Matrix4::from_angle_x(Deg(pitch))).transform_vector(forward);
    let norm_right = (Matrix4::from_angle_y(Deg(neg_yaw)) * Matrix4::from_angle_x(Deg(pitch))).transform_vector(forward);

    let norm_bottom = (Matrix4::from_angle_y(Deg(-yaw + 180.0)) * Matrix4::from_angle_x(Deg(neg_pitch))).transform_vector(forward);
    let norm_top    = (Matrix4::from_angle_y(Deg(-yaw + 180.0)) * Matrix4::from_angle_x(Deg(pos_pitch))).transform_vector(forward);

    let left   = Plane { n: norm_left,   d: 0.0 };
    let right  = Plane { n: norm_right,  d: 0.0 };
    let bottom = Plane { n: norm_bottom, d: 0.0 };
    let top    = Plane { n: norm_top,    d: 0.0 };
    let front  = Plane { n: Vector3::new(0.0, 0.0, -1.0), d: near_z };
    let rear   = Plane { n: Vector3::new(0.0, 0.0,  1.0), d: far_z };

    FrustumPlanes { left, right, bottom, top, front, rear }
}

pub fn lerp(a: f32, b: f32, alpha: f32) -> f32 {
    (a * (1.0 - alpha)) + (b * alpha)
}

pub fn aabb_plane_intersection(bmin: Point3<f32>, bmax: Point3<f32>, plane: Plane) -> bool {
    // Convert AABB to center-extents representation
    let center = (bmax + bmin.to_vec()) * 0.5; // Compute AABB center
    let extents = bmax - center.to_vec(); // Compute positive extents

    // Compute the projection interval radius of b onto L(t) = center + t * normal
    let proj_int_radius = extents.x*((plane.n.x).abs()) + extents.y*((plane.n.y).abs()) + extents.z*((plane.n.z).abs());

    // Compute distance of box center from plane
    let dist = dot(plane.n, center.to_vec()) - plane.d;

    // Intersection occurs when distance s falls within [-r,+r] interval
    dist.abs() <= proj_int_radius
}

pub fn aabb_frustum_intersection(bmin: Point3<f32>, bmax: Point3<f32>, p: FrustumPlanes) -> bool {
    for plane in &[p.left, p.right, p.top, p.bottom] {
        let mut closest_pt = Vector3::new(0.0, 0.0, 0.0);

        closest_pt.x = if plane.n.x > 0.0 { bmin.x } else { bmax.x };
        closest_pt.y = if plane.n.y > 0.0 { bmin.y } else { bmax.y };
        closest_pt.z = if plane.n.z > 0.0 { bmin.z } else { bmax.z };

        if dot(plane.n, closest_pt) > 0.0 {
            return false;
        }
    }
    true
}

pub fn point_box_intersection(point: [f32; 2], box_mins: [f32; 2], box_maxes: [f32; 2]) -> bool {
    point[0] >= box_mins[0] && point[0] <= box_maxes[0] && point[1] >= box_mins[1] && point[1] <= box_maxes[1]
}

pub fn format_bytes(bytes: u32, digits: u32) -> String {
    if bytes < 1024 {
        let s = bytes.to_string();
        if s.len() > digits as usize {
            if s.chars().nth(3).unwrap() == '.' { s[0..3].to_string() + " B" }
            else { s[0..4].to_string() + " B" }
        }
        else { s + " B" }
    }
    else if bytes < 1024*1024 {
        let s = (bytes as f32 / 1024.0).to_string();
        if s.len() > digits as usize {
            if s.chars().nth(3).unwrap() == '.' { s[0..3].to_string() + " kB" }
            else { s[0..4].to_string() + " kB" }
        }
        else { s + " kB" }
    }
    else {
        let s = (bytes as f32 / (1024.0*1024.0)).to_string();
        if s.len() > digits as usize {
            if s.chars().nth(3).unwrap() == '.' { s[0..3].to_string() + " MB" }
            else { s[0..4].to_string() + " MB" }
        }
        else { s + " MB" }
    }
}

/// Normalizes a 3-vector with one value that stays constant.
/// e.g. (*0.6*, 0.4, 0.8) => (*0.6*, 0.2, 0.4)
/// Only tested with all-positive vectors, may misbehave when negatives are involved.
pub fn normalize_with_constant(constant: f32, mut a: f32, mut b: f32) -> (f32, f32, f32) {
    let difference = 1.0 - (constant + a + b);
    let mut ratio_a = (a / (a + b)).max(0.001);
    let mut ratio_b = (b / (a + b)).max(0.001);
    if ratio_a.is_infinite() || ratio_a.is_nan() || ratio_b.is_infinite() || ratio_b.is_nan() {
        ratio_a = 0.5;
        ratio_b = 0.5;
    }
    let da = difference * ratio_a;
    let db = difference * ratio_b;

    if a < 0.0 {
        a = (a + da).clamp(-1.0, 0.0);
    }
    else {
        a = (a + da).clamp(0.0, 1.0);
    }

    if b < 0.0 {
        b = (b + db).clamp(-1.0, 0.0);
    }
    else {
        b = (b + db).clamp(0.0, 1.0);
    }

    (constant, a, b)
}

pub fn slice_max<T: Real>(slice: &[T]) -> T {
    if slice.len() == 0 { panic!("Can't get the maximum of an empty slice!") }
    let mut max = slice[0];
    for i in 1..slice.len() {
        max = max.max(slice[i]);
    }
    max
}

pub fn slice_min<T: Real>(slice: &[T]) -> T {
    if slice.len() == 0 { panic!("Can't get the minimum of an empty slice!") }
    let mut min = slice[0];
    for i in 1..slice.len() {
        min = min.min(slice[i]);
    }
    min
}

pub fn array_max<const N: usize, T: Real>(array: [T; N]) -> T {
    if N == 0 { panic!("Can't get the maximum of an empty array!") }
    let mut max = array[0];
    for i in 1..N {
        max = max.max(array[i]);
    }
    max
}

pub fn array_min<const N: usize, T: Real>(array: [T; N]) -> T {
    if N == 0 { panic!("Can't get the minimum of an empty array!") }
    let mut min = array[0];
    for i in 1..N {
        min = min.min(array[i]);
    }
    min
}