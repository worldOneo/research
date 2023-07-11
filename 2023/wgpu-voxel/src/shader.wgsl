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
const YANK = 1.e-5;

struct Marchable {
    distance: f32,
    yank: vec3<f32>,
}

fn cube_max_marchable_distance(bounds: Cube, point: vec3<f32>, dir: vec3<f32>) -> Marchable {
    let near = bounds.position;
    let far = bounds.position + bounds.size;
    let voxel_wall = select(near, far, vec3<bool>(dir.x > 0., dir.y > 0., dir.z > 0.));
    let distance = abs((voxel_wall - point) / dir);
    if distance.x < distance.y && distance.x < distance.z {
        return Marchable(distance.x, vec3<f32>(select(-YANK, YANK, dir.x > 0.), 0., 0.));
    }
    if distance.y < distance.z {
        return Marchable(distance.y, vec3<f32>(0., select(-YANK, YANK, dir.y > 0.), 0.));
    }
    return Marchable(distance.z, vec3<f32>(0., 0., select(-YANK, YANK, dir.z > 0.)));
}

fn cube_normal_of_ray(bounds: Cube, ray: Ray) -> vec3<f32> {
    let near = bounds.position;
    let far = bounds.position + bounds.size;
    let dir = -ray.dir;
    let point = ray.position;
    let voxel_wall = select(near, far, vec3<bool>(dir.x > 0., dir.y > 0., dir.z > 0.));
    let distance = abs((voxel_wall - point) / dir);
    if distance.x < distance.y && distance.x < distance.z {
        let normal = vec3<f32>(select(-1., 1., dir.x > 0.), 0., 0.);
        return normal;
    }
    if distance.y < distance.z {
        let normal = vec3<f32>(0., select(-1., 1., dir.y > 0.), 0.);
        return normal;
    }
    let normal = vec3<f32>(0., 0., select(-1., 1., dir.z > 0.));
    return normal;
}

//
// --- Ray
//

struct Ray {
    position: vec3<f32>,
    dir: vec3<f32>,
}

fn cube_planes_ray_intersection_dist(cube: Cube, ray: Ray) -> Marchable {
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
const CAST_MAX = 1.e20;
const MAX_STEPS = 64;

struct HitResult {
    jumps: i32,
    distance: f32,
    destination: vec3<f32>,
    query: ChunkQueryResult,
    color: vec3<f32>,
}

fn cast_to_hit(ray: Ray) -> HitResult {
    var ray = ray;
    let start = ray.position;
    var travelled = 0.;
    var query = chunk_query_ray(ray);
    for (var i = 0; i < MAX_STEPS; i++) {
        travelled += query.distance.distance;
        if travelled > 100. {
            return HitResult(i, CAST_MAX, ray.position, query, material_color(query.material));
        }
        if query.present {
            return HitResult(i, travelled, ray.position, query, material_color(query.material));
        }
        ray.position += query.distance.distance * ray.dir + query.distance.yank;
        query = chunk_query_ray(ray);
    }
    return HitResult(MAX_STEPS, CAST_MAX, ray.position, query, material_color(query.material));
}


struct TraceResult {
    moment: MomentInfo,
    destination: vec3<f32>,
    present: bool,
}

const SECONDARY_RAYS = 1;

fn ray_trace(ray: Ray) -> TraceResult {
    var color = vec3<f32>(0.);
    var hit = cast_to_hit(ray);
    var moment: MomentInfo ;
    moment.albedo = hit.color;
    moment.depth = hit.distance;
    moment.normal = hit.query.normal;
    moment.irradiance = 0.;
    if hit.distance == CAST_MAX {
        return TraceResult(moment, hit.destination, false);
    }
    if material_type(hit.query.material) == MATERIAL_TYPE_EMISSIVE {
        let light_strength = material_attrib(hit.query.material);
        moment.emittance = 1.;
        moment.irradiance = 0.;
        return TraceResult(moment, hit.destination, true);
    }
    var irradiance = 0.;
    var irradiance_square = 0.;

    for (var i = 0; i < SECONDARY_RAYS; i++) {
        let ray = Ray(hit.destination + hit.query.normal * 0.01, unit_vec_on_hemisphere(hit.query.normal));
        let bounce = cast_to_hit(ray);
        if material_type(bounce.query.material) != MATERIAL_TYPE_EMISSIVE {
            continue;
        }
        irradiance += 2.;// material_attrib(hit.query.material);
    }
    moment.irradiance += irradiance;
    irradiance_square += irradiance * irradiance;

    moment.irradiance /= f32(SECONDARY_RAYS);
    irradiance_square /= f32(SECONDARY_RAYS);

    moment.variance = abs(irradiance_square - moment.irradiance * moment.irradiance);
    return TraceResult(moment, hit.destination, true);
}

//
// --- Bits
//

fn bits_get_byte_n(data: u32, n: u32) -> u32 {
    return (data >> (n * 8u)) & 0xFFu;
}

fn bits_get_range(data: u32, start: u32, stop: u32) -> u32 {
    return ((data >> start) & ((2u << (stop - start)) - 1u));
}

fn bits_f32_as_u8(v: f32) -> u32 {
    return u32(clamp(v, 0.0, 1.0) * 255.0) & 0xFFu;
}

fn bits_u8_as_f32(v: u32) -> f32 {
    return f32(v & 0xFFu) * (1.0 / 255.0);
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
const MATERIAL_TYPE_OPACITY = 3u;

fn material_type(material: Material) -> u32 {
    return bits_get_range(material.data, 6u, 7u);
}

fn material_attrib(material: Material) -> f32 {
    let raw_attrib = bits_get_range(material.data, 0u, 5u);
    return pow(2., f32(raw_attrib) / 4.) - 1.;
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
    present: bool,
    material: Material,
    distance: Marchable,
    normal: vec3<f32>,
}

fn chunk_query_ray(ray: Ray) -> ChunkQueryResult {
    let material = chunk_material_f(ray.position);
    if material_present(material) {
        return ChunkQueryResult(true, material, Marchable(0., vec3<f32>(0.)), cube_normal_of_ray(Cube(floor(ray.position), 1.), ray));
    }
    let travel_distance = cube_planes_ray_intersection_dist(Cube(floor(ray.position), 1.), ray);
    return ChunkQueryResult(false, material, travel_distance, vec3<f32>(0.));
}

//
// --- Random
//

var<private> seed: u32 = 0u;

fn wang_hash_init(fCoords: vec2<u32>, frame: u32) {
    seed = (fCoords.x * 1973u + fCoords.y * 9277u + frame * 26699u) | 1u;
}

fn wang_hash() -> u32 {
    seed = seed ^ 61u;
    seed = seed ^ (seed >> 16u);
    seed *= u32(9);
    seed = seed ^ (seed >> 4u);
    seed *= u32(0x27d4eb2d);
    seed = seed ^ (seed >> 15u);
    return seed;
}

fn rand_float() -> f32 {
    return f32(wang_hash()) / 4294967295.;
}

const TWO_PI = 6.28318530718;

// https://graphics.pixar.com/library/OrthonormalB/paper.pdf
fn orthonormal_basis(n: vec3<f32>, b1: ptr<function, vec3<f32>>, b2: ptr<function, vec3<f32>>) {
    let s = select(-1., 1., n.z >= 0.0);
    let a = -1.0 / (s + n.z);
    let b = n.x * n.y * a;
    *b1 = vec3(1.0 + s * n.x * n.x * a, s * b, -s * n.x);
    *b2 = vec3(b, s + n.y * n.y * a, -n.y);
}

fn unit_vec_on_hemisphere(n: vec3<f32>) -> vec3<f32> {
    let r = rand_float();
    let angle = rand_float() * TWO_PI;
    let sr = sqrt(r);
    let p = vec2<f32>(sr * cos(angle), sr * sin(angle));
    let ph = vec3(p.xy, sqrt(1.0 - dot(p, p)));

    var b1 = vec3<f32>(0.);
    var b2 = vec3<f32>(0.);
    orthonormal_basis(n, &b1, &b2);
    return b1 * ph.x + b2 * ph.y + n * ph.z;
}

//
// --- Shared map
//

@group(1)@binding(0)
var<storage, read_write> chunk: Chunk;

//
// --- SVGF or smth idk
//

struct MomentInfo {
    depth: f32,
    emittance: f32,
    variance: f32,
    irradiance: f32,
    error_free_frames: u32,
    albedo: vec3<f32>,
    normal: vec3<f32>,
}

struct FrameInfo {
    camera: vec3<f32>,
    screen: vec2<f32>,
    arr: array<MomentInfo>,
}

fn world_space_to_screen_space(camera_position: vec3<f32>, rotation: vec2<f32>, position_ws: vec3<f32>) -> vec2<f32> {
    // undo tracing
    let norm = position_ws - camera_position;
    
    // undo up/down rotation
    let unrot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), -rotation.x);
    let rot_dir2 = quaternion_rotate(unrot2, norm);
    
    // undo left/right rotation
    let unrot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), -rotation.y);
    let rot_dir = quaternion_rotate(unrot, rot_dir2);
    
    // undo normalization
    // n = v / l => l = v / n
    let len = VERY_ODD_FOV_NUMBER / rot_dir.x;
    let stretched = rot_dir.yz * len;
    
    // undo aspect ratio to convert back to UV
    let aspect = render_data.screen / render_data.screen.x;
    let in_pos = stretched / aspect;

    // calculate screen space coordinates
    return in_pos;
}

fn packMoment(gbuf: MomentInfo) -> vec4<u32> {
    var p = vec4<u32>(0u);
    p.x = bits_f32_as_u8(gbuf.albedo.r) | (bits_f32_as_u8(gbuf.albedo.g) << 8u) | (bits_f32_as_u8(gbuf.albedo.b) << 16u) | (bits_f32_as_u8(gbuf.emittance) << 24u);
    let normal = (gbuf.normal + 1.0) * 0.5;
    p.y = bits_f32_as_u8(normal.x) | (bits_f32_as_u8(normal.y) << 8u) | (bits_f32_as_u8(normal.z) << 16u) | ((gbuf.error_free_frames & 0xFFu) << 24u);
    p.z = pack2x16float(vec2(gbuf.depth, gbuf.variance));
    p.w = bitcast<u32>(gbuf.irradiance);
    return p;
}

fn unpackMoment(p: vec4<u32>) -> MomentInfo {
    var moment: MomentInfo;
    moment.albedo.r = bits_u8_as_f32(p.x);
    moment.albedo.g = bits_u8_as_f32(p.x >> 8u);
    moment.albedo.b = bits_u8_as_f32(p.x >> 16u);
    moment.emittance = bits_u8_as_f32(p.x >> 24u);
    moment.normal.x = bits_u8_as_f32(p.y);
    moment.normal.y = bits_u8_as_f32(p.y >> 8u);
    moment.normal.z = bits_u8_as_f32(p.y >> 16u);
    moment.normal = normalize(moment.normal * 2.0 - 1.0);
    moment.error_free_frames = p.y >> 24u;
    let tmp = unpack2x16float(p.z);
    moment.depth = tmp.x;
    moment.variance = tmp.y;
    moment.irradiance = bitcast<f32>(p.w);
    return moment;
}


// https://www.semanticscholar.org/paper/Progressive-Spatiotemporal-Variance-Guided-Dundr/a81a4eed7f303f7e7f3ca1914ccab66351ce662b?p2df
// 4.4
fn normal_weight(n0: vec3<f32>, n1: vec3<f32>) -> f32 {
    return pow(max(0.0, dot(n0, n1)), 64.);
}

fn depth_weight(d0: f32, d1: f32, grad: vec2<f32>, off: vec2<f32>) -> f32 {
    return exp((-abs(d0 - d1)) / (abs(dot(grad, off)) + eps));
}

fn luminance_weight(l0: f32, l1: f32, variance: f32) -> f32 {
    return exp((-abs(l0 - l1)) / (4. * variance + eps));
}

fn denoise(moment: MomentInfo, coords: vec2<i32>, step: f32) -> MomentInfo {
    var moment = moment;
    var denoise_kernel: array<f32, 9> = array<f32,9>(0.0625, 0.125, 0.0625, 0.125, 0.25, 0.125, 0.0625, 0.125, 0.0625);
    let buffer_switch_read_offset = select(0, 1, (render_data.frame & 1u) == 0u);
    let grad = vec2<f32>(dpdxFine(moment.depth), dpdyFine(moment.depth));
    var irradiance = 0.;
    var wsum = 0.;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let otherOffset = vec2<f32>(f32(x), f32(y)) * step;
            var otherMoment = unpackMoment(textureLoad(moments, coords + vec2<i32>(otherOffset), buffer_switch_read_offset));
            if all(otherOffset == vec2<f32>(0., 0.)) {
                otherMoment = moment;
            }
            let n_weight = normal_weight(moment.normal, otherMoment.normal);
            let d_weight = depth_weight(moment.depth, otherMoment.depth, grad, otherOffset);
            let l_weight = luminance_weight(moment.irradiance, otherMoment.irradiance, moment.variance);
            let weight = clamp(d_weight * l_weight * n_weight, 0., 1.);
            let filter_weight = weight * denoise_kernel[x + 1 + (y + 1) * 3];
            irradiance += otherMoment.irradiance * filter_weight;
            wsum += filter_weight;
        }
    }
    moment.irradiance = irradiance / wsum;
    return moment;
} 

const HISTORY_FACTOR: f32 = 0.1;

fn temporal_accumulate(hit: MomentInfo, destination: vec3<f32>) -> MomentInfo {
    let reconstructed_uv = world_space_to_screen_space(render_data.old_camera.position, render_data.old_camera.rotation, destination);
    let reconstructed_coords = vec2<i32>(fragment_to_screen_coords(reconstructed_uv));

    let buffer_switch_read_offset = select(0, 1, (render_data.frame & 1u) == 0u);

    let old_moment = textureLoad(moments, reconstructed_coords, buffer_switch_read_offset);
    let unpacked_moment = unpackMoment(old_moment);
    var new_moment: MomentInfo;
    let mix = max(HISTORY_FACTOR, 1. / f32(unpacked_moment.error_free_frames));
    new_moment.albedo = mix(unpacked_moment.albedo, hit.albedo, HISTORY_FACTOR);
    new_moment.irradiance = mix(unpacked_moment.irradiance, hit.irradiance, HISTORY_FACTOR);
    new_moment.variance = mix(unpacked_moment.variance, hit.variance, HISTORY_FACTOR);
    new_moment.emittance = hit.emittance;
    new_moment.depth = hit.depth;
    new_moment.normal = hit.normal;

    let distance_to_large = abs(hit.depth - unpacked_moment.depth) > 0.02;
    let coords_invalid = reconstructed_uv.x < -1. || reconstructed_uv.y < -1. || reconstructed_uv.x > 1. || reconstructed_uv.y > 1.;

    if render_data.frame == 0u || distance_to_large || coords_invalid {
        new_moment.albedo = hit.albedo;
        new_moment.irradiance = hit.irradiance;
        new_moment.variance = hit.variance;
        new_moment.error_free_frames = 0u;
    } else {
        new_moment.error_free_frames = min(unpacked_moment.error_free_frames + 1u, 255u);
    }
    return new_moment;
}

@group(2)@binding(0)
var moments: texture_storage_2d_array<rgba32uint, read_write>;

// Fragment shader

struct Camera {
    position: vec3<f32>,
    rotation: vec2<f32>,
}

struct RenderData {
    screen: vec2<f32>,
    camera: Camera,
    old_camera: Camera,
    frame: u32,
};

@group(0)@binding(0)
var<uniform> render_data: RenderData;

fn fragment_to_screen_coords(uv: vec2<f32>) -> vec2<u32> {
    return  vec2<u32>(ceil((uv * 0.5 + 0.5) * (render_data.screen)));
}

const VERY_ODD_FOV_NUMBER: f32 = 2.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coords = fragment_to_screen_coords(in.position.xy);
    wang_hash_init(coords, render_data.frame);

    let aspect = render_data.screen / render_data.screen.x;
    let streched_xy_rot = in.position.xy * aspect;

    let dir = normalize(vec3<f32>(VERY_ODD_FOV_NUMBER, streched_xy_rot.x, streched_xy_rot.y));
    let rot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), render_data.camera.rotation.y);
    let rot_dir = quaternion_rotate(rot, dir);
    let rot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), render_data.camera.rotation.x);
    let rot_dir2 = quaternion_rotate(rot2, rot_dir);
    let ray = Ray(render_data.camera.position, rot_dir2);
    let hit = ray_trace(ray);

    // temporal accumulation

    let moment = temporal_accumulate(hit.moment, hit.destination);
    // let denoised1 = denoise(moment, vec2<i32>(coords), 1.);
    // let denoised2 = denoise(denoised1, vec2<i32>(coords), 2.);
    // let denoised3 = denoise(denoised2, vec2<i32>(coords), 4.);
    // let denoised = denoise(denoised3, vec2<i32>(coords), 8.);
    // denoise(vec2<i32>(coords), 4.);
    // let denoised = denoise(vec2<i32>(coords), 8.);
    let buffer_switch_write_offset = select(1, 0, (render_data.frame & 1u) == 0u);
    textureStore(moments, vec2<i32>(coords), buffer_switch_write_offset, packMoment(moment));

    // var col = vec3<f32>(f32(hit.jumps) / 64.);
    // let col = select(vec3<f32>((rot_dir2.xy + 1.) / 2., 0.), (hit.query.normal + 1.) / 2., hit.query.present);
    // let col = rand_unit_vec();
    // let col = vec3<f32>(hit.distance / 20.);
    // let col = select(abs(vec3<f32>(vec2<f32>(vec2<u32>(reconstructed_coords.xy) - coords.xy) / render_data.screen.xy, 1.)), vec3<f32>(0.), all(vec2<u32>(reconstructed_coords) == coords));
    // let col = select(vec3<f32>((rot_dir2.xy + 1.) / 2., 0.), denoised.albedo * (denoised.irradiance + denoised.emittance), hit.present);
    // let col = select(vec3<f32>((rot_dir2.xy + 1.) / 2., 0.), hit.color, hit.query.present);
    let col = select(vec3<f32>((rot_dir2.xy + 1.) / 2., 0.), vec3<f32>(moment.irradiance), hit.present);


    return vec4<f32>(col, 1.0);
}