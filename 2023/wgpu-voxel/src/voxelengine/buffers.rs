use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use super::world::{self};

pub struct Data<T> {
    pub data: T,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: [f32; 3],
    _pad0: [f32; 1],
    pub dir: [f32; 2],
    _pad1: [f32; 2],
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInputData {
    pub dim: [f32; 2],
    _pad0: [f32; 2],
    pub camera: Camera,
    pub old_camera: Camera,
    pub frame: u32,
    _pad3: [f32; 3],
}

pub fn create_render_input_buffer(device: &Device) -> Data<RenderInputData> {
    let mut render_input = RenderInputData::default();
    render_input.dim = [0., 0.];

    let render_input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Render Input Buffer"),
        contents: bytemuck::cast_slice(&[render_input]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let render_input_binding_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("render_input_layout"),
        });

    let render_input_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("screen_bind_group"),
        layout: &render_input_binding_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: render_input_buffer.as_entire_binding(),
        }],
    });

    Data {
        data: render_input,
        buffer: render_input_buffer,
        bind_group: render_input_bind_group,
        layout: render_input_binding_group_layout,
    }
}

pub fn create_chunk_buffer(device: &Device) -> Data<world::Chunk> {
    let mut chunk = world::Chunk::new(0, 0, 0);
    chunk.set_material(
        3,
        1,
        1,
        world::Material::new(
            world::Color::new(200, 100, 100),
            world::MaterialType::Rough(255),
        ),
    );
    chunk.set_material(
        3,
        2,
        1,
        world::Material::new(
            world::Color::new(100, 200, 100),
            world::MaterialType::Rough(255),
        ),
    );
    for x in (0..8).step_by(2) {
        for y in 0..8 {
            let xoffset = if y % 2 == 0 { 0 } else { 1 };
            chunk.set_material(
                x + xoffset,
                y,
                1,
                world::Material::new(
                    world::Color::new(100, 100, 200),
                    world::MaterialType::Rough(255),
                ),
            );
        }
    }
    chunk.set_material(
        4,
        3,
        1,
        world::Material::new(
            world::Color::new(200, 100, 100),
            world::MaterialType::Rough(255),
        ),
    );
    chunk.set_material(
        2,
        3,
        1,
        world::Material::new(
            world::Color::new(100, 200, 100),
            world::MaterialType::Rough(255),
        ),
    );
    chunk.set_material(
        3,
        2,
        2,
        world::Material::new(
            world::Color::new(200, 200, 200),
            world::MaterialType::Emissive(50),
        ),
    );

    let chunk_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Chunk Buffer"),
        contents: bytemuck::cast_slice(&[chunk]),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let chunk_binding_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("chunk_layout"),
        });

    let chunk_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("chunk_bind_group"),
        layout: &chunk_binding_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: chunk_buffer.as_entire_binding(),
        }],
    });

    Data {
        data: chunk,
        buffer: chunk_buffer,
        bind_group: chunk_bind_group,
        layout: chunk_binding_group_layout,
    }
}

pub struct Texutures {
    pub svgf: wgpu::Texture,
    pub resampling: wgpu::Texture,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

pub fn create_svgf_buffer(device: &Device) -> Texutures {
    let svgf_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SVGF Buffer"),
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 3,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Uint,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba32Uint],
    });

    let resampling_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Resampling Buffer"),
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 2,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Uint,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba32Uint],
    });

    let texture_buffer_binding_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
            label: Some("texture_buffer_layout"),
        });

    let texture_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texture_buffer_bind_group"),
        layout: &texture_buffer_binding_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &svgf_buffer.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &resampling_buffer.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    });

    Texutures {
        svgf: svgf_buffer,
        resampling: resampling_buffer,
        bind_group: texture_buffer_bind_group,
        layout: texture_buffer_binding_group_layout,
    }
}