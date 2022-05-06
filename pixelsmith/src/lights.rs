use toolbelt::cgmath::{Point2, Point3};
use winit::event::MouseButton;
use toolbelt::Color;
use toolbelt::drag::DragState;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingInfo {
    pub lights: Vec<Light>,
    pub enable_light_parallax: bool,
    pub global_ambient: f32,
    pub global_diffuse: f32,
    pub global_specular: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub position: Point2<f32>,
    pub height: f32,
    pub color: Color,
    pub gizmo_hovered: bool,
    pub falloff_exp: f32,
    pub enable_falloff: bool,
    #[serde(skip)]
    pub drag_state: DragState<MouseButton>,
    pub diffuse: f32,
    pub specular: f32,
}

impl Light {
    /// Returns the light's position as Point3(x, y, height)
    #[allow(dead_code)]
    pub fn position_3d(&self) -> Point3<f32> { Point3::new(self.position.x, self.position.y, self.height) }
}
