use cgmath::Vector3;

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
    Opacity(u8),
}

impl MaterialType {
    fn to_byte(&self) -> u8 {
        match self {
            MaterialType::Absent => 0,
            MaterialType::Rough(v) => (0b01 << 6) | (v >> 2),
            MaterialType::Emissive(v) => (0b10 << 6) | (v >> 2),
            MaterialType::Opacity(v) => (0b11 << 6) | (v >> 2),
        }
    }

    fn weight_from_float(f: f32) -> u8 {
        ((f + 1.).log2() * 16.) as u8
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
            data: (self.data & !0xFF) | t.to_byte() as u32,
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
    pub fn is_present(&self) -> bool {
        self.data & 0b11000000 != 0
    }

    pub fn absent() -> Self {
        Material { data: 0 }
    }

    pub fn material_type(&self) -> MaterialType {
        // TODO: Exponential scaling
        let attrib = (self.data & 0b111111) as u8;
        let flag = self.data & (0b01 << 6) >> 6;
        match flag {
            0b01 => MaterialType::Rough(attrib),
            0b10 => MaterialType::Emissive(attrib),
            0b11 => MaterialType::Opacity(attrib),
            _ => MaterialType::Absent,
        }
    }

    fn rgb(&self) -> Vector3<f32> {
        if !self.is_present() {
            return Vector3::new(0., 0., 0.);
        }
        return Vector3::new(
            (self.data >> 8 & 0xFF) as f32,
            (self.data >> 16 & 0xFF) as f32,
            (self.data >> 24 & 0xFF) as f32,
        );
    }

    fn mix_lod_materials(
        a: Material,
        b: Material,
        c: Material,
        d: Material,
    ) -> (Material, Material) {
        let mut opacity = 0.;
        let mut present = 0;
        let mut rgb = Vector3::new(0., 0., 0.);
        let mut emissive_rgb = Vector3::new(0., 0., 0.);
        let mut emissive_strength = 0.;
        let mut emitting = 0;
        for m in [a, b, c, d] {
            if m.is_present() {
                rgb += m.rgb();
                present += 1;
            }
            if let MaterialType::Opacity(v) = m.material_type() {
                opacity += v as f32;
            }
            if let MaterialType::Emissive(v) = m.material_type() {
                emissive_rgb += m.rgb();
                emissive_strength += v as f32;
                emitting += 1;
            }
        }
        if present > 0 {
            rgb /= present as f32;
        }
        if emitting > 0 {
            emissive_rgb /= emitting as f32;
            emissive_strength /= emitting as f32;
        }
        opacity /= 4.;
        let color = Color::new(rgb.x as u8, rgb.y as u8, rgb.z as u8);

        let emissive_color = emissive_rgb / emitting as f32;
        let emissive_color = Color::new(
            emissive_color.x as u8,
            emissive_color.y as u8,
            emissive_color.z as u8,
        );

        return (
            Material::new(color, MaterialType::Opacity(opacity as u8)),
            Material::new(
                emissive_color,
                MaterialType::Emissive(emissive_strength as u8),
            ),
        );
    }
}

const CHUNK_DIM: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Chunk {
    location: [i32; 3],
    data: [Material; CHUNK_DIM * CHUNK_DIM * CHUNK_DIM],
    _pad0: u32,
}

impl Chunk {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let mut chunk: Chunk = unsafe { std::mem::MaybeUninit::zeroed().assume_init() }; // Rust can't zero init big slices.
        chunk.location = [x, y, z];
        chunk
    }

    pub fn set_material(&mut self, x: i32, y: i32, z: i32, material: Material) {
        self.data[x as usize + y as usize * CHUNK_DIM + z as usize * CHUNK_DIM * CHUNK_DIM] =
            material;
    }
}
