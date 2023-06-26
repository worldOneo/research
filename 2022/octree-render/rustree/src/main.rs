use std::path::Path;

use image::{EncodableLayout, ImageBuffer, Rgb};

#[derive(Default, Debug, Clone, Copy)]
struct Point {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

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
            z: self.y - v.z,
        }
    }

    fn mul(&self, s: f64) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    fn normalized(&self) -> Vec3 {
        self.mul(1. / self.len())
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

#[derive(Debug)]
struct Cube {
    pub pos: Point,
    pub size: i64,
}

impl Cube {
    fn new(x: i64, y: i64, z: i64, size: i64) -> Cube {
        Cube {
            pos: Point::new(x, y, z),
            size,
        }
    }

    fn distance_to(&self, p: &Vec3) -> f64 {
        let pos = self.pos.to_vec3();
        let s = self.size as f64;
        let dx = (pos.x - p.x).max(p.x - (pos.x + s)).max(0.);
        let dy = (pos.y - p.y).max(p.y - (pos.y + s)).max(0.);
        let dz = (pos.z - p.z).max(p.z - (pos.z + s)).max(0.);

        return (dx * dx + dy * dy + dz * dz).sqrt();
    }

    fn containsf(&self, p: &Vec3) -> bool {
        let pos = self.pos.to_vec3();
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
        (dx / d.x).min(dy / d.y).min(dz / d.z)
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

const OCTREE_CAPACITY: usize = 4;

struct Octree {
    data: OctreeData,
    bounds: Cube,
}

enum OctreeData {
    Empty,
    Voxel,
    Split(Box<[Octree; 8]>),
}

impl Octree {
    fn new(bounds: Cube) -> Self {
        Octree {
            data: OctreeData::Empty,
            bounds,
        }
    }

    fn insert(&mut self, voxel: Point) {
        if !self.bounds.contains(&voxel) {
            return;
        }
        match &mut self.data {
            OctreeData::Split(children) => {
                children.iter_mut().for_each(|c| c.insert(voxel));
            }
            OctreeData::Empty => {
                if self.bounds.size == 1 {
                    self.data = OctreeData::Voxel;
                } else {
                    self.split();
                    self.insert(voxel);
                }
            }
            OctreeData::Voxel => {}
        }
    }

    fn split(&mut self) {
        let Cube {
            pos: Point { x, y, z },
            size,
        } = self.bounds;
        let n = size / 2;
        self.data = OctreeData::Split(Box::new([
            Self::new(Cube::new(x + 0, y + 0, z + 0, n)),
            Self::new(Cube::new(x + 0, y + n, z + 0, n)),
            Self::new(Cube::new(x + n, y + 0, z + 0, n)),
            Self::new(Cube::new(x + n, y + n, z + 0, n)),
            // Bottom
            Self::new(Cube::new(x + 0, y + 0, z + n, n)),
            Self::new(Cube::new(x + 0, y + n, z + n, n)),
            Self::new(Cube::new(x + n, y + 0, z + n, n)),
            Self::new(Cube::new(x + n, y + n, z + n, n)),
        ]));
    }

    fn find_closest(&self, p: &Vec3, dir: &Vec3, closest: f64) -> (Option<Point>, f64) {
        if !self.bounds.containsf(&p) {
            return (None, f64::MAX);
        }
        let march_dist = self.bounds.max_marchable_distance(p, dir);
        let mut voxel = None;
        let mut dist = march_dist;

        match &self.data {
            OctreeData::Split(subtrees) => {
                for tree in subtrees.iter() {
                    let (v, d) = tree.find_closest(p, dir, dist);
                    if d < dist {
                        voxel = v;
                        dist = d;
                    }
                }
            }
            OctreeData::Voxel => {
                dist = 0.;
                voxel = Some(self.bounds.pos);
            }
            OctreeData::Empty => {
                return (None, march_dist);
            }
        }
        return (voxel, dist);
    }
}

const MAX_SAMPLE_STEPS: usize = 10;
const MAX_DISTANCE: f64 = 100.;
const MIN_DISTANCE: f64 = 0.0001;
const PUSH_ANALAYZE_DISTANCE: f64 = 0.000000001;

fn main() {
    let mut tree = Octree::new(Cube::new(-1, -1, -1, 128));

    tree.insert(Point::new(5, 5, 10));
    tree.insert(Point::new(5, 4, 10));
    tree.insert(Point::new(5, 5, 9));
    for x in 3..8 {
        for z in 5..12 {
            tree.insert(Point::new(x, 7, z));
        }
    }

    let mut buffer = ImageBuffer::new(600, 600);
    for i in 0..600 {
        for j in 0..600 {
            let mut pos = Vec3::new(0., 0., 0.);
            let dir = Vec3::new(i as f64, j as f64, 600.).normalized();
            let mut voxel = None;
            for _ in 0..MAX_SAMPLE_STEPS {
                let (v, d) = tree.find_closest(&pos, &dir, f64::MAX);
                if let Some(v) = v {
                    voxel = Some(v);
                    break;
                }
                pos = pos.add(&dir.mul(d + PUSH_ANALAYZE_DISTANCE));
                let len = pos.len();
                if len > MAX_DISTANCE {
                    break;
                }
                if d < MIN_DISTANCE {
                    break;
                }
            }
            if let Some(_) = voxel {
                let level = ((1. / (pos.len() as f64 / 5.)).powf(2.) * 65536.) as u16;
                buffer.put_pixel(i, j, Rgb([level, level, level]));
            }
        }
    }
    buffer.save(&Path::new("out.png")).unwrap();
}
