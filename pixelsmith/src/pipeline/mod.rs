use wgpu::*;
use imgui_wgpu::Renderer;
use crate::geometry::VertexGroup;


pub mod sprite;
pub use sprite::CanvasSpritePipeline;
pub mod light;
pub use light::CanvasLightPipeline;
use crate::registry::TextureRegistry;


const PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: FrontFace::Cw,
    cull_mode: None,
    polygon_mode: PolygonMode::Fill,
    unclipped_depth: false,
    conservative: false,
};

const COLOR_TARGET_STATE: ColorTargetState = ColorTargetState {
    format: TextureFormat::Rgba8Unorm,
    blend: Some(BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    }),
    write_mask: ColorWrites::ALL,
};


pub trait SimpleGeometryPipeline {
    fn uniform_buffer(&self) -> &wgpu::Buffer;
    fn uniform_bind_group(&self) -> &wgpu::BindGroup;
    fn vertex_group(&self) -> &VertexGroup;
    fn pipeline(&self) -> &RenderPipeline;

    fn render(&self, encoder: &mut wgpu::CommandEncoder, imgui_renderer: &Renderer, registry: &TextureRegistry);
}
