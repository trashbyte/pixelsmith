use std::cell::RefCell;
use imgui::*;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit::event_loop::EventLoopWindowTarget;
use imgui::docking::DockNodeFlags;
use toolbelt::{SimpleCell, Defer, normalize_with_constant};
use toolbelt::once::DoOnce;
use crate::GLOBALS;
use crate::viewport::Viewport;
use crate::palette::PaletteEditor;
use crate::project::ProjectData;
use crate::recent::draw_recent_window;
use crate::registry::TextureRegistry;
use crate::scene::Scene;


// yes i know this is terrible
static mut __IMGUI_CTX: RefCell<Option<imgui::Context>> = RefCell::new(None);
fn init_imgui_ctx(ctx: imgui::Context) {
    unsafe {
        *(__IMGUI_CTX.get_mut()) = Some(ctx);
    }
}
pub fn IMGUI_CTX() -> &'static mut imgui::Context {
    unsafe { __IMGUI_CTX.get_mut().as_mut().unwrap() }
}


const SURFACE_CONF: wgpu::SurfaceConfiguration  = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: wgpu::TextureFormat::Bgra8UnormSrgb,
    width: 0,
    height: 0,
    present_mode: wgpu::PresentMode::Mailbox,
};


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MapType { Albedo = 0, Normal = 1, Roughness = 2, Height = 3, Rendered = 4 }
impl MapType {
    pub const TYPES: [MapType; 5] = [MapType::Albedo, MapType::Normal, MapType::Roughness, MapType::Height, MapType::Rendered];
}
impl std::fmt::Display for MapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MapType::Albedo => "Albedo",
            MapType::Normal => "Normal",
            MapType::Roughness => "Roughness",
            MapType::Height => "Height",
            MapType::Rendered => "Rendered",
        })
    }
}


pub struct App {
    project: Option<ProjectData>,
    demo_open: bool,
    window: Window,
    surface: wgpu::Surface,
    viewports: [Option<Viewport>; 4],
    scene: Option<SimpleCell<Scene>>,
    imgui_platform: imgui_winit_support::WinitPlatform,
    imgui_renderer: crate::imgui_wgpu::Renderer,
    texture_registry: TextureRegistry,
    last_frame: Instant,
    last_cursor: Option<Option<MouseCursor>>,
    palette: PaletteEditor,
    selected_viewport: Option<usize>,
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
        })).unwrap();

        let (device, queue) = pollster::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        ).unwrap();
        crate::init_globals(device, queue);
        let device = &GLOBALS.get().device;

        surface.configure(device, &wgpu::SurfaceConfiguration {
            width: size.width as u32,
            height: size.height as u32,
            ..SURFACE_CONF
        });

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

        let mut texture_registry = TextureRegistry::new();
        let imgui_renderer = crate::imgui_wgpu::Renderer::new(&mut imgui_ctx, &mut texture_registry);

        init_imgui_ctx(imgui_ctx);

        App {
            project: None,
            demo_open: false,
            viewports: [None, None, None, None],
            scene: None,
            window, surface,
            imgui_platform, imgui_renderer,
            texture_registry,
            last_frame: Instant::now(),
            last_cursor: None,
            palette: PaletteEditor::new(),
            selected_viewport: None,
        }
    }


    fn open_project(&mut self, project: ProjectData) {
        self.project = Some(project.clone());

        let ini_path = project.ini_path();
        if ini_path.exists() {
            if let Ok(text) = std::fs::read_to_string(&ini_path) {
                IMGUI_CTX().load_ini_settings(text.as_str());
            }
        }
        IMGUI_CTX().set_ini_filename(Some(ini_path));
    }


    fn select_viewport(&mut self, num: usize) {
        self.selected_viewport = Some(num);
    }

    fn close_viewport(&mut self, num: usize) {
        if let Some(vp) = self.viewports[num].as_mut() {
            vp.close()
        }
        self.viewports[num] = None;
        if self.selected_viewport == Some(num) {
            self.selected_viewport = None;
        }
    }

    fn create_viewport(&mut self, num: usize) {
        if self.viewports[num].is_none() {
            let vp = Viewport::create((400, 400), self.scene.as_ref().unwrap(), &mut self.texture_registry);
            self.viewports[num] = Some(vp);
        }
        self.select_viewport(num);
    }

    fn get_viewport(&self, num: usize) -> Option<&Viewport> {
        self.viewports[num].as_ref()
    }

    fn get_viewport_mut(&mut self, num: usize) -> Option<&mut Viewport> {
        self.viewports[num].as_mut()
    }

    fn main_loop(&mut self, event: Event<()>, _: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow) {
        let device = &GLOBALS.get().device;
        let queue = &GLOBALS.get().queue;
        let imgui_ctx = IMGUI_CTX();
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = self.window.inner_size();

                self.surface.configure(&GLOBALS.get().device, &wgpu::SurfaceConfiguration {
                    width: size.width as u32,
                    height: size.height as u32,
                    ..SURFACE_CONF
                });
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
                imgui_ctx.io_mut().update_delta_time(now - self.last_frame);
                self.last_frame = now;

                let frame = match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        println!("dropped frame: {:?}", e);
                        return;
                    }
                };

                self.imgui_platform
                    .prepare_frame(imgui_ctx.io_mut(), &self.window)
                    .expect("Failed to prepare frame");
                {
                    let ui = imgui_ctx.new_frame();

                    if self.project.is_none() {
                        let size = self.window.inner_size();
                        if let Some((name, path)) = draw_recent_window(ui, [size.width as f32, size.height as f32]) {
                            println!("selected {} {}", name, path);
                            self.open_project(ProjectData { path: path.into() });
                        }
                    } else if self.scene.is_none() {
                        let (path, data) = self.project.as_ref().unwrap().find_sprites().into_iter().next().unwrap();
                        let scene = Scene::from_sprite_path(path, &mut self.texture_registry);
                        self.scene = Some(scene);
                        for (i, open) in data.viewports_open.iter().enumerate() {
                            self.close_viewport(i);
                            if *open {
                                self.create_viewport(i);
                            }
                        }
                    } else {
                        ui.dockspace_over_viewport(DockNodeFlags::NONE);

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
                            if let Some(inner) = ui.begin_menu("Panels") {
                                for i in 0..4 {
                                    if ui.menu_item_config(format!("Viewport {}", i + 1))
                                        .selected(self.viewports[i].is_some())
                                        .build()
                                    {
                                        if self.viewports[i].is_some() {
                                            self.close_viewport(i);
                                        } else {
                                            self.create_viewport(i);
                                        }
                                    }
                                }
                                inner.end();
                            }
                        });

                        for (i, vp) in self.viewports.iter_mut().enumerate() {
                            if let Some(vp) = vp {
                                vp.draw(ui, i, &mut self.texture_registry);
                            }
                        }

                        ui.window("Sidebar")
                            .size([300.0, 100.0], Condition::FirstUseEver)
                            .position([800.0, 100.0], Condition::FirstUseEver)
                            .build(|| {
                                if ui.collapsing_header("Info", TreeNodeFlags::empty()) {
                                    ui.spacing();

                                    ui.text(format!("Frame time: {:?}", delta_s));
                                    ui.separator();

                                    match self.selected_viewport {
                                        Some(num) => {
                                            let vp = self.viewports[num].as_ref().unwrap();
                                            ui.text(format!("Viewport {} spaces", num+1));
                                            let [mx, my] = ui.io().mouse_pos;
                                            let (mx, my) = (mx - vp.bounds.x, my - vp.bounds.y);
                                            ui.text(format!("Screen Space: ({:.1},{:.1})", mx, my));
                                            {
                                                let mouse_vec = vp.transform_screen_to_canvas((mx, my).into());
                                                ui.text(format!("Canvas Space: ({:.1},{:.1})", mouse_vec.x, mouse_vec.y));
                                            }
                                            let mouse_vec = vp.transform_screen_to_fb((mx, my).into());
                                            ui.text(format!("Framebuffer Space: ({:.3},{:.3})", mouse_vec.x, mouse_vec.y));
                                        }
                                        None => {
                                            ui.text("No viewport selected");
                                        }
                                    }

                                    ui.spacing();
                                    ui.spacing();
                                    ui.spacing();
                                }

                                // FIXME: scoping for the lighting section is kind of a nightmare to
                                //        avoid double-borrowing. Probably a better way to do this.
                                if ui.collapsing_header("Lighting", TreeNodeFlags::DEFAULT_OPEN) {
                                    ui.spacing();

                                    match self.selected_viewport {
                                        Some(num) => {
                                            ui.text("Light Intensity");

                                            let vp = self.get_viewport_mut(num).unwrap();
                                            ui.checkbox("Normalize", &mut vp.normalize_intensities);

                                            let mut once = DoOnce::new(); // necessary to prevent simultaneous ripple-updates
                                            if ui.slider("Ambient##intensity", 0.0, 1.0, &mut vp.global_ambient)
                                                && vp.normalize_intensities
                                            {
                                                once.do_once(|| {
                                                    (vp.global_ambient, vp.global_diffuse, vp.global_specular)
                                                        = normalize_with_constant(vp.global_ambient, vp.global_diffuse, vp.global_specular);
                                                });
                                            }
                                            if ui.slider("Diffuse##intensity", 0.0, 1.0, &mut vp.global_diffuse)
                                                && vp.normalize_intensities
                                            {
                                                once.do_once(|| {
                                                    (vp.global_diffuse, vp.global_ambient, vp.global_specular)
                                                        = normalize_with_constant(vp.global_diffuse, vp.global_ambient, vp.global_specular);
                                                });
                                            }
                                            if ui.slider("Specular##intensity", 0.0, 1.0, &mut vp.global_specular)
                                                && vp.normalize_intensities
                                            {
                                                once.do_once(|| {
                                                    (vp.global_specular, vp.global_diffuse, vp.global_ambient)
                                                        = normalize_with_constant(vp.global_specular, vp.global_diffuse, vp.global_ambient);
                                                });
                                            }
                                            ui.separator();

                                            ui.text("Light Control");
                                            {
                                                let vp = self.viewports[num].as_mut().unwrap();
                                                ui.checkbox("Gizmo Interactable", &mut vp.light_gizmos_interactable);
                                                if ui.slider_config("Opacity##light-gizmo", 0.005, 1.0)
                                                    .flags(SliderFlags::LOGARITHMIC)
                                                    .build(&mut vp.gizmo_opacity)
                                                {
                                                    if vp.gizmo_opacity < 0.005 + f32::EPSILON {
                                                        vp.gizmo_opacity = 0.0;
                                                        vp.light_gizmos_interactable = false;
                                                    }
                                                }
                                            }
                                            {
                                                let lighting = &mut self.scene.as_ref().unwrap().get_mut().lighting;
                                                ui.slider_config("Height##light-gizmo", 10.0, 250.0)
                                                    .flags(SliderFlags::LOGARITHMIC)
                                                    .build(&mut lighting.lights[0].height);
                                                ui.slider_config("Intensity##light-gizmo", 0.25, 5.0)
                                                    .flags(SliderFlags::LOGARITHMIC)
                                                    .display_format("%1.3f")
                                                    .build(&mut lighting.lights[0].color.components_4_mut()[3]);
                                                ui.separator();

                                                ui.checkbox("Use Light Falloff", &mut lighting.lights[0].enable_falloff);
                                                if lighting.lights[0].enable_falloff {
                                                    ui.slider_config("Exponent##light-falloff", 0.25, 4.0)
                                                        .flags(SliderFlags::LOGARITHMIC)
                                                        .build(&mut lighting.lights[0].falloff_exp);
                                                }
                                                ui.separator();

                                                ui.checkbox("Light Parallax", &mut lighting.enable_light_parallax);
                                            }
                                        }
                                        None => {
                                            ui.text("No viewport selected");
                                        }
                                    }

                                    ui.spacing();
                                    ui.spacing();
                                    ui.spacing();
                                }

                                if ui.collapsing_header("General", TreeNodeFlags::empty()) {
                                    ui.spacing();

                                    if let Some(num) = self.selected_viewport {
                                        let vp = self.viewports[num].as_mut().unwrap();
                                        ui.slider_config("Zoom##vp-zoom", 1.0, 32.0)
                                            .flags(SliderFlags::LOGARITHMIC)
                                            .build(&mut vp.zoom);
                                    }
                                    else {
                                        ui.text("No viewport selected");
                                    }

                                    ui.spacing();
                                    ui.spacing();
                                    ui.spacing();
                                }
                            });

                        self.palette.draw(&ui);

                        if self.demo_open {
                            ui.show_demo_window(&mut self.demo_open);
                        }
                    }

                    if self.last_cursor != Some(ui.mouse_cursor()) {
                        self.last_cursor = Some(ui.mouse_cursor());
                        self.imgui_platform.prepare_render(&ui, &self.window);
                    }
                } // drop ui

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                for vp in self.viewports.iter() {
                    if let Some(vp) = vp {
                        vp.render(&mut encoder, &self.texture_registry);
                    }
                }

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
                        .render(imgui_ctx.render(), &mut rpass, &mut self.texture_registry)
                        .expect("imgui rendering failed");
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {},
        }

        let mut consume_mouse = false;

        let select_vp = Defer::new();
        for (i, vp) in self.viewports.iter_mut().enumerate() {
            if let Some(vp) = vp {
                let result = vp.handle_event(imgui_ctx.io(), &event);
                if !consume_mouse && result {
                    select_vp.defer(i);
                }
                consume_mouse |= result;
            }
        }
        if !consume_mouse {
            self.imgui_platform.handle_event(imgui_ctx.io_mut(), &self.window, &event);
        }
        select_vp.execute(|num| {
            self.select_viewport(num);
        });
    }


    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |a,b,c| { self.main_loop(a, b, c) });
    }
}
