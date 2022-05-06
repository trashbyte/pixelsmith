use imgui::{Context, DrawCmd::Elements, DrawData, DrawIdx, DrawVert, TextureId};
use std::mem::size_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use crate::GLOBALS;
use crate::pipeline::COLOR_TARGET_STATE;
use crate::registry::{RegistryKey, TextureRegistry};

pub type RendererResult<T> = Result<T, ()>;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct DrawVertPod(DrawVert);

unsafe impl bytemuck::Zeroable for DrawVertPod {}

unsafe impl bytemuck::Pod for DrawVertPod {}


pub struct RenderData {
    fb_size: [f32; 2],
    last_size: [f32; 2],
    last_pos: [f32; 2],
    vertex_buffer: Option<Buffer>,
    vertex_buffer_size: usize,
    index_buffer: Option<Buffer>,
    index_buffer_size: usize,
    draw_list_offsets: Vec<(i32, u32)>,
    render: bool,
}

pub struct Renderer {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    render_data: Option<RenderData>,
}

impl Renderer {
    pub fn new(
        imgui: &mut Context,
        registry: &mut TextureRegistry,
    ) -> Self {
        let device = &GLOBALS.get().device;

        let shader_module = device.create_shader_module(&include_wgsl!("imgui.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui-wgpu uniforms buffer"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui-wgpu uniforms bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("imgui-wgpu uniforms bind group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("imgui-wgpu pipeline layout"),
            bind_group_layouts: &[&uniform_layout, &GLOBALS.get().single_texture_bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("imgui-wgpu pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: size_of::<DrawVert>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                ..Default::default()
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main_linear",
                targets: &[COLOR_TARGET_STATE],
            }),
            multiview: None,
        });

        let mut renderer = Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            render_data: None,
        };

        // Immediately load the font texture to the GPU.
        renderer.reload_font_texture(imgui, registry);

        renderer
    }

    /// Prepares buffers for the current imgui frame.  This must be
    /// called before `Renderer::split_render`, and its output must
    /// be passed to the render call.
    pub fn prepare(
        &self,
        draw_data: &DrawData,
        render_data: Option<RenderData>,
    ) -> RenderData {
        let device = &GLOBALS.get().device;
        let queue = &GLOBALS.get().queue;

        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        let mut render_data = render_data.unwrap_or_else(|| RenderData {
            fb_size: [fb_width, fb_height],
            last_size: [0.0, 0.0],
            last_pos: [0.0, 0.0],
            vertex_buffer: None,
            vertex_buffer_size: 0,
            index_buffer: None,
            index_buffer_size: 0,
            draw_list_offsets: Vec::new(),
            render: false,
        });

        // If the render area is <= 0, exit here and now.
        if fb_width <= 0.0 || fb_height <= 0.0 {
            render_data.render = false;
            return render_data;
        } else {
            render_data.render = true;
        }

        // Only update matrices if the size or position changes
        if (render_data.last_size[0] - draw_data.display_size[0]).abs() > std::f32::EPSILON
            || (render_data.last_size[1] - draw_data.display_size[1]).abs() > std::f32::EPSILON
            || (render_data.last_pos[0] - draw_data.display_pos[0]).abs() > std::f32::EPSILON
            || (render_data.last_pos[1] - draw_data.display_pos[1]).abs() > std::f32::EPSILON
        {
            render_data.fb_size = [fb_width, fb_height];
            render_data.last_size = draw_data.display_size;
            render_data.last_pos = draw_data.display_pos;

            let width = draw_data.display_size[0];
            let height = draw_data.display_size[1];

            let offset_x = draw_data.display_pos[0] / width;
            let offset_y = draw_data.display_pos[1] / height;

            // Create and update the transform matrix for the current frame.
            // This is required to adapt to vulkan coordinates.
            let matrix = [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -height as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0 - offset_x * 2.0, 1.0 + offset_y * 2.0, 0.0, 1.0],
            ];
            self.update_uniform_buffer(queue, &matrix);
        }

        render_data.draw_list_offsets.clear();

        let mut vertex_count = 0;
        let mut index_count = 0;
        for draw_list in draw_data.draw_lists() {
            render_data
                .draw_list_offsets
                .push((vertex_count as i32, index_count as u32));
            vertex_count += draw_list.vtx_buffer().len();
            index_count += draw_list.idx_buffer().len();
        }

        let mut vertices = Vec::with_capacity(vertex_count * std::mem::size_of::<DrawVertPod>());
        let mut indices = Vec::with_capacity(index_count * std::mem::size_of::<DrawIdx>());

        for draw_list in draw_data.draw_lists() {
            // Safety: DrawVertPod is #[repr(transparent)] over DrawVert and DrawVert _should_ be Pod.
            let vertices_pod: &[DrawVertPod] = unsafe { draw_list.transmute_vtx_buffer() };
            vertices.extend_from_slice(bytemuck::cast_slice(vertices_pod));
            indices.extend_from_slice(bytemuck::cast_slice(draw_list.idx_buffer()));
        }

        // Copies in wgpu must be padded to 4 byte alignment
        indices.resize(
            indices.len() + COPY_BUFFER_ALIGNMENT as usize
                - indices.len() % COPY_BUFFER_ALIGNMENT as usize,
            0,
        );

        // If the buffer is not created or is too small for the new indices, create a new buffer
        if render_data.index_buffer.is_none() || render_data.index_buffer_size < indices.len() {
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("imgui-wgpu index buffer"),
                contents: &indices,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            render_data.index_buffer = Some(buffer);
            render_data.index_buffer_size = indices.len();
        } else if let Some(buffer) = render_data.index_buffer.as_ref() {
            // The buffer is large enough for the new indices, so reuse it
            queue.write_buffer(buffer, 0, &indices);
        } else {
            unreachable!()
        }

        // If the buffer is not created or is too small for the new vertices, create a new buffer
        if render_data.vertex_buffer.is_none() || render_data.vertex_buffer_size < vertices.len() {
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("imgui vertex buffer"),
                contents: &vertices,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            render_data.vertex_buffer = Some(buffer);
            render_data.vertex_buffer_size = vertices.len();
        } else if let Some(buffer) = render_data.vertex_buffer.as_ref() {
            // The buffer is large enough for the new vertices, so reuse it
            queue.write_buffer(buffer, 0, &vertices);
        } else {
            unreachable!()
        }

        render_data
    }

    /// Render the current imgui frame.
    pub fn render<'r>(
        &'r mut self,
        draw_data: &DrawData,
        rpass: &mut RenderPass<'r>,
        registry: &'r mut TextureRegistry,
    ) -> RendererResult<()> {
        let render_data = self.render_data.take();
        self.render_data = Some(self.prepare(draw_data, render_data));
        let render_data = self.render_data.as_ref().unwrap();
        if !render_data.render {
            return Ok(());
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, render_data.vertex_buffer.as_ref().unwrap().slice(..));
        rpass.set_index_buffer(
            render_data.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint16,
        );

        // Execute all the imgui render work.
        for (draw_list, bases) in draw_data
            .draw_lists()
            .zip(render_data.draw_list_offsets.iter())
        {
            let fb_size = render_data.fb_size;
            let clip_off = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;
            let (vertex_base, index_base) = *bases;
            let mut start = index_base;

            for cmd in draw_list.commands() {
                if let Elements { count, cmd_params } = cmd {
                    let clip_rect = [
                        (cmd_params.clip_rect[0] - clip_off[0]) * clip_scale[0],
                        (cmd_params.clip_rect[1] - clip_off[1]) * clip_scale[1],
                        (cmd_params.clip_rect[2] - clip_off[0]) * clip_scale[0],
                        (cmd_params.clip_rect[3] - clip_off[1]) * clip_scale[1],
                    ];

                    let maybe_bg = registry.texture_bind_group(RegistryKey::from_imgui(cmd_params.texture_id));
                    if let Some(bg) = maybe_bg {
                        rpass.set_bind_group(1, bg, &[]);
                    }
                    else {
                        eprintln!("No bind group found for texture with key {:?}", cmd_params.texture_id);
                        continue
                    }

                    // Set scissors on the renderpass.
                    let end = start + count as u32;
                    if clip_rect[0] < fb_size[0]
                        && clip_rect[1] < fb_size[1]
                        && clip_rect[2] >= 0.0
                        && clip_rect[3] >= 0.0
                    {
                        let scissors = (
                            clip_rect[0].max(0.0).floor() as u32,
                            clip_rect[1].max(0.0).floor() as u32,
                            (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                            (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                        );

                        // Only issue draw calls if the scissor rect is non-zero size.
                        // A zero-size scissor is essentially a no-op render anyway, so just skip it.
                        // See: https://github.com/gfx-rs/wgpu/issues/1750
                        if scissors.2 > 0 && scissors.3 > 0 {
                            rpass.set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);
                            rpass.draw_indexed(start..end, vertex_base, 0..1);
                        }
                    }

                    // Increment the index regardless of whether or not this batch of vertices was drawn.
                    start = end;
                }
            }
        }
        Ok(())
    }

    /// Updates the current uniform buffer containing the transform matrix.
    fn update_uniform_buffer(&self, queue: &Queue, matrix: &[[f32; 4]; 4]) {
        let data = bytemuck::bytes_of(matrix);
        queue.write_buffer(&self.uniform_buffer, 0, data);
    }

    /// Updates the texture on the GPU corresponding to the current imgui font atlas.
    ///
    /// This has to be called after loading a font.
    pub fn reload_font_texture(&mut self, imgui: &mut Context, registry: &mut TextureRegistry) {
        let mut fonts = imgui.fonts();
        // Remove possible font atlas texture.
        registry.remove(RegistryKey::new(0));

        // Create font texture and upload it.
        let handle = fonts.build_rgba32_texture();
        registry.create_font_atlas((handle.width, handle.height),
                                                "imgui font atlas texture",
                                                handle.data);
        fonts.tex_id = TextureId::new(0);
        // Clear imgui texture data to save memory.
        fonts.clear_tex_data();
    }
}
