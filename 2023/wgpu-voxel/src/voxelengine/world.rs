pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    data: u32,
}

pub enum MaterialType {
    Absent,
    Rough(u8),
    Emissive(u8),
}

impl MaterialType {
    fn to_byte(&self) -> u8 {
        match self {
            MaterialType::Absent => 0,
            MaterialType::Rough(v) => 0b01 | (v >> 2),
            MaterialType::Emissive(v) => 0b11 | (v >> 2),
        }
    }
}

impl Material {
    pub fn new(color: Color, t: MaterialType) -> Self {
        let material = Material { data: 0 };
        let material = material.set_type(t);
        material.set_color(color)
    }

    pub fn set_type(&self, t: MaterialType) -> Self {
        Material {
            data: ((self.data >> 8) << 8) | t.to_byte() as u32,
        }
    }

    pub fn set_color(&self, c: Color) -> Self {
        Material {
            data: (self.data & 0xFF)
                | ((c.r as u32) << 24)
                | ((c.g as u32) << 16)
                | ((c.b as u32) << 8),
        }
    }
}

const CHUNK_DIM: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Chunk {
    location: [i32; 3],
    _pad0: i32,
    data: [Material; CHUNK_DIM * CHUNK_DIM * CHUNK_DIM],
}

impl Chunk {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let mut chunk: Chunk = unsafe { std::mem::MaybeUninit::uninit().assume_init() }; // Rust can't zero init big slices.
        // chunk.data = chunk
        //     .data
        //     .map(|_| Material::new(Color::new(0, 0, 0), MaterialType::Absent));
        chunk.location = [x, y, z];
        chunk
    }

    pub fn set_material(&mut self, x: i32, y: i32, z: i32, material: Material) {
        self.data[x as usize + y as usize * CHUNK_DIM + z as usize * CHUNK_DIM * CHUNK_DIM] =
            material;
    }
}
