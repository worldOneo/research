use std::{f64::consts::PI, path::Path, thread, time::Instant};

use image::{ImageBuffer, Rgb};

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Vec3 {
    fn newi(x: i64, y: i64, z: i64) -> Vec3 {
        Vec3::new(x as f64, y as f64, z as f64)
    }

    fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn mixf(&self, other: &Vec3, a: f64) -> Vec3 {
        let b = (1. - a);
        Vec3::new(
            self.x * b + other.x * a,
            self.y * b + other.y * a,
            self.z * b + other.z * a,
        )
    }

    fn add(&self, v: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }

    fn sub(&self, v: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }

    fn mul(&self, v: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x * v.x,
            y: self.y * v.y,
            z: self.z * v.z,
        }
    }

    fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    fn powf(&self, f: f64) -> Vec3 {
        Vec3::new(self.x.powf(f), self.y.powf(f), self.z.powf(f))
    }

    fn mulf(&self, s: f64) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    fn normalized(&self) -> Vec3 {
        self.mulf(1. / self.len())
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn voxel_equals(&self, other: &Vec3) -> bool {
        self.x.floor() == other.x.floor()
            && self.y.floor() == other.y.floor()
            && self.z.floor() == other.z.floor()
    }

    fn prob_voxel_norm(&self, dir: &Vec3) -> Vec3 {
        let dir = dir.mulf(-1.);
        let voxel_position = Vec3::new(self.x.floor(), self.y.floor(), self.z.floor());

        // | <- voxel wall ->  |
        // |   x <- self       |
        // |    \  <- dir      |
        // |     \             |
        // |                   |
        //
        // [---] <- Distance to voxel wall near
        //     [-----------] <- Distance to voxel wall far
        //
        // The direction is inverse because self is inside the voxel and
        // we move beackwards to find the first wall intersection.
        //
        // Now we use the equation for lines in 3D:
        //        | self.x |       | dir.x |
        // g: x = | self.y | + r * | dir.y |
        //        | self.z |       | dir.z |
        //
        // Now we can set the equation for each dimension for the correct voxel wall
        //
        // voxel wall = self.dim + r * dir.dim
        //
        // Solve for r:
        // r = (voxel wall - self.dim) / dir.dim
        //
        // The normal is in the dimension which solves for the smallest r

        let mut dist_x = voxel_position.x;
        if dir.x > 0. {
            dist_x = voxel_position.x + 1.;
        }
        let mut dist_y = voxel_position.y;
        if dir.y > 0. {
            dist_y = voxel_position.y + 1.;
        }
        let mut dist_z = voxel_position.z;
        if dir.z > 0. {
            dist_z = voxel_position.z + 1.;
        }
        let step_x = (dist_x - self.x) / dir.x;
        let step_y = (dist_y - self.y) / dir.y;
        let step_z = (dist_z - self.z) / dir.z;

        let (min, norm) = (
            step_x,
            if dir.x > 0. {
                Vec3::new(1., 0., 0.)
            } else {
                Vec3::new(-1., 0., 0.)
            },
        );
        let (min, norm) = min_or(
            min,
            step_y,
            norm,
            if dir.y > 0. {
                Vec3::new(0., 1., 0.)
            } else {
                Vec3::new(0., -1., 0.)
            },
        );
        let (_, norm) = min_or(
            min,
            step_z,
            norm,
            if dir.z > 0. {
                Vec3::new(0., 0., 1.)
            } else {
                Vec3::new(0., 0., -1.)
            },
        );
        norm
    }

    fn rotate_z(&self, angle: f64) -> Vec3 {
        Vec3::new(
            self.x * angle.cos() - self.y * angle.sin(),
            self.x * angle.sin() + self.y * angle.cos(),
            self.z,
        )
    }

    fn rotate_y(&self, angle: f64) -> Vec3 {
        Vec3::new(
            self.x * angle.cos() - self.z * angle.sin(),
            self.y,
            -self.x * angle.sin() + self.z * angle.cos(),
        )
    }

    fn rotate_x(&self, angle: f64) -> Vec3 {
        Vec3::new(
            self.x,
            self.y * angle.cos() - self.z * angle.sin(),
            self.y * angle.sin() + self.z * angle.cos(),
        )
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn rotate_rel(&self, angle: f64, axis: &Vec3) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        self.mulf(cos)
            .add(&axis.cross(self).mulf(sin))
            .add(&axis.mulf(axis.dot(self)).mulf(1. - cos))
    }

    fn voxel_center(&self) -> Vec3 {
        Vec3::new(
            self.x.floor() + 0.5,
            self.y.floor() + 0.5,
            self.z.floor() + 0.5,
        )
    }
}

fn min_or<T>(a: f64, b: f64, at: T, bt: T) -> (f64, T) {
    if a < b {
        return (a, at);
    }
    return (b, bt);
}

#[derive(Debug, Clone, Copy)]
struct Cube {
    fpos: Vec3,
    size: f64,
}

impl Cube {
    fn new(x: f64, y: f64, z: f64, size: f64) -> Cube {
        Cube {
            fpos: Vec3::new(x, y, z),
            size,
        }
    }

    fn distance_to(&self, p: &Vec3) -> f64 {
        let pos = self.fpos;
        let s = self.size as f64;
        let dx = (pos.x - p.x).max(p.x - (pos.x + s)).max(0.);
        let dy = (pos.y - p.y).max(p.y - (pos.y + s)).max(0.);
        let dz = (pos.z - p.z).max(p.z - (pos.z + s)).max(0.);

        return (dx * dx + dy * dy + dz * dz).sqrt();
    }

    fn containsf(&self, p: &Vec3) -> bool {
        let pos = self.fpos;
        let size = self.size as f64;

        !(pos.x > p.x
            || pos.x + size <= p.x
            || pos.y > p.y
            || pos.y + size <= p.y
            || pos.z > p.z
            || pos.z + size <= p.z)
    }

    fn max_marchable_distance(&self, p: &Vec3, d: &Vec3) -> f64 {
        let bpos = self.fpos;
        let size = self.size as f64;
        let dx = if d.x > 0. { bpos.x + size } else { bpos.x };
        let dy = if d.y > 0. { bpos.y + size } else { bpos.y };
        let dz = if d.z > 0. { bpos.z + size } else { bpos.z };

        // dx = P_x + V_x * x solve for x: x = (dx - P_x) / Vx
        let min = ((dx - p.x) / d.x)
            .abs()
            .min(((dy - p.y) / d.y).abs())
            .min(((dz - p.z) / d.z).abs());
        min
    }

    fn center(&self) -> Vec3 {
        let s2 = self.size as f64 / 2.;
        Vec3::new(self.fpos.x + s2, self.fpos.y + s2, self.fpos.z + s2)
    }

    fn max_border_dist(&self, p: &Vec3) -> f64 {
        let s = self.size as f64;
        Vec3::new(
            (p.x - self.fpos.x).abs().max((self.fpos.x + s - p.x).abs()),
            (p.y - self.fpos.y).abs().max((self.fpos.y + s - p.y).abs()),
            (p.z - self.fpos.z).abs().max((self.fpos.z + s - p.z).abs()),
        )
        .len()
    }
}

struct Octree<T> {
    data: OOctree<T>,
    bounds: Cube,
}

struct OOctree<T> {
    data: OctreeData<T>,
}

type Color = [u8; 3];
fn color_to_f(c: &Color) -> Vec3 {
    Vec3::new(
        (c[0] as f64) / 255.,
        (c[1] as f64) / 255.,
        (c[2] as f64) / 255.,
    )
}

fn aces(x: f64) -> f64 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return (x * (a * x + b)) / (x * (c * x + d) + e).clamp(0.0, 1.0);
}

fn filmic_tone_mapping(color: f64) -> f64 {
    let color = (0_f64).max(color - 0.004);
    let color = (color * (6.2 * color + 0.5)) / (color * (6.2 * color + 1.7) + 0.06);
    return color;
}

fn white_preserving_luma_based_reinhard_tone_mapping(color: &Vec3) -> Vec3 {
    let white = 2.;
    let luma = color.dot(&Vec3::new(0.2126, 0.7152, 0.0722));
    let tone_mapped_luma = luma * (1. + luma / (white * white)) / (1. + luma);
    let color = color.mulf(tone_mapped_luma / luma);
    let inv_gamma = 1. / 2.2;
    let color = color.powf(inv_gamma);
    return color;
}

fn f_to_color(f: &Vec3) -> Color {
    [
        (filmic_tone_mapping(f.x) * 255.) as u8,
        (filmic_tone_mapping(f.y) * 255.) as u8,
        (filmic_tone_mapping(f.z) * 255.) as u8,
    ]
}

#[derive(Debug, Clone, Copy)]
enum VoxelMaterial {
    Rough { color: Color, roughness: u8 },
    Emission { color: Color, emission: u8 },
}

impl VoxelMaterial {
    fn rough(color: Color, roughness: u8) -> Self {
        Self::Rough { color, roughness }
    }

    fn emissive(color: Color, emission: u8) -> Self {
        Self::Emission { color, emission }
    }
}

enum OctreeData<T> {
    Empty,
    Voxel(T),
    Split(Box<[OOctree<T>; 8]>),
}

impl<T> Octree<T>
where
    T: Clone,
{
    fn new(bounds: Cube) -> Self {
        Octree {
            data: OOctree::new(),
            bounds,
        }
    }

    fn insert(&mut self, position: Vec3, voxel: T) {
        self.data.insert(&self.bounds, position, voxel);
    }

    fn find_closest(&self, p: &Vec3, dir: &Vec3) -> (Option<&T>, f64) {
        self.data.find_closest(&self.bounds, p, dir)
    }
}

impl<T> OOctree<T>
where
    T: Clone,
{
    fn new() -> Self {
        OOctree {
            data: OctreeData::Empty,
        }
    }

    fn insert(&mut self, bounds: &Cube, position: Vec3, voxel: T) {
        if !bounds.containsf(&position) {
            return;
        }
        let (idx, subbounds) = self.index_of(bounds, &position);
        match &mut self.data {
            OctreeData::Split(children) => {
                children[idx].insert(&subbounds, position, voxel);
            }
            OctreeData::Empty => {
                if bounds.size == 1. {
                    self.data = OctreeData::Voxel(voxel);
                } else {
                    self.split();
                    self.insert(&bounds, position, voxel);
                }
            }
            OctreeData::Voxel(_) => {
                self.data = OctreeData::Voxel(voxel);
            }
        }
    }

    fn split(&mut self) {
        self.data = OctreeData::Split(Box::new([
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
        ]));
    }

    fn index_of(&self, bounds: &Cube, p: &Vec3) -> (usize, Cube) {
        let hs = bounds.size * 0.5;
        let mut bounds = *bounds;
        let mut idx = 0;
        bounds.size = hs;
        if p.x >= bounds.fpos.x + hs {
            idx |= 0b100;
            bounds.fpos.x += hs;
        }
        if p.y >= bounds.fpos.y + hs {
            idx |= 0b010;
            bounds.fpos.y += hs;
        }
        if p.z >= bounds.fpos.z + hs {
            idx |= 0b001;
            bounds.fpos.z += hs;
        }
        (idx, bounds)
    }

    fn find_closest(&self, bounds: &Cube, p: &Vec3, dir: &Vec3) -> (Option<&T>, f64) {
        match &self.data {
            OctreeData::Split(subtrees) => {
                let (idx, bounds) = self.index_of(bounds, p);
                let tree = &subtrees[idx];
                return tree.find_closest(&bounds, p, dir);
            }
            OctreeData::Voxel(v) => {
                return (Some(v), 0.);
            }
            OctreeData::Empty => {
                return (None, bounds.max_marchable_distance(p, dir));
            }
        }
    }
}

struct LightingTree {
    split: Option<Box<[LightingTree; 8]>>,
    bounds: Cube,
    lights: Vec<Vec3>,
}

const USEFULL_LIGHT_LIMIT: f64 = 1. / 100.;
const NOT_USEFULL_LIGHT_LIMIT: f64 = 1. / 20.;

impl LightingTree {
    fn new(bounds: Cube) -> Self {
        LightingTree {
            split: None,
            lights: vec![],
            bounds,
        }
    }

    fn insert(&mut self, position: Vec3, emission_strength: u8) {
        // if minimimum possible ilumination is usefull insert (=> every voxel in this octet could be iluminated)
        // else if maximum possible ilumination not usefull break (no child could ever be iluminated)
        // else offer to children
        let strength = emission_strength_from_u8(emission_strength);

        let max_distance = self.bounds.max_border_dist(&position.voxel_center());
        let min_brightness = strength / max_distance.powi(2);
        let is_min_usefull = min_brightness > USEFULL_LIGHT_LIMIT;
        if is_min_usefull {
            self.lights.push(position);
            return;
        }

        let is_max_usefull = if self.bounds.containsf(&position) {
            true
        } else {
            (strength / self.bounds.distance_to(&position).powi(2)) > NOT_USEFULL_LIGHT_LIMIT
        };

        let is_single_voxel_size = self.bounds.size == 1.;

        if is_single_voxel_size || !is_max_usefull {
            return;
        }

        self.split();
        if let Some(c) = &mut self.split {
            c.iter_mut()
                .for_each(|c| c.insert(position, emission_strength));
        }
    }

    fn split(&mut self) {
        if let Some(_) = self.split {
            return;
        }
        let Cube {
            fpos: Vec3 { x, y, z },
            size,
        } = self.bounds;
        let n = size / 2.;
        let o = 0.;
        self.split = Some(Box::new([
            Self::new(Cube::new(x + o, y + o, z + o, n)),
            Self::new(Cube::new(x + o, y + o, z + n, n)),
            Self::new(Cube::new(x + o, y + n, z + o, n)),
            Self::new(Cube::new(x + o, y + n, z + n, n)),
            Self::new(Cube::new(x + n, y + o, z + o, n)),
            Self::new(Cube::new(x + n, y + o, z + n, n)),
            Self::new(Cube::new(x + n, y + n, z + o, n)),
            Self::new(Cube::new(x + n, y + n, z + n, n)),
        ]));
    }

    fn index_of(&self, p: &Vec3) -> usize {
        let hs = self.bounds.size / 2.;
        let mut idx = 0;
        if p.x >= self.bounds.fpos.x + hs {
            idx |= 0b100;
        }
        if p.y >= self.bounds.fpos.y + hs {
            idx |= 0b010;
        }
        if p.z >= self.bounds.fpos.z + hs {
            idx |= 0b001;
        }
        idx
    }

    fn index_of_f(&self, p: &Vec3) -> usize {
        let hs = self.bounds.size / 2.;
        let mut idx = 0;
        if p.x >= self.bounds.fpos.x + hs {
            idx |= 0b100;
        }
        if p.y >= self.bounds.fpos.y + hs {
            idx |= 0b010;
        }
        if p.z >= self.bounds.fpos.z + hs {
            idx |= 0b001;
        }
        idx
    }

    fn query<'a, F>(&'a self, p: &Vec3, mut f: F)
    where
        F: FnMut(&Vec3) -> (),
    {
        self.query_r(p, &mut f)
    }

    fn query_r<'a, F>(&'a self, p: &Vec3, f: &mut F)
    where
        F: FnMut(&Vec3) -> (),
    {
        for c in &self.lights {
            f(c);
        }
        let index = self.index_of_f(p);
        if let Some(c) = &self.split {
            c[index].query_r(p, f);
        }
    }
}

const MAX_SAMPLE_STEPS: usize = 100;
const MAX_DISTANCE: f64 = 1000.;
const CAMERA_SHAKE: f64 = 1e-3;
const SOLID_POS_PUSH: f64 = 2e-4;
const PUSH_ANALAYZE_DISTANCE: f64 = 1e-4;

#[derive(PartialEq)]
enum CastStatus {
    InsufficientSteps,
    OutOfTree,
    MaxDistance,
    Hit,
}

fn cast_to_hit<T>(mut pos: Vec3, dir: &Vec3, tree: &Octree<T>) -> (Option<T>, Vec3, CastStatus)
where
    T: Clone,
{
    let mut total_len = 0.;
    for _ in 0..MAX_SAMPLE_STEPS {
        if !tree.bounds.containsf(&pos) {
            return (None, pos, CastStatus::OutOfTree);
        }
        let (v, d) = tree.find_closest(&pos, &dir);
        if let Some(v) = v {
            return (Some(v.clone()), pos, CastStatus::Hit);
        }
        let buf = if d < PUSH_ANALAYZE_DISTANCE {
            PUSH_ANALAYZE_DISTANCE
        } else {
            d + PUSH_ANALAYZE_DISTANCE
        };
        pos = pos.add(&dir.mulf(buf));
        total_len += buf;
        if total_len > MAX_DISTANCE {
            return (None, pos, CastStatus::MaxDistance);
        }
    }
    return (None, pos, CastStatus::InsufficientSteps);
}

type MatTree = Octree<VoxelMaterial>;

fn emission_strength_from_u8(u: u8) -> f64 {
    (2_f64).powf(u as f64 / 16.)
}

fn direct_color(
    origin: &Vec3,
    dir: &Vec3,
    tree: &MatTree,
    lights: &LightingTree,
    bounces: usize,
) -> Vec3 {
    let px_color = Vec3::new(0., 0., 0.);
    if bounces == 0 {
        return px_color;
    }
    let (voxel, solidpos, _) = cast_to_hit(*origin, &dir, tree);

    // Render light first because it is faster
    if let Some(VoxelMaterial::Emission { color, emission }) = voxel {
        let light_strength = emission_strength_from_u8(emission);
        let adjusted_color = color_to_f(&color).mulf(light_strength);
        return adjusted_color;
    }

    if let Some(VoxelMaterial::Rough { color, roughness }) = voxel {
        let normal = solidpos.prob_voxel_norm(&dir);
        let albedo = color_to_f(&color);
        let direct_light_pos = solidpos.add(&dir.mulf(-SOLID_POS_PUSH));
        let mut currentc = Vec3::new(0., 0., 0.);
        let color = &mut currentc;
        lights.query(&direct_light_pos, |lightvoxel| {
            let dest = lightvoxel.voxel_center();
            let vec_to_dest = dest.sub(&direct_light_pos);
            let dist_to_dest = vec_to_dest.len();
            let dir = vec_to_dest.normalized();
            let (voxel, solid_hit_pos, status) = cast_to_hit(direct_light_pos, &dir, tree);
            if !solid_hit_pos.voxel_equals(&dest) {
                return;
            }

            if let None = voxel {
                if status != CastStatus::InsufficientSteps {
                    panic!("Missed Light");
                }
                return;
            }
            let emission_voxel = voxel.unwrap();
            if let VoxelMaterial::Emission { color: light_color, emission } = emission_voxel {
                // color += e.emission * e.color * albedo * (normal \cdot dir)
                let emission_strength = emission_strength_from_u8(emission)
                    / ((dist_to_dest - 0.5).powi(2))
                    * normal.dot(&dir);
                let mixed_color = albedo.mulf(emission_strength);
                *color = color.add(&color_to_f(&light_color).mul(&mixed_color));
            }
        });
        if roughness < 255 {
            // r = d - 2(d \dot n)n
            let reflection = dir.sub(&normal.mulf(&dir.dot(&normal) * 2.));
            let additional_color = direct_color(
                &direct_light_pos,
                &reflection,
                tree,
                lights,
                bounces - 1,
            );
            let reflected_back = 1. - (1. / (2_f64).powf(4. - roughness as f64 / 64.));
            return color.mixf(&additional_color, reflected_back);
        }
        return *color;
    }
    px_color
}

fn render(
    buf: &mut [Color],
    w: u32,
    h: u32,
    workers: u32,
    worker: u32,
    tree: &MatTree,
    lights: &LightingTree,
) {
    let camera = Vec3::new(-4. + CAMERA_SHAKE, -4. + CAMERA_SHAKE, -4. + CAMERA_SHAKE);
    let unit = h / workers;
    let start = unit * worker;
    let stop = (worker + 1) * unit;
    let scale = 1080.;
    let w2 = w as f64 / 2. + CAMERA_SHAKE;
    let h2 = h as f64 / 2. + CAMERA_SHAKE;

    for y in ((start)..(stop)).rev() {
        for x in (0..w).rev() {
            let px = (w * (y - start) + x) as usize;

            let camera_norm = Vec3::new(1., 0., 0.);
            let dir = Vec3::new(1., (x as f64 - w2) / scale, (y as f64 - h2) / scale).normalized();
            let dir = dir
                .rotate_rel(
                    PI / 7.,
                    &camera_norm.cross(&Vec3::new(0., 0., 1.)).normalized(),
                )
                .rotate_z(PI / 4.)
                .normalized();

            let direct_color = direct_color(&camera, &dir, tree, lights, 6);
            buf[px] = f_to_color(&direct_color);
        }
    }
}

fn image1(solids: &mut MatTree, light: &mut LightingTree) {
    for y in -2..=1 {
        for z in -3..0 {
            solids.insert(
                Vec3::newi(5, y, z),
                VoxelMaterial::rough([200, 200, 200], 150),
            );
        }
    }

    let blue_point = Vec3::newi(5, 3, -1);
    solids.insert(blue_point, VoxelMaterial::emissive([100, 200, 255], 30));
    light.insert(blue_point, 30);

    solids.insert(
        Vec3::newi(1, -1, -1),
        VoxelMaterial::rough([240, 130, 130], 254),
    );
    solids.insert(
        Vec3::newi(1, -1, -2),
        VoxelMaterial::rough([240, 130, 130], 254),
    );

    let green_light = Vec3::newi(0, 1, -1);
    solids.insert(green_light, VoxelMaterial::emissive([100, 200, 100], 30));
    light.insert(green_light, 30);

    let white_light = Vec3::newi(4, -3, -4);
    solids.insert(white_light, VoxelMaterial::emissive([255, 255, 255], 50));
    light.insert(white_light, 50);

    for x in -20..40 {
        for y in -20..40 {
            solids.insert(
                Vec3::newi(x, y, 0),
                VoxelMaterial::rough([255, 255, 255], 250),
            );
        }
    }
}

fn image2(solids: &mut MatTree, light: &mut LightingTree) {
    for x in (0..64).step_by(4) {
        for z in (0..64).step_by(4) {
            for y in (0..64).step_by(4) {
                if x % 8 == 4 && y % 8 == 4 && z % 8 == 4 {
                    solids.insert(
                        Vec3::newi(x, y, z),
                        VoxelMaterial::emissive(
                            [y.min(100) as u8, z.min(100) as u8, x.min(100) as u8],
                            50,
                        ),
                    );
                    light.insert(Vec3::newi(x, y, z), 50);
                } else {
                    solids.insert(
                        Vec3::newi(x, y, z),
                        VoxelMaterial::rough([200, 200, 200], 255),
                    );
                }
            }
        }
    }
}

const IMG_W: u32 = 1920;
const IMG_H: u32 = 1080;

fn main() {
    let mut tree = Octree::new(Cube::new(-64., -64., -64., 128.));
    let mut lights = LightingTree::new(Cube::new(-64., -64., -64., 128.));

    let now = Instant::now();
    image1(&mut tree, &mut lights);
    let scene_build = now.elapsed();
    println!("Scene build: {scene_build:?}");

    let mut buffer = ImageBuffer::new(IMG_W, IMG_H);
    let mut data = Box::new([[0, 0, 0] as Color; (IMG_W * IMG_H) as usize]);
    let thread_count = 12;
    let chunks = data.chunks_mut((IMG_H as usize / thread_count) * IMG_W as usize);
    let treeref = &tree;
    let lighting = &lights;
    thread::scope(|s| {
        chunks.enumerate().for_each(|(i, d)| {
            s.spawn(move || {
                render(
                    d,
                    IMG_W,
                    IMG_H,
                    thread_count as u32,
                    i as u32,
                    treeref,
                    lighting,
                )
            });
        })
    });
    println!("Render: {:.2?}", now.elapsed());

    for y in 0..IMG_H {
        for x in 0..IMG_W {
            let color = data[(IMG_W * y + x) as usize];
            buffer.put_pixel(x, y, Rgb(color));
        }
    }
    let elapsed = now.elapsed();
    println!("Image Generation: {:.2?}", elapsed);
    buffer.save(&Path::new("out.png")).unwrap();
}
