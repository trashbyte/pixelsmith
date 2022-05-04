use std::path::PathBuf;
use std::sync::Arc;
use imgui::*;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit::event_loop::EventLoopWindowTarget;
use imgui::__core::fmt::Formatter;
use imgui::docking::DockNodeFlags;
use toolbelt::normalize_with_constant;
use toolbelt::once::DoOnce;
use crate::canvas::Canvas;
use crate::palette::PaletteEditor;
use crate::recent::draw_recent_window;
use crate::registry::TextureRegistry;


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MapType { Albedo = 0, Normal = 1, Roughness = 2, Height = 3, Rendered = 4 }
impl MapType {
    pub const TYPES: [MapType; 5] = [MapType::Albedo, MapType::Normal, MapType::Roughness, MapType::Height, MapType::Rendered];
}
impl std::fmt::Display for MapType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MapType::Albedo => "Albedo",
            MapType::Normal => "Normal",
            MapType::Roughness => "Roughness",
            MapType::Height => "Height",
            MapType::Rendered => "Rendered",
        })
    }
}


struct ProjectData;
pub struct App {
    project: Option<ProjectData>,
    demo_open: bool,
    window: Window,
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    canvas: Canvas,
    imgui_platform: imgui_winit_support::WinitPlatform,
    imgui_renderer: imgui_wgpu::Renderer,
    imgui_ctx: imgui::Context,
    texture_registry: Arc<TextureRegistry>,
    last_frame: Instant,
    last_cursor: Option<Option<MouseCursor>>,
    palette: PaletteEditor,
}


impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let (window, size, surface) = {
            let window = Window::new(&event_loop).unwrap();
            window.set_inner_size(LogicalSize::<f64> { width: 1440.0, height: 810.0 });
            window.set_title(&format!("pixelsmith"));
            let size = window.inner_size();
            let surface = unsafe { instance.create_surface(&window) };
            (window, size, surface)
        };

        let hidpi_factor = window.scale_factor();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
            .unwrap();

        let (device, queue) = pollster::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        ).unwrap();
        let (device, queue) = (Arc::new(device), Arc::new(queue));

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &surface_desc);

        let mut imgui_ctx = imgui::Context::create();
        imgui_ctx.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
        imgui_ctx.io_mut().config_docking_with_shift = true;
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        imgui_platform.attach_window(
            imgui_ctx.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui_ctx.set_ini_filename(None);

        imgui_ctx.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui_ctx.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../../resources/MerriweatherSans-Light.ttf"),
            size_pixels: (14.0 * hidpi_factor) as f32,
            config: Some(FontConfig {
                name: Some("Merriweather Sans 14px".to_string()),
                oversample_h: 4,
                oversample_v: 4,
                pixel_snap_h: true,
                size_pixels: (14.0 * hidpi_factor) as f32,
                rasterizer_multiply: 1.0,
                glyph_extra_spacing: [1.1, 0.0],
                ..Default::default()
            })
        }]);
        imgui_ctx.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                name: Some("Proggy Clean 13px".to_string()),
                oversample_h: 1,
                oversample_v: 1,
                pixel_snap_h: true,
                size_pixels: (13.0 * hidpi_factor) as f32,
                ..Default::default()
            }),
        }]);

        let mut imgui_renderer = imgui_wgpu::Renderer::new(&mut imgui_ctx, &device, &queue,
            imgui_wgpu::RendererConfig {
                texture_format: surface_desc.format,
                ..Default::default()
            });

        let registry = Arc::new(TextureRegistry::new());
        let canvas = Canvas::create((64, 64), (600, 600), &device, &queue, &mut imgui_renderer, &registry);

        imgui_ctx.set_ini_filename(Some(PathBuf::from("settings.ini")));

        App {
            project: None,
            demo_open: false,
            window, surface, device, queue, canvas, imgui_platform, imgui_renderer, imgui_ctx,
            texture_registry: registry,
            last_frame: Instant::now(),
            last_cursor: None,
            palette: PaletteEditor::new(),
        }
    }


    fn main_loop(&mut self, event: Event<()>, _: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = self.window.inner_size();

                let surface_desc = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width as u32,
                    height: size.height as u32,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                self.surface.configure(&self.device, &surface_desc);
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    state: ElementState::Pressed,
                    .. },
                .. }, .. }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => self.window.request_redraw(),
            Event::RedrawEventsCleared => {
                let delta_s = self.last_frame.elapsed();
                let now = Instant::now();
                self.imgui_ctx.io_mut().update_delta_time(now - self.last_frame);
                self.last_frame = now;

                let frame = match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        println!("dropped frame: {:?}", e);
                        return;
                    }
                };
                self.imgui_platform
                    .prepare_frame(self.imgui_ctx.io_mut(), &self.window)
                    .expect("Failed to prepare frame");
                let ui = self.imgui_ctx.new_frame();

                if self.project.is_none() {
                    let size = self.window.inner_size();
                    if let Some((name, path)) = draw_recent_window(ui, [size.width as f32, size.height as f32]) {
                        println!("selected {} {}", name, path);
                        self.project = Some(ProjectData{});
                    }
                }
                else {
                    let root_dockspace_id = ui.dockspace_over_viewport(DockNodeFlags::PASSTHRU_CENTRAL_NODE);

                    ui.main_menu_bar(|| {
                        if let Some(inner) = ui.begin_menu("File") {
                            if ui.menu_item_config("Show Demo Window")
                                .selected(self.demo_open)
                                .build()
                            {
                                self.demo_open = !self.demo_open;
                            }
                            if ui.menu_item("Quit") {
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                            inner.end();
                        }
                    });

                    self.canvas.draw(ui, &self.device, &self.queue, &mut self.imgui_renderer);

                    ui.window("Sidebar")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .position([800.0, 100.0], Condition::FirstUseEver)
                        .build(|| {
                            if ui.collapsing_header("Info", TreeNodeFlags::empty()) {
                                ui.spacing();

                                ui.text(format!("Frame time: {:?}", delta_s));
                                ui.separator();

                                let [mx, my] = ui.io().mouse_pos;
                                let canvas = self.canvas.bounds;
                                let (mx, my) = (mx - canvas.x, my - canvas.y);
                                ui.text(format!("Screen Space: ({:.1},{:.1})", mx, my));
                                {
                                    let (mx, my) = self.canvas.transform_screen_to_canvas((mx, my));
                                    ui.text(format!("Canvas Space: ({:.1},{:.1})", mx, my));
                                }
                                let (mx, my) = self.canvas.transform_screen_to_fb((mx, my));
                                ui.text(format!("Framebuffer Space: ({:.3},{:.3})", mx, my));

                                ui.spacing(); ui.spacing(); ui.spacing();
                            }

                            if ui.collapsing_header("Lighting", TreeNodeFlags::DEFAULT_OPEN) {
                                ui.spacing();

                                ui.text("Light Intensity");
                                ui.checkbox("Normalize", &mut self.canvas.normalize_intensity);

                                let mut once = DoOnce::new(); // necessary to prevent simultaneous ripple-updates
                                if ui.slider("Ambient##intensity", 0.0, 1.0, &mut self.canvas.ambient_intensity)
                                    && self.canvas.normalize_intensity
                                {
                                    once.do_once(|| {
                                        let c = &mut self.canvas;
                                        (c.ambient_intensity, c.diffuse_intensity, c.specular_intensity)
                                            = normalize_with_constant(c.ambient_intensity, c.diffuse_intensity, c.specular_intensity);
                                    });
                                }
                                if ui.slider("Diffuse##intensity", 0.0, 1.0, &mut self.canvas.diffuse_intensity)
                                    && self.canvas.normalize_intensity
                                {
                                    once.do_once(|| {
                                        let c = &mut self.canvas;
                                        (c.diffuse_intensity, c.ambient_intensity, c.specular_intensity)
                                            = normalize_with_constant(c.diffuse_intensity, c.ambient_intensity, c.specular_intensity);
                                    });
                                }
                                if ui.slider("Specular##intensity", 0.0, 1.0, &mut self.canvas.specular_intensity)
                                    && self.canvas.normalize_intensity
                                {
                                    once.do_once(|| {
                                        let c = &mut self.canvas;
                                        (c.specular_intensity, c.ambient_intensity, c.diffuse_intensity)
                                            = normalize_with_constant(c.specular_intensity, c.ambient_intensity, c.diffuse_intensity);
                                    });
                                }
                                ui.separator();

                                ui.text("Light Control");
                                ui.checkbox("Gizmo Interactable", &mut self.canvas.light_gizmos_interactable);
                                if ui.slider_config("Opacity##light-gizmo", 0.005, 1.0)
                                    .flags(SliderFlags::LOGARITHMIC)
                                    .build(&mut self.canvas.gizmo_opacity)
                                {
                                    if self.canvas.gizmo_opacity < 0.005 + f32::EPSILON {
                                        self.canvas.gizmo_opacity = 0.0;
                                        self.canvas.light_gizmos_interactable = false;
                                    }
                                }
                                ui.slider_config("Height##light-gizmo", 10.0, 250.0)
                                    .flags(SliderFlags::LOGARITHMIC)
                                    .build(&mut self.canvas.main_light.position.2);
                                ui.slider_config("Intensity##light-gizmo", 0.25, 5.0)
                                    .flags(SliderFlags::LOGARITHMIC)
                                    .display_format("%1.3f")
                                    .build(&mut self.canvas.main_light.color.a);

                                ui.separator();

                                ui.checkbox("Use Light Falloff", &mut self.canvas.enable_light_falloff);
                                if self.canvas.enable_light_falloff {
                                    ui.slider_config("Exponent##light-falloff", 0.25, 4.0)
                                        .flags(SliderFlags::LOGARITHMIC)
                                        .build(&mut self.canvas.light_falloff);
                                }
                                ui.separator();

                                ui.checkbox("Light Parallax", &mut self.canvas.enable_light_parallax);

                                ui.spacing(); ui.spacing(); ui.spacing();
                            }

                            if ui.collapsing_header("General", TreeNodeFlags::empty()) {
                                ui.spacing();

                                ui.slider_config("Zoom##main-zoom", 1.0, 32.0)
                                    .flags(SliderFlags::LOGARITHMIC)
                                    .build(&mut self.canvas.zoom);

                                ui.spacing(); ui.spacing(); ui.spacing();
                            }
                        });

                    self.palette.draw(&ui);

                    if self.demo_open {
                        ui.show_demo_window(&mut self.demo_open);
                    }
                }

                let mut encoder: wgpu::CommandEncoder =
                    self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if self.last_cursor != Some(ui.mouse_cursor()) {
                    self.last_cursor = Some(ui.mouse_cursor());
                    self.imgui_platform.prepare_render(&ui, &self.window);
                }

                self.canvas.render(&mut encoder, &self.queue, &self.imgui_renderer);

                {
                    let view = frame.texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                    self.imgui_renderer
                        .render(self.imgui_ctx.render(), &self.queue, &self.device, &mut rpass)
                        .expect("imgui rendering failed");
                }

                self.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {},
        }

        let consume_mouse = self.canvas.handle_event(self.imgui_ctx.io(), &event);
        if !consume_mouse {
            self.imgui_platform.handle_event(self.imgui_ctx.io_mut(), &self.window, &event);
        }
    }


    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |a,b,c| { self.main_loop(a, b, c) });
    }
}
