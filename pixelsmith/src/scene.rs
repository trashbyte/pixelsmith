use std::path::PathBuf;
use std::sync::Arc;
use toolbelt::cgmath::Point2;
use parking_lot::Mutex;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource, FilterMode, SamplerDescriptor, TextureFormat, TextureUsages};
use toolbelt::{SimpleCell, Color, ColorSpace};
use toolbelt::drag::DragState;
use crate::GLOBALS;
use crate::lights::{Light, LightingInfo};
use crate::registry::{TextureMapSet, TextureRegistry};


pub fn TEMP_create_texture_map_set(path: PathBuf, registry: &mut TextureRegistry) -> TextureMapSet {
    let maps_sampler = GLOBALS.get().device.create_sampler(&SamplerDescriptor {
        label: Some("sprite maps sampler"),
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let albedo_img = image::io::Reader::open(path.join("albedo.png")).unwrap().decode().unwrap();
    let img_size = (albedo_img.width(), albedo_img.height());
    let albedo_key = registry.create_with_data(img_size, "sprite albedo texture",
                                                   TextureFormat::Rgba8Unorm,
                                                   TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                                   &albedo_img.to_rgba8().into_vec()[..]);

    let normal_key = registry.create_with_data(img_size, "sprite normal texture",
                                                   TextureFormat::Rgba8Unorm,
                                                   TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                                   &image::io::Reader::open(path.join("normal.png")).unwrap()
                                                       .decode().unwrap()
                                                       .to_rgba8().into_vec()[..]);

    let specular_key = registry.create_with_data(img_size, "sprite specular texture",
                                                     TextureFormat::Rgba8Unorm,
                                                     TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                                     &image::io::Reader::open(path.join("specular.png")).unwrap()
                                                         .decode().unwrap()
                                                         .to_rgba8().into_vec()[..]);

    let height_key = registry.create_with_data(img_size, "sprite height texture",
                                                   TextureFormat::Rgba8Unorm,
                                                   TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                                                   &image::io::Reader::open(path.join("height.png")).unwrap()
                                                       .decode().unwrap()
                                                       .to_rgba8().into_vec()[..]);

    let maps_bind_group = registry.add_bind_group(BindGroupDescriptor {
        label: Some("sprite maps bind group"),
        layout: &GLOBALS.get().sprite_maps_bind_layout,
        entries: &[
            BindGroupEntry { binding: 0, resource: BindingResource::Sampler(&maps_sampler) },
            BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&*registry.find(albedo_key).unwrap().view()) },
            BindGroupEntry { binding: 2, resource: BindingResource::TextureView(&*registry.find(normal_key).unwrap().view()) },
            BindGroupEntry { binding: 3, resource: BindingResource::TextureView(&*registry.find(specular_key).unwrap().view()) },
            BindGroupEntry { binding: 4, resource: BindingResource::TextureView(&*registry.find(height_key).unwrap().view()) },
        ],
    });

    registry.find_mut(albedo_key).unwrap().replace_bind_group_idx(maps_bind_group);
    registry.find_mut(normal_key).unwrap().replace_bind_group_idx(maps_bind_group);
    registry.find_mut(specular_key).unwrap().replace_bind_group_idx(maps_bind_group);
    registry.find_mut(height_key).unwrap().replace_bind_group_idx(maps_bind_group);

    TextureMapSet {
        size: img_size,
        albedo: albedo_key,
        ao: albedo_key,
        normal: normal_key,
        specular: specular_key,
        height: height_key,
        extras: vec![],
        bind_group_idx: maps_bind_group,
    }
}


#[derive(Debug)]
/// Contains information about the sprite and lighting being displayed
pub struct Scene {
    pub textures: TextureMapSet,
    pub lighting: LightingInfo,
}

impl Scene {
    fn create(textures: TextureMapSet) -> SimpleCell<Self> {
        SimpleCell::new(Scene {
            textures,
            lighting: LightingInfo {
                enable_light_parallax: false,
                lights: vec![
                    Light {
                        position: Point2::new(-16.0, -16.0),
                        height: 50.0,
                        color: Color::white(ColorSpace::RGBA).with_alpha(2.0),
                        gizmo_hovered: false,
                        falloff_exp: 2.0,
                        enable_falloff: true,
                        drag_state: DragState::new(),
                        diffuse: 1.0,
                        specular: 1.0
                    }
                ],
                global_ambient: 0.05,
                global_diffuse: 0.475,
                global_specular: 0.475
            }
        })
    }

    pub fn from_sprite_path(path: PathBuf, registry: &mut TextureRegistry) -> SimpleCell<Self> {
        let textures = TEMP_create_texture_map_set(path, registry);
        Self::create(textures)
    }
}
