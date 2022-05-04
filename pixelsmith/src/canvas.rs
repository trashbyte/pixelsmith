use std::sync::Arc;
use cgmath::MetricSpace;
use wgpu::*;
use winit::event::{ElementState, Event, KeyboardInput, MouseScrollDelta, TouchPhase, VirtualKeyCode, WindowEvent};
use imgui::{Condition, WindowFlags};
use imgui_wgpu::{Texture, TextureConfig};
use toolbelt::Rect;
use crate::app::MapType;
use crate::pipeline::{CanvasLightPipeline, CanvasSpritePipeline, SimpleGeometryPipeline};
use crate::pipeline::sprite::CanvasSpritePipelineUniforms;
use crate::registry::{TextureInfo, TextureRegistry};


struct DragState {
    pub prev_pos: (f32, f32),
    pub dragging: bool,
}


pub struct Light {
    pub position: (f32, f32, f32),
    pub color: wgpu::Color,
    pub hovered: bool,
}


// TODO: make quick canvas rendering as simple as possible e.g. for previews
//       Canvas::new(params...).render(...)
pub struct Canvas {
    canvas_size: (u32, u32),
    rt_size: (u32, u32),
    sprite_pipeline: CanvasSpritePipeline,
    light_gizmo_pipeline: CanvasLightPipeline,
    /// view offset in canvas space (canvas pixels)
    pub offset: (f32, f32),
    /// zoom, canvas pixels * zoom = screen pixels
    pub zoom: f32,
    canvas_drag_state: DragState,
    main_light_drag_state: DragState,
    is_hovered: bool,
    pub bounds: Rect<f32>,
    pub main_light: Light,
    rt_tex_id: imgui::TextureId,
    pub light_falloff: f32,
    pub enable_light_falloff: bool,
    pub enable_light_parallax: bool,
    pub light_gizmos_interactable: bool,
    pub gizmo_opacity: f32,
    pub shown_map_type: MapType,
    camera_height: f32,
    pub ambient_intensity: f32,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub normalize_intensity: bool,
    texture_registry: Arc<TextureRegistry>,
}


impl Canvas {
    pub fn create(canvas_size: (u32, u32),
                  rt_size: (u32, u32),
                  device: &Arc<Device>,
                  queue: &Queue,
                  renderer: &mut imgui_wgpu::Renderer,
                  registry: &Arc<TextureRegistry>) -> Self {
        let maps_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("canvas map sampler"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let map_entry_default = BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: BindingType::Texture { multisampled: false, sample_type: TextureSampleType::Float { filterable: false }, view_dimension: TextureViewDimension::D2, },
            count: None,
        };
        let maps_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("canvas maps bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                BindGroupLayoutEntry { binding: 1, ..map_entry_default },
                BindGroupLayoutEntry { binding: 2, ..map_entry_default },
                BindGroupLayoutEntry { binding: 3, ..map_entry_default },
                BindGroupLayoutEntry { binding: 4, ..map_entry_default },
            ],
        });

        let albedo_texture = TextureInfo::new(canvas_size, "canvas albedo texture",
                                              TextureFormat::Rgba8Unorm,
                                              TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                              &device);
        albedo_texture.write(&queue, &image::io::Reader::open("project/sprites/bricks/albedo.png").unwrap().decode().unwrap().to_rgba8().into_vec()[..], canvas_size.0, canvas_size.1);
        registry.add("albedo", &albedo_texture);

        let normal_texture = TextureInfo::new(canvas_size, "canvas normal texture",
                                              TextureFormat::Rgba8Unorm,
                                              TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                              &device);
        normal_texture.write(&queue, &image::io::Reader::open("project/sprites/bricks/normal.png").unwrap().decode().unwrap().to_rgba8().into_vec()[..], canvas_size.0, canvas_size.1);
        registry.add("normal", &normal_texture);

        let roughness_texture = TextureInfo::new(canvas_size, "canvas roughness texture",
                                              TextureFormat::Rgba8Unorm,
                                              TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                              &device);
        roughness_texture.write(&queue, &image::io::Reader::open("project/sprites/bricks/roughness.png").unwrap().decode().unwrap().to_rgba8().into_vec()[..], canvas_size.0, canvas_size.1);
        registry.add("roughness", &roughness_texture);

        let height_texture = TextureInfo::new(canvas_size, "canvas height texture",
                                              TextureFormat::Rgba8Unorm,
                                              TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                              &device);
        height_texture.write(&queue, &image::io::Reader::open("project/sprites/bricks/height.png").unwrap().decode().unwrap().to_rgba8().into_vec()[..], canvas_size.0, canvas_size.1);
        registry.add("height", &height_texture);

        let maps_bind_group = Arc::new(device.create_bind_group(&BindGroupDescriptor {
            label: Some("canvas maps bind group"),
            layout: &maps_bg_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::Sampler(&maps_sampler) },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&*albedo_texture.view()) },
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(&*normal_texture.view()) },
                BindGroupEntry { binding: 3, resource: BindingResource::TextureView(&*roughness_texture.view()) },
                BindGroupEntry { binding: 4, resource: BindingResource::TextureView(&*height_texture.view()) },
            ],
        }));

        albedo_texture.replace_bind_group(&maps_bind_group);
        normal_texture.replace_bind_group(&maps_bind_group);
        roughness_texture.replace_bind_group(&maps_bind_group);
        height_texture.replace_bind_group(&maps_bind_group);

        let rt_texture = Texture::new(device, renderer, TextureConfig {
            size: Extent3d { width: rt_size.0, height: rt_size.1, ..Default::default() },
            label: Some("canvas render target"),
            format: Some(TextureFormat::Rgba8Unorm),
            usage: TextureUsages::all(),
            dimension: wgpu::TextureDimension::D2,
            sampler_desc: SamplerDescriptor {
                label: Some("canvas rt sampler"),
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        });
        let data = vec![127u8; (rt_size.0*rt_size.1) as usize * 4];
        rt_texture.write(&queue, &data[0..((rt_size.0*rt_size.1) as usize * 4)], rt_size.0, rt_size.1);

        let rt_tex_id = renderer.textures.insert(rt_texture);

        let sprite_pipeline = CanvasSpritePipeline::new(rt_tex_id, canvas_size, &maps_bg_layout, &maps_bind_group, &device);
        let light_gizmo_pipeline = CanvasLightPipeline::new(rt_tex_id, &device);

        Canvas {
            canvas_size,
            rt_size,
            sprite_pipeline,
            light_gizmo_pipeline,
            offset: (32.0, 32.0),
            zoom: 4.0,
            canvas_drag_state: DragState { prev_pos: (0.0, 0.0), dragging: false },
            main_light_drag_state: DragState { prev_pos: (0.0, 0.0), dragging: false },
            is_hovered: false,
            bounds: Rect::default(),
            main_light: Light { position: (-16.0, -16.0, 50.0), color: wgpu::Color { a: 2.0, ..wgpu::Color::WHITE }, hovered: false },
            rt_tex_id,
            light_falloff: 1.0,
            enable_light_falloff: true,
            enable_light_parallax: false,
            light_gizmos_interactable: true,
            gizmo_opacity: 0.02,
            shown_map_type: MapType::Rendered,
            camera_height: 25.0,
            ambient_intensity: 0.05,
            diffuse_intensity: 0.475,
            specular_intensity: 0.475,
            normalize_intensity: true,
            texture_registry: registry.clone(),
        }
    }


    pub fn move_offset(&mut self, dx: f32, dy: f32) {
        let dp_canvas = self.scale_screen_to_canvas((dx, dy));
        self.offset = (
            self.offset.0 + dp_canvas.0,
            self.offset.1 + dp_canvas.1
        );
    }


    pub fn zoom(&mut self, zoom_in: bool, snap: bool, precision: bool, mouse_pos: (f32, f32)) {
        let (mx, my) = (mouse_pos.0 - self.bounds.x, mouse_pos.1 - self.bounds.y);
        self.move_offset(-mx, -my);

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

        self.move_offset(mx, my);
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
                        winit::event::MouseButton::Left => {
                            if self.main_light.hovered {
                                self.main_light_drag_state.dragging = true;
                                self.main_light_drag_state.prev_pos = (mouse_x, mouse_y);
                            }
                            return true;
                        },
                        winit::event::MouseButton::Right => {
                            self.canvas_drag_state.dragging = true;
                            self.canvas_drag_state.prev_pos = (mouse_x, mouse_y);
                            return true;
                        },
                        _ => (),
                    }
                }
                else {
                    match button {
                        winit::event::MouseButton::Left => {
                            self.main_light_drag_state.dragging = false;
                        }
                        winit::event::MouseButton::Right => {
                            self.canvas_drag_state.dragging = false;
                        }
                        _ => {}
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if self.canvas_drag_state.dragging {
                    let (dx, dy) = (
                        position.x as f32 - self.canvas_drag_state.prev_pos.0,
                        position.y as f32 - self.canvas_drag_state.prev_pos.1
                    );
                    self.canvas_drag_state.prev_pos = (position.x as f32, position.y as f32);
                    self.move_offset(dx, dy);
                    if self.enable_light_parallax {
                        let ratio = (self.main_light.position.2 / self.camera_height) - 1.0;
                        let dp_canvas = self.scale_screen_to_canvas((dx, dy));
                        self.main_light.position.0 += dp_canvas.0 * ratio;
                        self.main_light.position.1 += dp_canvas.1 * ratio;
                    }
                }
                if self.main_light_drag_state.dragging {
                    let (dx, dy) = (
                        position.x as f32 - self.main_light_drag_state.prev_pos.0,
                        position.y as f32 - self.main_light_drag_state.prev_pos.1
                    );
                    self.main_light_drag_state.prev_pos = (position.x as f32, position.y as f32);
                    let dp_canvas = self.scale_screen_to_canvas((dx, dy));
                    if io.key_shift {
                        self.main_light.position.2 = (self.main_light.position.2 - dp_canvas.1).clamp(10.0, 250.0);
                    }
                    if io.key_ctrl {
                        self.main_light.color.a = (self.main_light.color.a * (1.0 - dy as f64 / 100.0)).clamp(0.25, 5.0);
                    }
                    if !io.key_shift && !io.key_ctrl {
                        self.main_light.position.0 += dp_canvas.0;
                        self.main_light.position.1 += dp_canvas.1;
                    }
                }

                let canvas_pos = self.transform_screen_to_canvas(
                    (position.x as f32 - self.bounds.x, position.y as f32 - self.bounds.y));
                let canvas_pos = cgmath::Vector2::new(canvas_pos.0, canvas_pos.1);
                let light_pos = cgmath::Vector2::new(self.main_light.position.0, self.main_light.position.1);
                let light_distance = canvas_pos.distance(light_pos);
                self.main_light.hovered = self.light_gizmos_interactable
                    && light_distance < (6.25 * (self.main_light.position.2 / 100.0)).max(2.0);
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
                    self.enable_light_parallax = !self.enable_light_parallax;
                }
            }
            _ => {}
        }
        false
    }


    // TODO: all this dependency injection is annoying
    pub fn resize(&mut self, new_size: (u32, u32), device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut imgui_wgpu::Renderer) {
        let rt_texture = Texture::new(device, renderer, TextureConfig {
            size: Extent3d { width: new_size.0, height: new_size.1, ..Default::default() },
            label: Some("canvas render target"),
            format: Some(TextureFormat::Rgba8Unorm),
            usage: TextureUsages::all(),
            dimension: wgpu::TextureDimension::D2,
            sampler_desc: SamplerDescriptor {
                label: Some("canvas rt sampler"),
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        });
        // TODO: do i need to initialize every time?
        let data = vec![127u8; (new_size.0*new_size.1) as usize * 4];
        rt_texture.write(&queue, &data[0..((new_size.0*new_size.1) as usize * 4)], new_size.0, new_size.1);

        renderer.textures.remove(self.rt_tex_id);
        self.rt_tex_id = renderer.textures.insert(rt_texture);
        self.sprite_pipeline.rt_tex_id = self.rt_tex_id;
        self.light_gizmo_pipeline.rt_tex_id = self.rt_tex_id;
        self.rt_size = new_size;
    }


    pub fn draw(&mut self, ui: &imgui::Ui, device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut imgui_wgpu::Renderer) {
        let [mouse_x, mouse_y] = ui.io().mouse_pos;
        ui.window("Viewport")
            .position([0.0, 20.0], Condition::FirstUseEver)
            .flags(WindowFlags::MENU_BAR)
            .size([
                      self.canvas_size.0 as f32 * self.zoom,
                      self.canvas_size.1 as f32 * self.zoom / 1.5
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
                if self.bounds.w != self.rt_size.0 as f32 || self.bounds.h != self.rt_size.1 as f32 {
                    self.resize((self.bounds.w.abs().floor() as u32, self.bounds.h.abs().floor() as u32), device, queue, renderer);
                }

                ui.get_window_draw_list().add_image(self.rt_tex_id, [x, y], [x+w, y+h]).build();
                self.is_hovered = ui.is_window_hovered() && self.viewport_bounds().test(mouse_x, mouse_y);
            });
    }


    pub fn viewport_bounds(&self) -> Rect<f32> {
        self.bounds.adjusted_by(5.0, 20.0, -10.0, -25.0)
    }


    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, renderer: &imgui_wgpu::Renderer) {
        let (scale_x, scale_y) = self.scale_screen_to_fb((self.zoom, self.zoom));
        let mut matrix = [
            [scale_x * self.canvas_size.0 as f32, 0.0, 0.0, 0.0],
            [0.0, -scale_y * self.canvas_size.1 as f32, 0.0, 0.0],
            [0.0,     0.0, 1.0, 0.0],
            [-1.0+self.offset.0*scale_x, 1.0-self.offset.1*scale_y, 0.0, 1.0],
        ];

        let center_screen = (self.bounds.w / 2.0, self.bounds.h / 2.0);
        let offset_screen = self.scale_canvas_to_screen(self.offset);
        let center_canvas = self.scale_screen_to_canvas((center_screen.0 - offset_screen.0, center_screen.1 - offset_screen.1));

        let l = &self.main_light;
        self.sprite_pipeline.update_uniforms(queue, CanvasSpritePipelineUniforms {
            matrix,
            light_color: [l.color.r as f32, l.color.g as f32, l.color.b as f32, l.color.a as f32],
            light_pos: [l.position.0, l.position.1, l.position.2, 0.0],
            cam_pos: [center_canvas.0, center_canvas.1, self.camera_height, 0.0],
            spec_power: 32.0,
            ambient_intensity: self.ambient_intensity,
            diffuse_intensity: self.diffuse_intensity,
            specular_intensity: self.specular_intensity,
            sprite_size: [self.canvas_size.0 as f32, self.canvas_size.1 as f32],
            light_falloff: if self.enable_light_falloff { self.light_falloff } else { 0.0 },
            map_view_type: self.shown_map_type as u32,
        });
        self.sprite_pipeline.render(encoder, renderer, &self.texture_registry);

        matrix[3][0] += self.main_light.position.0*scale_x;
        matrix[3][1] -= self.main_light.position.1*scale_y;
        let light_scale = self.main_light.position.2 / 100.0;
        matrix[0][0] *= light_scale;
        matrix[1][1] *= light_scale;

        let mut gizmo_color = self.main_light.color;
        gizmo_color.a = if self.main_light.hovered { self.gizmo_opacity * 2.0 } else { self.gizmo_opacity } as f64;
        self.light_gizmo_pipeline.update_uniforms(queue, &matrix, gizmo_color);
        self.light_gizmo_pipeline.render(encoder, renderer, &self.texture_registry);
    }

    #[allow(dead_code)]
    pub fn scale_screen_to_canvas(&self, vec: (f32, f32)) -> (f32, f32) {
        (vec.0 / self.zoom, vec.1 / self.zoom)
    }

    #[allow(dead_code)]
    pub fn scale_canvas_to_screen(&self, vec: (f32, f32)) -> (f32, f32) {
        (vec.0 * self.zoom, vec.1 * self.zoom)
    }

    #[allow(dead_code)]
    pub fn scale_screen_to_fb(&self, vec: (f32, f32)) -> (f32, f32) {
        let pixel_ratio_x = 2.0 / self.rt_size.0 as f32;
        let pixel_ratio_y = 2.0 / self.rt_size.1 as f32;
        (vec.0 * pixel_ratio_x, vec.1 * pixel_ratio_y)
    }

    #[allow(dead_code)]
    pub fn scale_fb_to_screen(&self, vec: (f32, f32)) -> (f32, f32) {
        let pixel_ratio_x = 2.0 / self.rt_size.0 as f32;
        let pixel_ratio_y = 2.0 / self.rt_size.1 as f32;
        (vec.0 / pixel_ratio_x, vec.1 / pixel_ratio_y)
    }

    #[allow(dead_code)]
    pub fn scale_fb_to_canvas(&self, vec: (f32, f32)) -> (f32, f32) {
        self.scale_screen_to_canvas(self.scale_fb_to_screen(vec))
    }

    #[allow(dead_code)]
    pub fn scale_canvas_to_fb(&self, vec: (f32, f32)) -> (f32, f32) {
        self.scale_screen_to_fb(self.scale_canvas_to_screen(vec))
    }

    #[allow(dead_code)]
    pub fn transform_screen_to_canvas(&self, pos: (f32, f32)) -> (f32, f32) {
        ((pos.0 / self.zoom) - self.offset.0, (pos.1 / self.zoom) - self.offset.1)
    }

    #[allow(dead_code)]
    pub fn transform_canvas_to_screen(&self, pos: (f32, f32)) -> (f32, f32) {
        ((pos.0 + self.offset.0) * self.zoom, (pos.1 + self.offset.1) * self.zoom)
    }

    #[allow(dead_code)]
    pub fn transform_screen_to_fb(&self, pos: (f32, f32)) -> (f32, f32) {
        (pos.0 * 2.0 / self.rt_size.0 as f32 - 1.0, pos.1 * 2.0 / self.rt_size.1 as f32 - 1.0)
    }

    #[allow(dead_code)]
    pub fn transform_fb_to_screen(&self, pos: (f32, f32)) -> (f32, f32) {
        ((pos.0 + 1.0) / self.rt_size.0 as f32 * 2.0, (pos.1 + 1.0) / self.rt_size.1 as f32 * 2.0)
    }

    #[allow(dead_code)]
    pub fn transform_canvas_to_fb(&self, pos: (f32, f32)) -> (f32, f32) {
        self.transform_screen_to_fb(self.transform_canvas_to_screen(pos))
    }

    #[allow(dead_code)]
    pub fn transform_fb_to_canvas(&self, pos: (f32, f32)) -> (f32, f32) {
        self.transform_screen_to_canvas(self.transform_fb_to_screen(pos))
    }
}