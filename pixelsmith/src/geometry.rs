use num::Integer;
use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VertexPosUV {
    pub pos: [f32; 2],
    pub uv: [f32; 2]
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct VertexPosUVPod(pub VertexPosUV);
unsafe impl bytemuck::Zeroable for VertexPosUVPod {}
unsafe impl bytemuck::Pod for VertexPosUVPod {}


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VertexPos {
    pub pos: [f32; 2]
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct VertexPosPod(pub VertexPos);
unsafe impl bytemuck::Zeroable for VertexPosPod {}
unsafe impl bytemuck::Pod for VertexPosPod {}


pub struct VertexGroup {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}


impl VertexGroup {
    pub fn from_data_with_labels<V: bytemuck::Pod, I: bytemuck::Pod + Integer>(
        vertices: &[V], indices: &[I], label: Option<&str>, device: &wgpu::Device
    ) -> Self {
        let v_label = label.map(|s| format!("{} vertex buffer", s));
        let mut vertex_data = vec![0u8; vertices.len() * std::mem::size_of::<V>()];
        let len = vertex_data.len();
        vertex_data.copy_from_slice(unsafe { core::slice::from_raw_parts(vertices.as_ptr() as *const u8, len) });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: v_label.as_ref().map(|s| s.as_str()),
            contents: &vertex_data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let i_label = label.map(|s| format!("{} index buffer", s));
        let mut index_data = vec![0u8; indices.len() * std::mem::size_of::<I>()];
        let len = index_data.len();
        index_data.copy_from_slice(unsafe { core::slice::from_raw_parts(indices.as_ptr() as *const u8, len) });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: i_label.as_ref().map(|s| s.as_str()),
            contents: &index_data,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        VertexGroup { vertex_buffer, index_buffer }
    }


    #[allow(dead_code)]
    pub fn from_data<V: bytemuck::Pod, I: bytemuck::Pod + Integer>(
        vertices: &[V], indices: &[I], device: &wgpu::Device
    ) -> Self {
        VertexGroup::from_data_with_labels(vertices, indices, None, device)
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer { &self.vertex_buffer }
    pub fn index_buffer(&self) -> &wgpu::Buffer { &self.index_buffer }
}
