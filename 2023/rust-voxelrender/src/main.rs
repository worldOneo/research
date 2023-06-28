use std::{ops::Mul, path::Path, thread, time::Instant};

use image::{ImageBuffer, Rgb};

#[derive(Default, Debug, Clone, Copy)]
struct Point {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

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
}

fn min_or<T>(a: f64, b: f64, at: T, bt: T) -> (f64, T) {
    if a < b {
        return (a, at);
    }
    return (b, bt);
}

#[derive(Debug)]
struct Cube {
    pos: Point,
    fpos: Vec3,
    size: i64,
}

impl Cube {
    fn new(x: i64, y: i64, z: i64, size: i64) -> Cube {
        let p = Point::new(x, y, z);
        Cube {
            pos: p,
            fpos: p.to_vec3(),
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

    fn contains(&self, p: &Point) -> bool {
        !(self.pos.x > p.x
            || self.pos.x + self.size <= p.x
            || self.pos.y > p.y
            || self.pos.y + self.size <= p.y
            || self.pos.z > p.z
            || self.pos.z + self.size <= p.z)
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

    fn max_border_dist(&self, p: Point) -> i64 {
        (p.x - self.pos.x)
            .max(p.y - self.pos.y)
            .max(p.z - self.pos.z)
            .max(self.pos.x + self.size - p.x)
            .max(self.pos.y + self.size - p.y)
            .max(self.pos.z + self.size - p.z)
    }
}

impl Point {
    fn new(x: i64, y: i64, z: i64) -> Self {
        Point { x, y, z }
    }

    fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }

    fn distance_to(&self, p: &Vec3) -> f64 {
        let pos = self.to_vec3();
        let dx = (pos.x - p.x).max(p.x - (pos.x + 1.)).max(0.);
        let dy = (pos.y - p.y).max(p.y - (pos.y + 1.)).max(0.);
        let dz = (pos.z - p.z).max(p.z - (pos.z + 1.)).max(0.);

        return (dx * dx + dy * dy + dz * dz).sqrt();
    }

    fn voxel_center(&self) -> Vec3 {
        Vec3::new(
            self.x as f64 + 0.5,
            self.y as f64 + 0.5,
            self.z as f64 + 0.5,
        )
    }
}

struct Octree<T> {
    data: OctreeData<T>,
    bounds: Cube,
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
struct RoughVoxel {
    color: Color,
    roughness: u8,
}

#[derive(Debug, Clone, Copy)]
struct EmissionVoxel {
    color: Color,
    emission: u8,
}

impl EmissionVoxel {
    fn new(color: Color, emission: u8) -> Self {
        Self { color, emission }
    }
}

impl RoughVoxel {
    fn new(color: Color, roughness: u8) -> Self {
        Self { color, roughness }
    }
}

enum OctreeData<T> {
    Empty,
    Voxel(T),
    Split(Box<[Octree<T>; 8]>),
}

impl<T> Octree<T>
where
    T: Clone,
{
    fn new(bounds: Cube) -> Self {
        Octree {
            data: OctreeData::Empty,
            bounds,
        }
    }

    fn insert(&mut self, position: Point, voxel: T) {
        if !self.bounds.contains(&position) {
            return;
        }
        match &mut self.data {
            OctreeData::Split(children) => {
                children
                    .iter_mut()
                    .for_each(|c| c.insert(position, voxel.clone()));
            }
            OctreeData::Empty => {
                if self.bounds.size == 1 {
                    self.data = OctreeData::Voxel(voxel);
                } else {
                    self.split();
                    self.insert(position, voxel);
                }
            }
            OctreeData::Voxel(_) => {
                self.data = OctreeData::Voxel(voxel);
            }
        }
    }

    fn split(&mut self) {
        let Cube {
            pos: Point { x, y, z },
            fpos: _,
            size,
        } = self.bounds;
        let n = size / 2;
        self.data = OctreeData::Split(Box::new([
            Self::new(Cube::new(x + 0, y + 0, z + 0, n)),
            Self::new(Cube::new(x + 0, y + 0, z + n, n)),
            Self::new(Cube::new(x + 0, y + n, z + 0, n)),
            Self::new(Cube::new(x + 0, y + n, z + n, n)),
            Self::new(Cube::new(x + n, y + 0, z + 0, n)),
            Self::new(Cube::new(x + n, y + 0, z + n, n)),
            Self::new(Cube::new(x + n, y + n, z + 0, n)),
            Self::new(Cube::new(x + n, y + n, z + n, n)),
        ]));
    }

    fn index_of(&self, p: &Vec3) -> usize {
        let hs = (self.bounds.size / 2) as f64;
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

    fn find_closest(&self, p: &Vec3, dir: &Vec3) -> (Option<&T>, f64) {
        match &self.data {
            OctreeData::Split(subtrees) => {
                let tree = &subtrees[self.index_of(p)];
                return tree.find_closest(p, dir);
            }
            OctreeData::Voxel(v) => {
                return (Some(v), 0.);
            }
            OctreeData::Empty => {
                return (None, self.bounds.max_marchable_distance(p, dir));
            }
        }
    }

    fn query<'a, F>(&'a self, p: &Vec3, rad: f64, mut f: F)
    where
        F: FnMut(&T, &Cube) -> (),
    {
        self.query_r(p, rad, &mut f)
    }

    fn query_r<'a, F>(&'a self, p: &Vec3, rad: f64, f: &mut F)
    where
        F: FnMut(&T, &Cube) -> (),
    {
        if !self.bounds.containsf(p) && self.bounds.distance_to(p) > rad {
            return;
        }
        match &self.data {
            OctreeData::Empty => {}
            OctreeData::Voxel(v) => f(v, &self.bounds),
            OctreeData::Split(tree) => tree.iter().for_each(|t| t.query_r(p, rad, f)),
        }
    }
}

struct LightingTree {
    split: Option<Box<[LightingTree; 8]>>,
    bounds: Cube,
    lights: Vec<Point>,
}

impl LightingTree {
    fn new(bounds: Cube) -> Self {
        LightingTree {
            split: None,
            lights: vec![],
            bounds,
        }
    }

    fn insert(&mut self, position: Point, emission_strength: u8) {
        let strength = emission_strength_from_u8(emission_strength);
        let border_brightness =
            strength / (self.bounds.max_border_dist(position) - 1).pow(2) as f64;
        if border_brightness > MIN_LIGHTING {
            self.lights.push(position);
            return;
        }
        if self.bounds.size == 1 {
            return;
        }
        let child = self.index_of(&position);
        self.split();
        if let Some(c) = &mut self.split {
            c[child].insert(position, emission_strength);
        }
    }

    fn split(&mut self) {
        if let Some(_) = self.split {
            return;
        }
        let Cube {
            pos: Point { x, y, z },
            fpos: _,
            size,
        } = self.bounds;
        let n = size / 2;
        self.split = Some(Box::new([
            Self::new(Cube::new(x + 0, y + 0, z + 0, n)),
            Self::new(Cube::new(x + 0, y + 0, z + n, n)),
            Self::new(Cube::new(x + 0, y + n, z + 0, n)),
            Self::new(Cube::new(x + 0, y + n, z + n, n)),
            Self::new(Cube::new(x + n, y + 0, z + 0, n)),
            Self::new(Cube::new(x + n, y + 0, z + n, n)),
            Self::new(Cube::new(x + n, y + n, z + 0, n)),
            Self::new(Cube::new(x + n, y + n, z + n, n)),
        ]));
    }

    fn index_of(&self, p: &Point) -> usize {
        let hs = self.bounds.size / 2;
        let mut idx = 0;
        if p.x >= self.bounds.pos.x + hs {
            idx |= 0b100;
        }
        if p.y >= self.bounds.pos.y + hs {
            idx |= 0b010;
        }
        if p.z >= self.bounds.pos.z + hs {
            idx |= 0b001;
        }
        idx
    }

    fn index_of_f(&self, p: &Vec3) -> usize {
        let hs = (self.bounds.size / 2) as f64;
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
        F: FnMut(&Point) -> (),
    {
        self.query_r(p, &mut f)
    }

    fn query_r<'a, F>(&'a self, p: &Vec3, f: &mut F)
    where
        F: FnMut(&Point) -> (),
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
const MIN_LIGHTING: f64 = 1. / 300.;
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

type MatTree = Octree<RoughVoxel>;
type EmissionTree = Octree<EmissionVoxel>;

fn emission_strength_from_u8(u: u8) -> f64 {
    (2_f64).powf(u as f64 / 16.)
}

fn direct_color(
    origin: &Vec3,
    dir: &Vec3,
    tree: &MatTree,
    ltree: &EmissionTree,
    lights: &LightingTree,
    bounces: usize,
) -> Vec3 {
    let px_color = Vec3::new(0., 0., 0.);
    if bounces == 0 {
        return px_color;
    }
    let (voxel, solidpos, _) = cast_to_hit(*origin, &dir, tree);

    // Render light first because it is faster
    let solid_dist = origin.sub(&solidpos).len();
    let lpos = origin.clone();
    if let (Some(v), lpos, _) = cast_to_hit(lpos, &dir, ltree) {
        if origin.sub(&lpos).len() < solid_dist {
            let light_strength = emission_strength_from_u8(v.emission);
            let adjusted_color = color_to_f(&v.color).mulf(light_strength);
            return adjusted_color;
        }
    }

    if let Some(v) = voxel {
        let voxpos = solidpos.clone();
        let normal = solidpos.prob_voxel_norm(&dir);
        let albedo = color_to_f(&v.color);
        let direct_light_pos = solidpos.add(&dir.mulf(-SOLID_POS_PUSH));
        let mut currentc = Vec3::new(0., 0., 0.);
        let color = &mut currentc;
        lights.query(&direct_light_pos, |lightvoxel| {
                    let dest = lightvoxel.voxel_center();
                    let vec_to_dest = dest.sub(&direct_light_pos);
                    let dist_to_dest = vec_to_dest.len();
                    let dir = vec_to_dest.normalized();
                    let (_, solid_hit_pos, _) = cast_to_hit(direct_light_pos, &dir, tree);
                    let (lvoxel, light_hit_pos, status) =
                        cast_to_hit(direct_light_pos, &dir, ltree);
                    if let None = lvoxel {
                        if status != CastStatus::InsufficientSteps {
                            panic!("\nMissed Light - Blocked: {:?} Light: {:?}\n Voxel: {:?}\n Dest: {:?}\n LPos: {:?}\n BPos: {:?}\n Start: {:?}\n Normal: {:?}\n Dir: {:?}\n", solid_hit_pos.sub(&direct_light_pos).len(), light_hit_pos.sub(&direct_light_pos).len(), voxpos, dest, light_hit_pos, solid_hit_pos, direct_light_pos, normal, dir);
                        }
                        return;
                    }
                    let emission_voxel = lvoxel.unwrap();
                    let light_dist = light_hit_pos.sub(&direct_light_pos).len();
                    let solid_dist = solid_hit_pos.sub(&direct_light_pos).len();
                    if light_hit_pos.voxel_equals(&dest) && light_dist < solid_dist {
                        // color += e.emission * e.color * albedo * (normal \cdot dir)
                        let emission_strength = emission_strength_from_u8(emission_voxel.emission)
                            / ((dist_to_dest - 0.5).powi(2))
                            * normal.dot(&dir);
                        let mixed_color = albedo.mulf(emission_strength);
                        *color = color.add(&color_to_f(&emission_voxel.color).mul(&mixed_color));
                    }
                });
        if v.roughness < 255 {
            // r = d - 2(d \dot n)n
            let reflection = dir.sub(&normal.mulf(&dir.dot(&normal) * 2.));
            let additional_color = direct_color(
                &direct_light_pos,
                &reflection,
                tree,
                ltree,
                lights,
                bounces - 1,
            );
            let reflected_back = 1. - (1. / (2_f64).powf(2. - v.roughness as f64 / 128.));
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
    ltree: &EmissionTree,
    lights: &LightingTree,
) {
    let camera = Vec3::new(CAMERA_SHAKE, CAMERA_SHAKE, CAMERA_SHAKE);
    let unit = w / workers;
    let start = unit * worker;
    let stop = (worker + 1) * unit;
    for y in ((start)..(stop)).rev() {
        for x in (0..h).rev() {
            let px = (w * (y - start) + x) as usize;
            let dir =
                Vec3::new(x as f64 + CAMERA_SHAKE, y as f64 + CAMERA_SHAKE, 600.).normalized();
            let direct_color = direct_color(&camera, &dir, tree, ltree, lights, 6);
            buf[px] = f_to_color(&direct_color);
        }
    }
}

fn image1(solids: &mut MatTree, emissions: &mut EmissionTree, light: &mut LightingTree) {
    for x in 3..=6 {
        for y in 3..=6 {
            solids.insert(Point::new(x, y, 13), RoughVoxel::new([200, 200, 200], 50));
        }
    }
    emissions.insert(
        Point::new(8, 6, 13),
        EmissionVoxel::new([100, 200, 255], 30),
    );
    light.insert(Point::new(8, 6, 13), 30);

    solids.insert(Point::new(5, 5, 10), RoughVoxel::new([240, 130, 130], 255));
    solids.insert(Point::new(5, 6, 10), RoughVoxel::new([240, 130, 130], 255));

    emissions.insert(Point::new(7, 6, 9), EmissionVoxel::new([100, 200, 100], 30));
    emissions.insert(
        Point::new(2, 2, 12),
        EmissionVoxel::new([255, 255, 255], 50),
    );

    light.insert(Point::new(7, 6, 9), 30);
    light.insert(Point::new(2, 2, 12), 50);

    for x in 0..30 {
        for z in 5..50 {
            solids.insert(Point::new(x, 7, z), RoughVoxel::new([255, 255, 255], 230));
        }
    }
}

fn image2(solids: &mut MatTree, emissions: &mut EmissionTree, light: &mut LightingTree) {
    for x in (4..64).step_by(4) {
        for z in (4..64).step_by(4) {
            for y in (4..64).step_by(4) {
                if x % 8 == 0 && y % 8 == 0 && z % 8 == 0 {
                    emissions.insert(
                        Point::new(x, y, z),
                        EmissionVoxel::new(
                            [
                                (x % 256).min(100) as u8,
                                (y % 256).min(100) as u8,
                                (z % 256).min(100) as u8,
                            ],
                            50,
                        ),
                    );
                    light.insert(Point::new(x, y, z), 50);
                } else {
                    solids.insert(Point::new(x, y, z), RoughVoxel::new([200, 200, 200], 255));
                }
            }
        }
    }
}

fn main() {
    let mut ltree = Octree::new(Cube::new(-1, -1, -1, 128));
    let mut tree = Octree::new(Cube::new(-1, -1, -1, 128));
    let mut lights = LightingTree::new(Cube::new(-1, -1, -1, 128));

    image2(&mut tree, &mut ltree, &mut lights);

    let now = Instant::now();
    let mut buffer = ImageBuffer::new(600, 600);
    let mut data = Box::new([[0, 0, 0] as Color; 600 * 600]);
    let thread_count = 12;
    let chunks = data.chunks_mut((600 / thread_count) * 600);
    let treeref = &tree;
    let ltreeref = &ltree;
    let lighting = &lights;
    thread::scope(|s| {
        chunks.enumerate().for_each(|(i, d)| {
            s.spawn(move || {
                render(
                    d,
                    600,
                    600,
                    thread_count as u32,
                    i as u32,
                    treeref,
                    ltreeref,
                    lighting,
                )
            });
        })
    });
    println!("Elapsed: {:.2?}", now.elapsed());

    for y in 0..600 {
        for x in 0..600 {
            let color = data[(600 * y + x) as usize];
            buffer.put_pixel(x, y, Rgb(color));
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    buffer.save(&Path::new("out.png")).unwrap();
}
