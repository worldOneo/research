use std::{path::Path, thread, time::Instant};

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

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
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

    fn prob_voxel_norm(&self) -> Vec3 {
        let min = self.x - self.x.floor();
        let norm = Vec3::new(-1., 0., 0.);
        let (min, norm) = min_or(
            min,
            self.x.floor() + 1. - self.x,
            norm,
            Vec3::new(1., 0., 0.),
        );
        let (min, norm) = min_or(min, self.y - self.y.floor(), norm, Vec3::new(0., -1., 0.));
        let (min, norm) = min_or(
            min,
            self.y.floor() + 1. - self.y,
            norm,
            Vec3::new(0., 1., 0.),
        );
        let (min, norm) = min_or(min, self.z - self.z.floor(), norm, Vec3::new(0., 0., -1.));
        let (_, norm) = min_or(
            min,
            self.z.floor() + 1. - self.z,
            norm,
            Vec3::new(0., 0., 1.),
        );
        return norm;
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
        let bpos = self.pos.to_vec3();
        let size = self.size as f64;
        let dx = if d.x > 0. {
            bpos.x + size - p.x
        } else {
            p.x - bpos.x
        };
        let dy = if d.y > 0. {
            bpos.y + size - p.y
        } else {
            p.y - bpos.y
        };
        let dz = if d.z > 0. {
            bpos.z + size - p.z
        } else {
            p.z - bpos.z
        };

        // dx = V_x * x solve for x: x = dx / Vx
        (dx / d.x).abs().min((dy / d.y).abs()).min((dz / d.z).abs())
    }

    fn center(&self) -> Vec3 {
        let s2 = self.size as f64 / 2.;
        Vec3::new(self.fpos.x + s2, self.fpos.y + s2, self.fpos.z + s2)
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

fn f_to_color(f: &Vec3) -> Color {
    [(f.x * 255.) as u8, (f.y * 255.) as u8, (f.z * 255.) as u8]
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
        let hs = self.bounds.size as f64 * 0.5;
        let mut idx = 0;
        if p.x > self.bounds.fpos.x + hs {
            idx |= 0b100;
        }
        if p.y > self.bounds.fpos.y + hs {
            idx |= 0b010;
        }
        if p.z > self.bounds.fpos.z + hs {
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

const MAX_SAMPLE_STEPS: usize = 1000;
const MAX_DISTANCE: f64 = 100.;
const MIN_DISTANCE: f64 = 0.0001;
const PUSH_ANALAYZE_DISTANCE: f64 = 0.000000001;

fn cast_to_hit<T>(pos: &mut Vec3, dir: &Vec3, tree: &Octree<T>) -> Option<T>
where
    T: Clone,
{
    let dir_len = dir.len();
    let mut total_len = 0.;
    for _ in 0..MAX_SAMPLE_STEPS {
        if !tree.bounds.containsf(&pos) {
            return None;
        }
        let (v, d) = tree.find_closest(&pos, &dir);
        if let Some(v) = v {
            return Some(v.clone());
        }
        if d < 0. {
            panic!("d = {d}");
        }
        *pos = pos.add(&dir.mulf(d + PUSH_ANALAYZE_DISTANCE));
        total_len += dir_len * (d + PUSH_ANALAYZE_DISTANCE);
        if total_len > MAX_DISTANCE {
            return None;
        }
    }
    println!("Step limit");
    return None;
}

type MatTree = Octree<RoughVoxel>;
type LightTree = Octree<EmissionVoxel>;

fn render(
    buf: &mut [Color],
    w: u32,
    h: u32,
    workers: u32,
    worker: u32,
    tree: &MatTree,
    ltree: &LightTree,
) {
    let unit = w / workers;
    let start = unit * worker;
    let stop = (worker + 1) * unit;
    let mut px = 0;
    for i in (start)..(stop) {
        for j in 0..h {
            let mut pos = Vec3::new(0., 0., 0.);
            let dir = Vec3::new(i as f64, j as f64, 600.).normalized();
            if let Some(v) = cast_to_hit(&mut pos, &dir, tree) {
                // let level = ((1. / (pos.len() as f64 / 7.)).powi(2) * 65536.) as u16;
                // buf[px] = v.color;
                let voxpos = pos.clone();
                let normal = pos.prob_voxel_norm();
                let albedo = color_to_f(&v.color);
                let mut currentc = Vec3::new(0., 0., 0.);
                let color = &mut currentc;
                ltree.query(&pos, 100., |e, lightvox| {
                    let dest = lightvox.center();
                    let pos = pos.add(&normal.mulf(5. * MIN_DISTANCE));
                    let dir = dest.sub(&pos).normalized();
                    let mut posmat = pos.clone();
                    cast_to_hit(&mut posmat, &dir, tree);
                    let mut poslight = pos.clone();
                    if let None = cast_to_hit(&mut poslight, &dir, ltree) {
                        panic!("\nBlocked: {:?} Light: {:?}\n Voxel: {:?}\n Dest: {:?}\n LPos: {:?}\n BPos: {:?}\n Start: {:?}\n Normal: {:?}\n Dir: {:?}\n", posmat.sub(&pos).len(), poslight.sub(&pos).len(), voxpos, dest, poslight, posmat, pos, normal, dir);
                    }
                    if poslight.sub(&pos).len() < posmat.sub(&pos).len() {
                        *color =
                             color.add(&color_to_f(&e.color).mul(&albedo.mulf(normal.dot(&dir))));
                        // *color = Vec3::new(0., 255., 255.);
                        // println!("Light hit: {:?} from {:?} to {:?}", e, pos, poslight);
                    } else {
                        // println!("Blocked: {:?} Light: {:?}\n LPos: {:?}\n BPos: {:?}\n Start: {:?}\n Normal: {:?}", posmat.sub(&pos).len(), poslight.sub(&pos).len(), poslight, posmat, pos, normal);
                    }
                });
                buf[px] = f_to_color(color);
            }
            if let Some(v) = cast_to_hit(&mut pos, &dir, ltree) {
                buf[px] = v.color;
            }
            px += 1;
        }
    }
}

fn main() {
    let mut ltree = Octree::new(Cube::new(-1, -1, -1, 128));
    let mut tree = Octree::new(Cube::new(-1, -1, -1, 128));

    tree.insert(Point::new(5, 5, 10), RoughVoxel::new([200, 200, 200], 255));
    tree.insert(Point::new(5, 6, 10), RoughVoxel::new([200, 200, 200], 100));
    ltree.insert(
        Point::new(5, 5, 9),
        EmissionVoxel::new([100, 200, 100], 255),
    );

    for x in 0..30 {
        for z in 5..50 {
            tree.insert(Point::new(x, 7, z), RoughVoxel::new([255, 255, 255], 255));
        }
    }
    let now = Instant::now();
    let mut buffer = ImageBuffer::new(600, 600);
    let mut data = Box::new([[0, 0, 0] as Color; 600 * 600]);
    let thread_count = 1;
    let chunks = data.chunks_mut((600 / thread_count) * 600);
    let treeref = &tree;
    let ltreeref = &ltree;
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
                )
            });
        })
    });
    println!("Elapsed: {:.2?}", now.elapsed());

    for i in 0..600 {
        for j in 0..600 {
            let color = data[(600 * i + j) as usize];
            buffer.put_pixel(i, j, Rgb(color));
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    buffer.save(&Path::new("out.png")).unwrap();
}
