use toolbelt::cgmath::{MetricSpace, Point2, Vector2};
use wgpu::*;
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode, WindowEvent};
use imgui::{Condition, WindowFlags};
use toolbelt::drag::DragState;
use toolbelt::{SimpleCell, cgmath, Rect};
use crate::app::MapType;
use crate::{GLOBALS, Toggle};
use crate::pipeline::{COLOR_TARGET_STATE, ViewportLightGizmoPipeline, ViewportSpritePipeline};
use crate::pipeline::sprite::CanvasSpritePipelineUniforms;
use crate::registry::{RegistryKey, TextureRegistry};
use crate::scene::Scene;


// TODO: make quick viewport rendering as simple as possible e.g. for previews
//       Viewport::new(params...).render(...)
/// One rendered view into a Scene
pub struct Viewport {
    rt_size: (u32, u32),
    rt_key: RegistryKey,
    sprite_pipeline: ViewportSpritePipeline,
    light_gizmo_pipeline: ViewportLightGizmoPipeline,
    /// view offset in sprite space (sprite pixels)
    pub offset: Vector2<f32>,
    /// zoom * sprite pixels = screen pixels
    pub zoom: f32,
    drag_state: DragState<MouseButton>,
    is_hovered: bool,
    pub bounds: Rect<f32>,
    pub light_gizmos_interactable: bool,
    pub gizmo_opacity: f32,
    pub shown_map_type: MapType,
    camera_height: f32,
    pub global_ambient: f32,
    pub global_diffuse: f32,
    pub global_specular: f32,
    pub normalize_intensities: bool,
    scene: SimpleCell<Scene>
}

impl Viewport {
    fn recreate_render_target(size: (u32, u32), old_key: Option<RegistryKey>, registry: &mut TextureRegistry) -> RegistryKey {
        if let Some(key) = old_key {
            registry.remove(key);
        }
        let rt_key = registry.create_texture(size, "viewport render target", COLOR_TARGET_STATE.format, TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING);
        let rt_bg = registry.add_bind_group(BindGroupDescriptor {
            label: Some("viewport render target"),
            layout: &GLOBALS.get().single_texture_bind_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&*registry.find(rt_key).unwrap().view()) },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&GLOBALS.get().rt_sampler) }
            ]
        });
        registry.find_mut(rt_key).unwrap().replace_bind_group_idx(rt_bg);
        rt_key
    }

    pub fn create(size: (u32, u32),
                  scene: &SimpleCell<Scene>,
                  registry: &mut TextureRegistry) -> Self {
        let rt_key = Viewport::recreate_render_target(size, None, registry);

        let sprite_pipeline = ViewportSpritePipeline::new(rt_key);
        let light_gizmo_pipeline = ViewportLightGizmoPipeline::new(rt_key);

        Viewport {
            scene: (*scene).clone(),
            rt_size: size,
            rt_key,
            sprite_pipeline,
            light_gizmo_pipeline,
            offset: Vector2::new(32.0, 32.0),
            zoom: 4.0,
            drag_state: DragState::new(),
            is_hovered: false,
            bounds: Rect::default(),
            light_gizmos_interactable: true,
            gizmo_opacity: 0.02,
            shown_map_type: MapType::Rendered,
            camera_height: 25.0,
            global_ambient: 0.05,
            global_diffuse: 0.475,
            global_specular: 0.475,
            normalize_intensities: true
        }
    }


    pub fn move_offset(&mut self, delta: Vector2<f32>) {
        self.offset += self.scale_screen_to_canvas(delta);
    }


    pub fn zoom(&mut self, zoom_in: bool, snap: bool, precision: bool, mouse_pos: (f32, f32)) {
        let (mx, my) = (mouse_pos.0 - self.bounds.x, mouse_pos.1 - self.bounds.y);
        self.move_offset([-mx, -my].into());

        if snap {
            if precision {
                self.zoom = (self.zoom * 100.0).round() / 100.0;
            }
            else {
                self.zoom = if zoom_in { self.zoom.floor() } else { self.zoom.ceil() };
            }

            let zoom_delta = if precision {
                if self.zoom >= 8.0 { 0.1 } else { 0.01 }
            } else { 1.0 };

            if zoom_in { self.zoom += zoom_delta; }
            else       { self.zoom -= zoom_delta; }
        }
        else {
            if zoom_in {
                self.zoom *= if precision { 1.02 } else { 1.2 };
            }
            else {
                self.zoom /= if precision { 1.02 } else { 1.2 };
            }
        }
        self.zoom = self.zoom.clamp(1.0, 32.0);

        self.move_offset([mx, my].into());
    }


    pub fn handle_event(&mut self, io: &imgui::Io, event: &Event<()>) -> bool {
        let [mouse_x, mouse_y] = io.mouse_pos;
        match event {
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. }, ..
            } => {
                let pressed = *state == ElementState::Pressed;
                if pressed && self.is_hovered {
                    match button {
                        MouseButton::Left => {
                            let light = &mut self.scene.get_mut().lighting.lights[0];
                            if light.gizmo_hovered {
                                light.drag_state.activate(MouseButton::Left, Some([mouse_x, mouse_y]));
                            }
                            return true;
                        },
                        MouseButton::Right => {
                            self.drag_state.activate(MouseButton::Right, Some([mouse_x, mouse_y]));
                            return true;
                        },
                        _ => (),
                    }
                }
                else {
                    self.drag_state.deactivate();
                    self.scene.get_mut().lighting.lights[0].drag_state.deactivate();
                }
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if self.drag_state.active() {
                    let delta = self.drag_state.update([position.x as f32, position.y as f32]).unwrap();
                    self.move_offset(delta);
                    if self.scene.get().lighting.enable_light_parallax {
                        let light = &mut self.scene.get_mut().lighting.lights[0];
                        let ratio = (light.height / self.camera_height) - 1.0;
                        let dp_canvas = self.scale_screen_to_canvas(delta);
                        light.position += dp_canvas * ratio;
                    }
                }

                let light = &mut self.scene.get_mut().lighting.lights[0];
                if light.drag_state.active() {
                    let delta = light.drag_state.update([position.x as f32, position.y as f32]).unwrap();
                    let dp_canvas = self.scale_screen_to_canvas(delta);
                    if io.key_shift {
                        light.height = (light.height - dp_canvas.y).clamp(10.0, 250.0);
                    }
                    if io.key_ctrl {
                        light.color = light.color.with_alpha((light.color.alpha() * (1.0 - delta.y / 100.0)).clamp(0.25, 5.0));
                    }
                    if !io.key_shift && !io.key_ctrl {
                        light.position += dp_canvas;
                    }
                }

                let canvas_pos = self.transform_screen_to_canvas(
                    (position.x as f32 - self.bounds.x, position.y as f32 - self.bounds.y).into());
                let light_distance = canvas_pos.distance(light.position.into());
                light.gizmo_hovered = self.light_gizmos_interactable
                    && light_distance < (6.25 * (light.height / 100.0)).max(2.0);
            }
            Event::WindowEvent { event: WindowEvent::MouseWheel { delta, phase: TouchPhase::Moved, .. }, .. } => {
                if self.is_hovered {
                    let [mx, my] = io.mouse_pos;
                    match delta {
                        MouseScrollDelta::LineDelta(_, v) => {
                            if *v < 0.0 {
                                self.zoom(false, io.key_shift, io.key_ctrl, (mx, my));
                            }
                            else if *v > 0.0 {
                                self.zoom(true, io.key_shift, io.key_ctrl, (mx, my));
                            }
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y < 0.0 {
                                self.zoom(false, io.key_shift, io.key_ctrl, (mx, my));
                            }
                            else if pos.y > 0.0 {
                                self.zoom(true, io.key_shift, io.key_ctrl, (mx, my));
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, state, .. }, .. }, ..
            } => {
                if *state == ElementState::Pressed && *virtual_keycode == Some(VirtualKeyCode::P) {
                    self.scene.get_mut().lighting.enable_light_parallax.toggle();
                }
            }
            _ => {}
        }
        false
    }


    pub fn resize(&mut self, new_size: (u32, u32), registry: &mut TextureRegistry) {
        self.rt_key = Viewport::recreate_render_target(new_size, Some(self.rt_key), registry);
        self.sprite_pipeline.rt_key = self.rt_key;
        self.light_gizmo_pipeline.rt_key = self.rt_key;
        self.rt_size = new_size;
    }


    pub fn draw(&mut self, ui: &imgui::Ui, num: usize, registry: &mut TextureRegistry) {
        let rt = registry.find(self.rt_key).unwrap();
        let [mouse_x, mouse_y] = ui.io().mouse_pos;
        let rt_size = rt.size();
        ui.window(format!("Viewport {}", num+1))
            .position([0.0, 20.0], Condition::FirstUseEver)
            .flags(WindowFlags::MENU_BAR)
            .size([
                      rt_size.0 as f32 * self.zoom,
                      rt_size.1 as f32 * self.zoom / 1.5
                  ], Condition::FirstUseEver)
            .build(|| {
                if let Some(token) = ui.begin_menu_bar() {
                    if let Some(token) = ui.begin_menu(format!("View: {}", self.shown_map_type)) {
                        for map_type in MapType::TYPES {
                            if ui.menu_item_config(map_type.to_string())
                                .selected(self.shown_map_type == map_type)
                                .build()
                            {
                                self.shown_map_type = map_type;
                            }
                        }
                        token.end();
                    }
                    token.end();
                }

                let [x, y] = ui.window_pos();
                let [w, h] = ui.window_size();
                self.bounds = Rect { x, y, w, h };
                if self.bounds.w != rt_size.0 as f32 || self.bounds.h != rt_size.1 as f32 {
                    self.resize((self.bounds.w.abs().floor() as u32, self.bounds.h.abs().floor() as u32), registry);
                }

                ui.get_window_draw_list().add_image(self.rt_key.into(), [x, y], [x+w, y+h]).build();
                self.is_hovered = ui.is_window_hovered() && self.viewport_bounds().test(mouse_x, mouse_y);
            });
    }


    fn rt_size(&self) -> (u32, u32) {
        self.rt_size
    }


    pub fn viewport_bounds(&self) -> Rect<f32> {
        self.bounds.adjusted_by(5.0, 20.0, -10.0, -25.0)
    }


    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, registry: &TextureRegistry) {
        let textures = &self.scene.get().textures;
        let Vector2 { x: scale_x, y: scale_y } = self.scale_screen_to_fb((self.zoom, self.zoom).into());
        let mut matrix = [
            [scale_x * textures.size.0 as f32, 0.0, 0.0, 0.0],
            [0.0, -scale_y * textures.size.1 as f32, 0.0, 0.0],
            [0.0,     0.0, 1.0, 0.0],
            [-1.0+self.offset.x*scale_x, 1.0-self.offset.y*scale_y, 0.0, 1.0],
        ];

        let center_screen = Vector2::new(self.bounds.w / 2.0, self.bounds.h / 2.0);
        let offset_screen = self.scale_canvas_to_screen(self.offset);
        let center_vp = self.scale_screen_to_canvas(center_screen - offset_screen);

        let l = &self.scene.get().lighting.lights[0];
        self.sprite_pipeline.update_uniforms(CanvasSpritePipelineUniforms {
            matrix,
            light_color: *l.color.components_4(),
            light_pos: [l.position.x, l.position.y, l.height, 0.0],
            cam_pos: [center_vp.x, center_vp.y, self.camera_height, 0.0],
            spec_power: 32.0,
            ambient_intensity: self.global_ambient,
            diffuse_intensity: self.global_diffuse,
            specular_intensity: self.global_specular,
            sprite_size: [textures.size.0 as f32, textures.size.1 as f32],
            light_falloff: if l.enable_falloff { l.falloff_exp } else { 0.0 },
            map_view_type: self.shown_map_type as u32,
        });
        self.sprite_pipeline.render(encoder, registry, self.scene.get().textures.bind_group_idx);

        let l = &self.scene.get().lighting.lights[0];

        matrix[3][0] += l.position.x*scale_x;
        matrix[3][1] -= l.position.y*scale_y;
        let light_scale = l.height / 100.0;
        matrix[0][0] *= light_scale;
        matrix[1][1] *= light_scale;

        let mut gizmo_color = l.color;
        gizmo_color.components_4_mut()[3] =
            if l.gizmo_hovered { self.gizmo_opacity * 2.0 } else { self.gizmo_opacity };
        self.light_gizmo_pipeline.update_uniforms(&matrix, gizmo_color);
        self.light_gizmo_pipeline.render(encoder, registry);
    }

    pub fn close(&mut self) {

    }

    #[allow(dead_code)]
    pub fn scale_screen_to_canvas(&self, vec: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(vec.x / self.zoom, vec.y / self.zoom)
    }

    #[allow(dead_code)]
    pub fn scale_canvas_to_screen(&self, vec: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(vec.x * self.zoom, vec.y * self.zoom)
    }

    #[allow(dead_code)]
    pub fn scale_screen_to_fb(&self, vec: Vector2<f32>) -> Vector2<f32> {
        let rt_size = self.rt_size();
        let pixel_ratio_x = 2.0 / rt_size.0 as f32;
        let pixel_ratio_y = 2.0 / rt_size.1 as f32;
        Vector2::new(vec.x * pixel_ratio_x, vec.y * pixel_ratio_y)
    }

    #[allow(dead_code)]
    pub fn scale_fb_to_screen(&self, vec: Vector2<f32>) -> Vector2<f32> {
        let rt_size = self.rt_size();
        let pixel_ratio_x = 2.0 / rt_size.0 as f32;
        let pixel_ratio_y = 2.0 / rt_size.1 as f32;
        Vector2::new(vec.x / pixel_ratio_x, vec.y / pixel_ratio_y)
    }

    #[allow(dead_code)]
    pub fn scale_fb_to_canvas(&self, vec: Vector2<f32>) -> Vector2<f32> {
        self.scale_screen_to_canvas(self.scale_fb_to_screen(vec))
    }

    #[allow(dead_code)]
    pub fn scale_canvas_to_fb(&self, vec: Vector2<f32>) -> Vector2<f32> {
        self.scale_screen_to_fb(self.scale_canvas_to_screen(vec))
    }

    #[allow(dead_code)]
    pub fn transform_screen_to_canvas(&self, pos: Point2<f32>) -> Point2<f32> {
        (pos / self.zoom) - self.offset
    }

    #[allow(dead_code)]
    pub fn transform_canvas_to_screen(&self, pos: Point2<f32>) -> Point2<f32> {
        (pos + self.offset) * self.zoom
    }

    #[allow(dead_code)]
    pub fn transform_screen_to_fb(&self, pos: Point2<f32>) -> Point2<f32> {
        let rt_size = self.rt_size();
        Point2::new(pos.x * 2.0 / rt_size.0 as f32 - 1.0, pos.y * 2.0 / rt_size.1 as f32 - 1.0)
    }

    #[allow(dead_code)]
    pub fn transform_fb_to_screen(&self, pos: Point2<f32>) -> Point2<f32> {
        let rt_size = self.rt_size();
        Point2::new((pos.x + 1.0) / rt_size.0 as f32 * 2.0, (pos.y + 1.0) / rt_size.1 as f32 * 2.0)
    }

    #[allow(dead_code)]
    pub fn transform_canvas_to_fb(&self, pos: Point2<f32>) -> Point2<f32> {
        self.transform_screen_to_fb(self.transform_canvas_to_screen(pos))
    }

    #[allow(dead_code)]
    pub fn transform_fb_to_canvas(&self, pos: Point2<f32>) -> Point2<f32> {
        self.transform_screen_to_canvas(self.transform_fb_to_screen(pos))
    }
}