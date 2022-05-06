mod app;
mod geometry;
mod lights;
mod palette;
mod pipeline;
mod project;
mod recent;
mod registry;
mod scene;
mod sprite;
mod viewport;
mod imgui_wgpu;


trait Toggle {
    fn toggle(&mut self);
}
impl Toggle for bool {
    fn toggle(&mut self) {
        *self = !*self
    }
}


pub static GLOBALS: toolbelt::once::InitOnce<GlobalStatics> =
    toolbelt::once::InitOnce::uninitialized();
#[derive(Debug)]
pub struct GlobalStatics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub sprite_maps_bind_layout: wgpu::BindGroupLayout,
    pub single_texture_bind_layout: wgpu::BindGroupLayout,

    pub rt_sampler: wgpu::Sampler,
    pub font_atlas_sampler: wgpu::Sampler,
}

pub fn init_globals(device: wgpu::Device, queue: wgpu::Queue) {
    const MAP_BIND_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2, },
        count: None,
    };

    let sprite_maps_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("sprite maps bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry { binding: 1, ..MAP_BIND_ENTRY },
            wgpu::BindGroupLayoutEntry { binding: 2, ..MAP_BIND_ENTRY },
            wgpu::BindGroupLayoutEntry { binding: 3, ..MAP_BIND_ENTRY },
            wgpu::BindGroupLayoutEntry { binding: 4, ..MAP_BIND_ENTRY },
        ],
    });

    let single_texture_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("imgui texture bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let rt_sampler = device.create_sampler(
        &wgpu::SamplerDescriptor { label: Some("font atlas sampler"), ..Default::default() });
    let font_atlas_sampler = device.create_sampler(
        &wgpu::SamplerDescriptor { label: Some("font atlas sampler"), .. Default::default() });

    GLOBALS.initialize(GlobalStatics {
        device,
        queue,
        sprite_maps_bind_layout,
        single_texture_bind_layout,
        rt_sampler,
        font_atlas_sampler,
    });
}


fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    app::App::new(&event_loop).run(event_loop);
}
