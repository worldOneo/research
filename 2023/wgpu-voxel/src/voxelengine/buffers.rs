use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use super::world::{self};

pub struct Data<T> {
    pub data: T,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInputData {
    pub dim: [f32; 2],
    _pad0: [f32; 2],
    pub camera: [f32; 3],
    _pad1: [f32; 1],
    pub dir: [f32; 2],
    pub frame: u32,
    _pad3: [f32; 4],
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
                visibility: wgpu::ShaderStages::FRAGMENT,
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
    chunk.set_material(
        3,
        3,
        1,
        world::Material::new(
            world::Color::new(100, 100, 200),
            world::MaterialType::Rough(255),
        ),
    );
    chunk.set_material(
        3,
        2,
        2,
        world::Material::new(
            world::Color::new(100, 100, 200),
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
                visibility: wgpu::ShaderStages::FRAGMENT,
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
