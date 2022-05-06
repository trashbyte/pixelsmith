use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource};
use imgui::TextureId;
use toolbelt::MonoCounter;
use crate::GLOBALS;


static KEY: MonoCounter = MonoCounter::new();
static BG_KEY: MonoCounter = MonoCounter::new();


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistryKey(u64);
impl Deref for RegistryKey {
    type Target = u64;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl Into<imgui::TextureId> for RegistryKey {
    fn into(self) -> TextureId { TextureId::new(self.0 as usize) }
}
impl RegistryKey {
    pub fn new(id: u64) -> Self { Self(id) }
    pub fn from_imgui(id: imgui::TextureId) -> Self { RegistryKey(id.id() as u64) }
}


#[derive(Debug, Clone)]
pub struct TextureMapSet {
    pub size: (u32, u32),
    pub albedo: RegistryKey,
    pub ao: RegistryKey,
    pub normal: RegistryKey,
    pub specular: RegistryKey,
    pub height: RegistryKey,
    pub extras: Vec<RegistryKey>,
    pub bind_group_idx: usize,
}

#[derive(Debug)]
pub struct TextureInfo {
    key: RegistryKey,
    label: String,
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    bind_group: Option<usize>,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
}
impl TextureInfo {
    fn create_texture(key: RegistryKey,
                      size: (u32, u32),
                      label: impl AsRef<str>,
                      format: wgpu::TextureFormat,
                      usage: wgpu::TextureUsages) -> Self
    {
        let (width, height) = size;
        let texture = Arc::new(GLOBALS.get().device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label.as_ref()),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        }));
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        TextureInfo {
            key,
            label: label.as_ref().to_string(),
            texture,
            view,
            bind_group: None,
            size,
            format,
            usage,
        }
    }

    pub fn replace_bind_group_idx(&mut self, bind_group_idx: usize) {
        self.bind_group = Some(bind_group_idx);
    }

    #[allow(dead_code)]
    pub fn key(&self) -> RegistryKey { self.key }
    #[allow(dead_code)]
    pub fn size(&self) -> (u32, u32) { self.size }
    //pub fn gpu_texture(&self) -> Arc<wgpu::Texture> { self.texture.clone() }
    pub fn view(&self) -> Arc<wgpu::TextureView> { self.view.clone() }
    #[allow(dead_code)]
    pub fn label(&self) -> String { self.label.clone() }
    #[allow(dead_code)]
    pub fn format(&self) -> wgpu::TextureFormat { self.format }
    #[allow(dead_code)]
    pub fn usage(&self) -> wgpu::TextureUsages { self.usage }

    /// Write `data` to the texture.
    ///
    /// - `data`: 32-bit RGBA bitmap data.
    /// - `width`: The width of the source bitmap (`data`) in pixels.
    /// - `height`: The height of the source bitmap (`data`) in pixels.
    pub fn write(&self, data: &[u8], width: u32, height: u32) {
        GLOBALS.get().queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &*self.texture,
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
    entries: HashMap<RegistryKey, TextureInfo>,
    bind_groups: HashMap<usize, BindGroup>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        // increment id past zero since zero is reserved for the font atlas
        let test = KEY.next();
        if test > 0 {
            panic!("Can't create more than one TextureRegistry")
        }
        TextureRegistry {
            entries: HashMap::new(),
            bind_groups: HashMap::new(),
        }
    }

    pub fn create_font_atlas(&mut self,
                             size: (u32, u32),
                             label: impl AsRef<str>,
                             data: &[u8]) -> RegistryKey {
        let mut info = TextureInfo::create_texture(RegistryKey(0),
                           size,
                           label,
                           wgpu::TextureFormat::Rgba8Unorm,
                           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
        info.write(data, size.0, size.1);
        let new_bg = self.add_bind_group(BindGroupDescriptor {
            label: Some("font atlas bind group"),
            layout: &GLOBALS.get().single_texture_bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&*info.view())
            }, BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&GLOBALS.get().font_atlas_sampler)
            }]
        });
        info.replace_bind_group_idx(new_bg);
        self.entries.insert(RegistryKey(0), info);
        RegistryKey(0)
    }

    pub fn create_texture(&mut self,
                      size: (u32, u32),
                      label: impl AsRef<str>,
                      format: wgpu::TextureFormat,
                      usage: wgpu::TextureUsages) -> RegistryKey {
        let key = RegistryKey(KEY.next());
        let info = TextureInfo::create_texture(key, size, label, format, usage);
        self.entries.insert(key, info);
        key
    }

    pub fn create_with_data(&mut self,
                          size: (u32, u32),
                          label: impl AsRef<str>,
                          format: wgpu::TextureFormat,
                          usage: wgpu::TextureUsages,
                          data: &[u8]) -> RegistryKey {
        let key = RegistryKey(KEY.next());
        let info = TextureInfo::create_texture(key, size, label, format, usage);
        info.write(data, size.0, size.1);
        self.entries.insert(key, info);
        key
    }

    pub fn find(&self, key: RegistryKey) -> Option<&TextureInfo> {
        self.entries.get(&key)
    }

    pub fn find_mut(&mut self, key: RegistryKey) -> Option<&mut TextureInfo> {
        self.entries.get_mut(&key)
    }

    pub fn remove(&mut self, key: RegistryKey) -> Option<TextureInfo> {
        self.entries.remove(&key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<RegistryKey, TextureInfo> {
        self.entries.iter()
    }

    pub fn texture_bind_group(&self, key: RegistryKey) -> Option<&BindGroup> {
        match self.entries.get(&key) {
            Some(tex) => {
                match tex.bind_group {
                    Some(id) => self.bind_groups.get(&id),
                    None => None
                }
            },
            None => None
        }
    }

    pub fn find_bind_group(&self, key: usize) -> Option<&BindGroup> {
        self.bind_groups.get(&key)
    }

    pub fn add_bind_group(&mut self, descriptor: wgpu::BindGroupDescriptor) -> usize {
        let key = BG_KEY.next() as usize;
        self.bind_groups.insert(key, GLOBALS.get().device.create_bind_group(&descriptor));
        key
    }
}
