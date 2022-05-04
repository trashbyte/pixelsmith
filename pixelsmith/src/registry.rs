use std::collections::HashMap;
use std::sync::{Arc, Mutex};


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureKey(usize);


#[derive(Debug, Clone)]
pub struct TextureMapSet {
    pub albedo: TextureKey,
    pub ao: TextureKey,
    pub normal: TextureKey,
    pub specular: TextureKey,
    pub height: TextureKey,
    pub extras: Vec<TextureKey>,
}


#[derive(Debug)]
pub struct TextureInfoInner {
    device: Arc<wgpu::Device>,
    label: String,
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    bind_group: Option<Arc<wgpu::BindGroup>>,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
}

#[derive(Debug, Clone)]
pub struct TextureInfo(Arc<Mutex<TextureInfoInner>>);
impl TextureInfo {
    pub fn new(size: (u32, u32),
               label: impl AsRef<str>,
               format: wgpu::TextureFormat,
               usage: wgpu::TextureUsages,
               device: &Arc<wgpu::Device>) -> Self
    {
        let (width, height) = size;
        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label.as_ref()),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        }));
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        TextureInfo(Arc::new(Mutex::new(TextureInfoInner {
            device: device.clone(),
            label: label.as_ref().to_string(),
            texture,
            view,
            bind_group: None,
            size,
            format,
            usage,
        })))
    }

    /// Note: returns descriptor without label because wgpu hates allowing anything to own data.
    /// Call `label()` to get the label.
    pub fn descriptor(&self) -> wgpu::TextureDescriptor {
        let info = self.0.lock().unwrap();
        let (width, height) = info.size;
        wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: info.format,
            usage: info.usage,
        }
    }

    pub fn replace_bind_group(&self, bind_group: &Arc<wgpu::BindGroup>) {
        self.0.lock().unwrap().bind_group = Some(bind_group.clone());
    }

    pub fn gpu_texture(&self) -> Arc<wgpu::Texture> { self.0.lock().unwrap().texture.clone() }

    pub fn view(&self) -> Arc<wgpu::TextureView> { self.0.lock().unwrap().view.clone() }

    pub fn label(&self) -> String { self.0.lock().unwrap().label.clone() }

    pub fn bind_group(&self) -> Option<Arc<wgpu::BindGroup>> { self.0.lock().unwrap().bind_group.clone() }

    /// Write `data` to the texture.
    ///
    /// - `data`: 32-bit RGBA bitmap data.
    /// - `width`: The width of the source bitmap (`data`) in pixels.
    /// - `height`: The height of the source bitmap (`data`) in pixels.
    pub fn write(&self, queue: &wgpu::Queue, data: &[u8], width: u32, height: u32) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &*self.0.lock().unwrap().texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: core::num::NonZeroU32::new(width * 4),
                rows_per_image: core::num::NonZeroU32::new(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
    }
}

#[derive(Debug)]
pub struct TextureRegistry {
    entries: Mutex<HashMap<String, TextureInfo>>
}

impl TextureRegistry {
    pub fn new() -> Self {
        TextureRegistry {
            entries: Mutex::new(HashMap::new())
        }
    }

    pub fn add(&self, label: impl AsRef<str>, info: &TextureInfo) {
        self.entries.lock().unwrap().insert(label.as_ref().to_string(), info.clone());
    }

    pub fn get(&self, label: impl AsRef<str>) -> Option<TextureInfo> {
        let lock = self.entries.lock().unwrap();
        lock.get(label.as_ref()).map(|entry| entry.clone())
    }
}
unsafe impl Send for TextureRegistry {}
unsafe impl Sync for TextureRegistry {}
