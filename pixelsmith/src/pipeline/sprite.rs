use wgpu::*;
use crate::geometry::{VertexGroup, VertexPosUV, VertexPosUVPod};
use crate::GLOBALS;
use crate::pipeline::{COLOR_TARGET_STATE, PRIMITIVE_STATE};
use crate::registry::{RegistryKey, TextureRegistry};


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CanvasSpritePipelineUniforms {
    pub matrix: [[f32; 4]; 4],
    pub light_color: [f32; 4],
    pub light_pos: [f32; 4],
    pub cam_pos: [f32; 4],
    pub spec_power: f32,
    pub ambient_intensity: f32,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub sprite_size: [f32; 2],
    pub light_falloff: f32,
    pub map_view_type: u32,
}
unsafe impl bytemuck::Zeroable for CanvasSpritePipelineUniforms {}
unsafe impl bytemuck::Pod for CanvasSpritePipelineUniforms {}


pub struct ViewportSpritePipeline {
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    sprite_vg: VertexGroup,
    pipeline: RenderPipeline,
    pub rt_key: RegistryKey
}


impl ViewportSpritePipeline {
    pub fn new(rt_key: RegistryKey) -> Self {
        let device = &GLOBALS.get().device;
        let shader_module = device.create_shader_module(&wgpu::include_wgsl!("canvas_sprite.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("viewport sprite uniform buffer"),
            size: std::mem::size_of::<CanvasSpritePipelineUniforms>() as BufferAddress,
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
            label: Some("canvas sprite bind group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("canvas sprite pipeline layout"),
            bind_group_layouts: &[&uniform_layout, &GLOBALS.get().sprite_maps_bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("canvas sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<VertexPosUV>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
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

        let sprite_vg = VertexGroup::from_data_with_labels(&[
            VertexPosUVPod(VertexPosUV { pos: [0.0, 0.0], uv: [0.0, 0.0] }),
            VertexPosUVPod(VertexPosUV { pos: [1.0, 0.0], uv: [1.0, 0.0] }),
            VertexPosUVPod(VertexPosUV { pos: [1.0, 1.0], uv: [1.0, 1.0] }),
            VertexPosUVPod(VertexPosUV { pos: [0.0, 1.0], uv: [0.0, 1.0] }),
        ], &[0u16, 2, 1, 0, 2, 3], Some("canvas sprite geometry"), &device);

        ViewportSpritePipeline {
            uniform_buffer,
            uniform_bind_group,
            sprite_vg,
            pipeline,
            rt_key
        }
    }

    pub fn update_uniforms(&self, uniforms: CanvasSpritePipelineUniforms) {
        GLOBALS.get().queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, registry: &TextureRegistry, maps_bg: usize) {
        let rt_info = registry.find(self.rt_key).unwrap();
        let view = rt_info.view();
        let bind_group = registry.find_bind_group(maps_bg).unwrap();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("canvas sprite renderpass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            rpass.set_bind_group(1, bind_group, &[]);
            rpass.set_vertex_buffer(0, self.sprite_vg.vertex_buffer().slice(..));
            rpass.set_index_buffer(self.sprite_vg.index_buffer().slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..6, 0, 0..1);
        }
    }
}