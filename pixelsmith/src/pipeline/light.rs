use wgpu::*;
use imgui::TextureId;
use imgui_wgpu::Renderer;
use crate::geometry::{VertexGroup, VertexPos, VertexPosPod};
use crate::pipeline::{COLOR_TARGET_STATE, PRIMITIVE_STATE, SimpleGeometryPipeline};
use crate::registry::TextureRegistry;


pub struct CanvasLightPipeline {
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    gizmo_vg: VertexGroup,
    pipeline: RenderPipeline,
    pub rt_tex_id: TextureId,
}


impl CanvasLightPipeline {
    pub fn new(rt_tex_id: TextureId, device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(&wgpu::include_wgsl!("light_gizmo.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("canvas light gizmo uniform buffer"),
            size: 80,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("canvas light gizmo bind group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("canvas light gizmo pipeline layout"),
            bind_group_layouts: &[&uniform_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("canvas light gizmo pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<VertexPos>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2],
                }],
            },
            primitive: PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: MultisampleState { count: 1, ..Default::default() },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[COLOR_TARGET_STATE],
            }),
            multiview: None,
        });

        let gizmo_vg = VertexGroup::from_data_with_labels(
            &[VertexPosPod(VertexPos { pos: [ 0.000000, -0.500000 ] }), VertexPosPod(VertexPos { pos: [ -0.097545, -0.490393 ] }), VertexPosPod(VertexPos { pos: [ -0.191342, -0.461940 ] }), VertexPosPod(VertexPos { pos: [ -0.277785, -0.415735 ] }), VertexPosPod(VertexPos { pos: [ -0.353553, -0.353553 ] }), VertexPosPod(VertexPos { pos: [ -0.415735, -0.277785 ] }), VertexPosPod(VertexPos { pos: [ -0.461940, -0.191342 ] }), VertexPosPod(VertexPos { pos: [ -0.490393, -0.097545 ] }), VertexPosPod(VertexPos { pos: [ -0.500000, 0.000000 ] }), VertexPosPod(VertexPos { pos: [ -0.490393, 0.097545 ] }), VertexPosPod(VertexPos { pos: [ -0.461940, 0.191342 ] }), VertexPosPod(VertexPos { pos: [ -0.415735, 0.277785 ] }), VertexPosPod(VertexPos { pos: [ -0.353553, 0.353553 ] }), VertexPosPod(VertexPos { pos: [ -0.277785, 0.415735 ] }), VertexPosPod(VertexPos { pos: [ -0.191342, 0.461940 ] }), VertexPosPod(VertexPos { pos: [ -0.097545, 0.490393 ] }), VertexPosPod(VertexPos { pos: [ 0.000000, 0.500000 ] }), VertexPosPod(VertexPos { pos: [ 0.097545, 0.490393 ] }), VertexPosPod(VertexPos { pos: [ 0.191342, 0.461940 ] }), VertexPosPod(VertexPos { pos: [ 0.277785, 0.415735 ] }), VertexPosPod(VertexPos { pos: [ 0.353553, 0.353553 ] }), VertexPosPod(VertexPos { pos: [ 0.415735, 0.277785 ] }), VertexPosPod(VertexPos { pos: [ 0.461940, 0.191342 ] }), VertexPosPod(VertexPos { pos: [ 0.490393, 0.097545 ] }), VertexPosPod(VertexPos { pos: [ 0.500000, -0.000000 ] }), VertexPosPod(VertexPos { pos: [ 0.490393, -0.097545 ] }), VertexPosPod(VertexPos { pos: [ 0.461940, -0.191342 ] }), VertexPosPod(VertexPos { pos: [ 0.415735, -0.277785 ] }), VertexPosPod(VertexPos { pos: [ 0.353553, -0.353553 ] }), VertexPosPod(VertexPos { pos: [ 0.277785, -0.415735 ] }), VertexPosPod(VertexPos { pos: [ 0.191342, -0.461940 ] }), VertexPosPod(VertexPos { pos: [ 0.097545, -0.490393 ] }), VertexPosPod(VertexPos { pos: [ -0.000000, -0.450000 ] }), VertexPosPod(VertexPos { pos: [ -0.087791, -0.441353 ] }), VertexPosPod(VertexPos { pos: [ -0.172208, -0.415746 ] }), VertexPosPod(VertexPos { pos: [ -0.250007, -0.374161 ] }), VertexPosPod(VertexPos { pos: [ -0.318198, -0.318198 ] }), VertexPosPod(VertexPos { pos: [ -0.374161, -0.250007 ] }), VertexPosPod(VertexPos { pos: [ -0.415746, -0.172208 ] }), VertexPosPod(VertexPos { pos: [ -0.441353, -0.087791 ] }), VertexPosPod(VertexPos { pos: [ -0.450000, 0.000000 ] }), VertexPosPod(VertexPos { pos: [ -0.441353, 0.087791 ] }), VertexPosPod(VertexPos { pos: [ -0.415746, 0.172208 ] }), VertexPosPod(VertexPos { pos: [ -0.374161, 0.250007 ] }), VertexPosPod(VertexPos { pos: [ -0.318198, 0.318198 ] }), VertexPosPod(VertexPos { pos: [ -0.250007, 0.374161 ] }), VertexPosPod(VertexPos { pos: [ -0.172208, 0.415746 ] }), VertexPosPod(VertexPos { pos: [ -0.087791, 0.441353 ] }), VertexPosPod(VertexPos { pos: [ 0.000000, 0.450000 ] }), VertexPosPod(VertexPos { pos: [ 0.087791, 0.441353 ] }), VertexPosPod(VertexPos { pos: [ 0.172208, 0.415746 ] }), VertexPosPod(VertexPos { pos: [ 0.250007, 0.374161 ] }), VertexPosPod(VertexPos { pos: [ 0.318198, 0.318198 ] }), VertexPosPod(VertexPos { pos: [ 0.374161, 0.250007 ] }), VertexPosPod(VertexPos { pos: [ 0.415746, 0.172208 ] }), VertexPosPod(VertexPos { pos: [ 0.441353, 0.087791 ] }), VertexPosPod(VertexPos { pos: [ 0.450000, -0.000000 ] }), VertexPosPod(VertexPos { pos: [ 0.441353, -0.087791 ] }), VertexPosPod(VertexPos { pos: [ 0.415746, -0.172208 ] }), VertexPosPod(VertexPos { pos: [ 0.374161, -0.250007 ] }), VertexPosPod(VertexPos { pos: [ 0.318198, -0.318198 ] }), VertexPosPod(VertexPos { pos: [ 0.250007, -0.374161 ] }), VertexPosPod(VertexPos { pos: [ 0.172208, -0.415746 ] }), VertexPosPod(VertexPos { pos: [ 0.087791, -0.441353 ] }),  VertexPosPod(VertexPos { pos: [ 0.000000, 0.000000 ] }), VertexPosPod(VertexPos { pos: [ 0.000000, -0.100000 ] }), VertexPosPod(VertexPos { pos: [ -0.038268, -0.092388 ] }), VertexPosPod(VertexPos { pos: [ -0.070711, -0.070711 ] }), VertexPosPod(VertexPos { pos: [ -0.092388, -0.038268 ] }), VertexPosPod(VertexPos { pos: [ -0.100000, 0.000000 ] }), VertexPosPod(VertexPos { pos: [ -0.092388, 0.038268 ] }), VertexPosPod(VertexPos { pos: [ -0.070711, 0.070711 ] }), VertexPosPod(VertexPos { pos: [ -0.038268, 0.092388 ] }), VertexPosPod(VertexPos { pos: [ 0.000000, 0.100000 ] }), VertexPosPod(VertexPos { pos: [ 0.038268, 0.092388 ] }), VertexPosPod(VertexPos { pos: [ 0.070711, 0.070711 ] }), VertexPosPod(VertexPos { pos: [ 0.092388, 0.038268 ] }), VertexPosPod(VertexPos { pos: [ 0.100000, -0.000000 ] }), VertexPosPod(VertexPos { pos: [ 0.092388, -0.038268 ] }), VertexPosPod(VertexPos { pos: [ 0.070711, -0.070711 ] }), VertexPosPod(VertexPos { pos: [ 0.038268, -0.092388 ] })],
            &[27u16, 60, 28, 14, 47, 15, 2, 33, 34, 28, 61, 29, 15, 48, 16, 2, 35, 3, 30, 61, 62, 16, 49, 17, 3, 36, 4, 30, 63, 31, 17, 50, 18, 5, 36, 37, 31, 0, 32, 19, 50, 51, 5, 38, 6, 32, 0, 1, 19, 52, 20, 6, 39, 7, 20, 53, 21, 7, 40, 8, 22, 53, 54, 9, 40, 41, 22, 55, 23, 9, 42, 10, 23, 56, 24, 10, 43, 11, 25, 56, 57, 11, 44, 12, 25, 58, 26, 13, 44, 45, 26, 59, 27, 13, 46, 14, 27, 59, 60, 14, 46, 47, 2, 1, 33, 28, 60, 61, 15, 47, 48, 2, 34, 35, 30, 29, 61, 16, 48, 49, 3, 35, 36, 30, 62, 63, 17, 49, 50, 5, 4, 36, 31, 63, 32, 19, 18, 50, 5, 37, 38, 32, 1, 33, 19, 51, 52, 6, 38, 39, 20, 52, 53, 7, 39, 40, 22, 21, 53, 9, 8, 40, 22, 54, 55, 9, 41, 42, 23, 55, 56, 10, 42, 43, 25, 24, 56, 11, 43, 44, 25, 57, 58, 13, 12, 44, 26, 58, 59, 13, 45, 46, 64, 65, 66, 64, 66, 67, 64, 67, 68, 64, 68, 69, 64, 69, 70, 64, 70, 71, 64, 71, 72, 64, 72, 73, 64, 73, 74, 64, 74, 75, 64, 75, 76, 64, 76, 77, 64, 77, 78, 64, 78, 79, 64, 79, 80, 64, 80, 65],
            Some("canvas light gizmo geometry"), &device);

        CanvasLightPipeline {
            uniform_buffer,
            uniform_bind_group,
            gizmo_vg,
            pipeline,
            rt_tex_id,
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, m: &[[f32; 4]; 4], color: wgpu::Color) {
        let data = [
            m[0], m[1], m[2], m[3],
            [color.r as f32, color.g as f32, color.b as f32, color.a as f32]
        ];
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&data));
    }
}


impl SimpleGeometryPipeline for CanvasLightPipeline {
    fn uniform_buffer(&self) -> &Buffer { &self.uniform_buffer }
    fn uniform_bind_group(&self) -> &BindGroup { &self.uniform_bind_group }
    fn vertex_group(&self) -> &VertexGroup { &self.gizmo_vg }
    fn pipeline(&self) -> &RenderPipeline { &self.pipeline }

    fn render(&self, encoder: &mut wgpu::CommandEncoder, imgui_renderer: &Renderer, _registry: &TextureRegistry) {
        let view = imgui_renderer.textures.get(self.rt_tex_id).expect("Failed to retrieve render target texture").view();

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("canvas light gizmo renderpass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.gizmo_vg.vertex_buffer().slice(..));
        rpass.set_index_buffer(self.gizmo_vg.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..240, 0, 0..1);
    }
}
