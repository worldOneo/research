struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    var arr = array<vec2<f32>, 6>(
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
    );
    var xy = arr[in_vertex_index];
    out.position = xy;
    out.clip_position = vec4<f32>(xy, 1.0, 1.0);
    return out;
}

//
// --- Quaternions
//
fn quaternion_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let tmp = cross(q.xyz, v) + q.w * v;
    let rotated = v + 2.0 * cross(q.xyz, tmp);
    return rotated;
}

fn create_quaternion_rotation(axis: vec3<f32>, angle: f32) -> vec4<f32> {
    let cos = cos(angle * .5);
    let sin = sin(angle * .5);
    return normalize(vec4<f32>(axis.xyz * sin, cos));
}

//
// --- Cube
//

struct Cube {
    position: vec3<f32>,
    size: f32,
}

fn create_cube(position: vec3<f32>, size: f32) -> Cube {
    return Cube(position, size);
}

const F32_MAX = 3.40282347e+38;

fn next_plane_intersection(origin: vec3<f32>, point: vec3<f32>, dir: vec3<f32>) -> f32 {
    let relative_position = origin - point;
    let distance = relative_position / dir;
    return min(
        min(select(F32_MAX, distance.x, distance.x > 0.), select(F32_MAX, distance.y, distance.y > 0.)),
        select(F32_MAX, distance.z, distance.z > 0.)
    );
}

fn cube_max_marchable_distance(bounds: Cube, point: vec3<f32>, dir: vec3<f32>) -> f32 {
    let near = (bounds.position - point) / dir;
    let far = (bounds.position + bounds.size - point) / dir;
    let smallest = min(abs(near), abs(far));
    return min(next_plane_intersection(bounds.position, point, dir), next_plane_intersection(bounds.position + bounds.size, point, dir));
}

//
// --- Ray
//

struct Ray {
    position: vec3<f32>,
    dir: vec3<f32>,
}

fn cube_planes_ray_intersection_dist(cube: Cube, ray: Ray) -> f32 {
    return cube_max_marchable_distance(cube, ray.position, ray.dir);
}

fn distance_to_cube(cube: Cube, point: vec3<f32>) -> f32 {
    var d = abs((cube.position + cube.size / 2.) - point) - cube.size / 2.;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, vec3<f32>(0.0)));
}


//
// --- Ray tracing
//


const eps = 1.e-4;
const cast_max = 1.e20;
const MAX_STEPS = 64;

struct HitResult {
    jumps: i32,
    distance: f32,
    destination: vec3<f32>,
    hit: ChunkQueryResult,
}

fn cast_to_hit(ray: Ray) -> HitResult {
    var ray = ray;
    let start = ray.position;
    var travelled = 0.;
    var hit = chunk_query_ray(ray);
    for (var i = 0; i < MAX_STEPS; i++) {
        travelled += hit.distance;
        if travelled > 1000. {
            return HitResult(i, cast_max, ray.position, hit);
        }
        if hit.hit {
            return HitResult(i, travelled, ray.position, hit);
        }
        ray.position += (hit.distance + 1.e-5) * ray.dir;
        hit = chunk_query_ray(ray);
    }
    return HitResult(MAX_STEPS, cast_max, ray.position, hit);
}

//
// --- Bits
//

fn bits_get_byte_n(data: u32, n: u32) -> u32 {
    return (data >> (n * 8u)) & 0xFFu;
}

fn bits_get_range(data: u32, start: u32, stop: u32) -> u32 {
    return (data >> start) & ((2u << (stop - start)) - 1u);
}

//
// --- Material
//

// Material format:
// RRRRRRRRGGGGGGGGBBBBBBBBTTAAAAAA
// [------][------][------][][----]
// Red     Green   Blue    | Attribute
//                         Type: 0 = Absent, 1 = Rough, 2 = Emissive, 3 = Transparent
// Attribute:
// Rough/Emissive: VVVVVV
//                 [----]
//                 Value
// Transparent: Soon...

struct Material {
    data: u32,
}

fn material_color(material: Material) -> vec3<f32> {
    return vec3<f32>(
        f32(bits_get_byte_n(material.data, 3u)),
        f32(bits_get_byte_n(material.data, 2u)),
        f32(bits_get_byte_n(material.data, 1u))
    ) / 255.;
}

const MATERIAL_TYPE_ABSENT = 0u;
const MATERIAL_TYPE_ROUGH = 1u;
const MATERIAL_TYPE_EMISSIVE = 2u;
const MATERIAL_TYPE_TRANSPARENT = 3u;

fn material_type(material: Material) -> u32 {
    return bits_get_range(material.data, 6u, 7u);
}

fn material_present(material: Material) -> bool {
    return material_type(material) != MATERIAL_TYPE_ABSENT;
}

//
// --- Chunk
//

const CHUNK_DIMENSION = 16;
const CHUNK_SIZE = 4096; // 16*16*16

struct Chunk {
    location: vec3<i32>,
    data: array<u32, CHUNK_SIZE>,
}

fn chunk_material_f(position: vec3<f32>) -> Material {
    let floored = vec3<i32>(position);
    return chunk_material(floored);
}

fn chunk_material(position: vec3<i32>) -> Material {
    let relative_position = position - chunk.location;
    let x = position.x;
    let y = position.y;
    let z = position.z;
    let max = x | y | z;
    if x < 0 || y < 0 || z < 0 || max > 15 {
        return Material(0u);
    }
    let idx = x + y * CHUNK_DIMENSION + z * CHUNK_DIMENSION * CHUNK_DIMENSION;
    let d = chunk.data[idx];
    return Material(d);
}

struct ChunkQueryResult {
    hit: bool,
    material: Material,
    distance: f32,
}

fn chunk_query_ray(ray: Ray) -> ChunkQueryResult {
    let material = chunk_material_f(ray.position);
    if material_present(material) {
        return ChunkQueryResult(true, material, 0.);
    }
    let travel_distance = cube_planes_ray_intersection_dist(Cube(floor(ray.position), 1.), ray);
    return ChunkQueryResult(false, material, travel_distance);
}

//
// --- Shared map
//

@group(1)@binding(0)
var<storage, read_write> chunk: Chunk;

// Fragment shader


struct RenderData {
    screen: vec2<f32>,
    camera: vec3<f32>,
    rotations: vec2<f32>,
};

@group(0)@binding(0)
var<uniform> render_data: RenderData;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cube = create_cube(vec3(3., 0., 0.), 1.0);

    let streched_xy_rot = (in.position.xy * render_data.screen) / render_data.screen.x;

    let dir = normalize(vec3<f32>(2.0, streched_xy_rot.x, streched_xy_rot.y));
    let rot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), render_data.rotations.y);
    let rot_dir = quaternion_rotate(rot, dir);
    let rot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), render_data.rotations.x);
    let rot_dir2 = quaternion_rotate(rot2, rot_dir);
    let ray = Ray(render_data.camera, rot_dir2);
    let hit = cast_to_hit(ray);
    
    // var col = vec3<f32>(pow(hit.distance / 10., 4.));
    let col = select(vec3<f32>(0.), material_color(hit.hit.material), hit.hit.hit);

    return vec4<f32>(col, 1.0);
}