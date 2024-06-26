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

struct TraceResult {
    moment: MomentInfo,
    destination: vec3<f32>,
    present: bool,
}

const SECONDARY_RAYS = 2;

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
    return pow(2., f32(raw_attrib) / 16.) - 1.;
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

const TWO_PI = 6.283185307179586;

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

fn camera_to_ray(camera: Camera, in_pos: vec2<f32>) -> Ray {
    let aspect = render_data.screen / render_data.screen.x;
    let streched_xy_rot = in_pos * aspect;

    let dir = normalize(vec3<f32>(VERY_ODD_FOV_NUMBER, streched_xy_rot.x, streched_xy_rot.y));
    let rot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), camera.rotation.y);
    let rot_dir = quaternion_rotate(rot, dir);
    let rot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), camera.rotation.x);
    let rot_dir2 = quaternion_rotate(rot2, rot_dir);
    let ray = Ray(camera.position, rot_dir2);
    return ray;
}

fn world_space_to_screen_space(camera: Camera, position_ws: vec3<f32>) -> vec2<f32> {
    // undo tracing
    let rot_dir2 = normalize(position_ws - camera.position);
    
    // undo up/down rotation
    let unrot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., -1.0), camera.rotation.x);
    let rot_dir = quaternion_rotate(unrot2, rot_dir2);
    
    // undo left/right rotation
    let unrot = create_quaternion_rotation(vec3<f32>(0.0, -1.0, 0.0), camera.rotation.y);
    let dir = quaternion_rotate(unrot, rot_dir);
    
    // undo normalization
    // n = v / l => l = v / n
    let len = VERY_ODD_FOV_NUMBER / dir.x;
    let stretched = dir.yz * len;
    
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
    p.z = pack2x16float(vec2(gbuf.irradiance, gbuf.variance));
    p.w = bitcast<u32>(gbuf.depth);
    return p;
}

fn unpack_moment(p: vec4<u32>) -> MomentInfo {
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
    moment.irradiance = tmp.x;
    moment.variance = tmp.y;
    moment.depth = bitcast<f32>(p.w);
    return moment;
}

fn load_moment(coords: vec2<i32>, index: i32) -> MomentInfo {
    return unpack_moment(textureLoad(moments, coords, index));
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

fn denoise(index: i32, coords: vec2<i32>, step: f32) -> MomentInfo {
    //        16 8 16
    // 1 over 8  4 8
    //        16 8 16
    var denoise_kernel: array<f32, 9> = array<f32,9>(0.0625, 0.125, 0.0625, 0.125, 0.25, 0.125, 0.0625, 0.125, 0.0625);
    var moment = load_moment(coords, index);
    let momentx = load_moment(coords + vec2<i32>(1, 0), index);
    let momenty = load_moment(coords + vec2<i32>(0, 1), index);

    let grad = vec2<f32>(momentx.depth - moment.depth, momenty.depth - moment.depth);
    var irradiance = 0.;
    var wsum = 0.;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let otherOffset = vec2<f32>(f32(x), f32(y)) * step;
            let otherMoment = load_moment(coords + vec2<i32>(otherOffset), index);
            let n_weight = normal_weight(moment.normal, otherMoment.normal);
            let d_weight = depth_weight(moment.depth, otherMoment.depth, grad, otherOffset);
            let l_weight = luminance_weight(moment.irradiance, otherMoment.irradiance, moment.variance);
            let weight = clamp(d_weight * n_weight, 0., 1.);
            let filter_weight = weight * denoise_kernel[x + 1 + (y + 1) * 3];
            irradiance += otherMoment.irradiance * filter_weight;
            wsum += filter_weight;
        }
    }
    moment.irradiance = irradiance / wsum;
    return moment;
} 

const HISTORY_FACTOR: f32 = 0.05;

// fn select_closest_moment(uv: ptr<function, vec2<f32>>, to: MomentInfo, index: i32, destination: vec3<f32>) -> MomentInfo {
//     var min_distance = CAST_MAX;
//     var moment: MomentInfo;
//     var min_uv: vec2<f32> = *uv;
//     let uv_coords = fragment_to_screen_coords(*uv);
//     for (var x = -1; x <= 1; x++) {
//         for (var y = -1; y <= 1; y++) {
//             let xy_uv = *uv + vec2<f32>(f32(x), f32(y)) / (render_data.screen * 0.5);
//             let xy_coords = uv_coords + vec2<i32>(x, y);
//             if xy_coords.x < 0 || xy_coords.y < 0 || xy_coords.x > i32(render_data.screen.x) || xy_coords.y > i32(render_data.screen.y) {
//                 continue;
//             }
//             let unpacked = load_moment(xy_coords, index);
//             let norm_dist = distance(unpacked.normal, to.normal);
//             let alb_dist = distance(unpacked.albedo, to.albedo);
//             if norm_dist > 0.1 || alb_dist > 0.1 {
//                 continue;
//             }

//             let ray = camera_to_ray(render_data.old_camera, xy_uv);
//             let dist = distance(destination, ray.position + ray.dir * unpacked.depth);
//             if dist < min_distance {
//                 moment = unpacked;
//                 min_distance = dist;
//                 min_uv = xy_uv;
//             }
//         }
//     }
//     *uv = min_uv;
//     return moment;
// }

fn temporal_accumulate(hit: MomentInfo, indecies: StorageIndecies, in_pos: vec2<f32>, destination: vec3<f32>) -> MomentInfo {
    var org_hit = hit;
    let reconstructed_uv = world_space_to_screen_space(render_data.old_camera, destination);
    var closest_uv = reconstructed_uv;
    let reprojected_coords = prev_screen_coord;
    // let prev_moment = load_moment(reprojected_coords, indecies.history); // select_closest_moment(&closest_uv, hit, indecies.history, destination); // load_moment(fragment_to_screen_coords(closest_uv), indecies.history);// select_closest_moment(&closest_uv, indecies.history, destination);
    var denoised_hit = denoise(indecies.history, vec2<i32>(reprojected_coords), 1.);

    let distance_to_large = abs(hit.depth - prev_moment.depth) > 0.02;
    let coords_invalid = reconstructed_uv.x < -1. || reconstructed_uv.y < -1. || reconstructed_uv.x > 1. || reconstructed_uv.y > 1.;
    let equal_normals = distance(hit.normal, prev_moment.normal) < eps;

    if render_data.frame != 0u && !distance_to_large && !coords_invalid && equal_normals {
        let mix = max(HISTORY_FACTOR, 1. / f32(prev_moment.error_free_frames));
        org_hit.irradiance = mix(denoised_hit.irradiance, hit.irradiance, mix);
        org_hit.variance = mix(denoised_hit.variance, hit.variance, mix);
        org_hit.error_free_frames = min(prev_moment.error_free_frames + 1u, 255u);
    } else {
        org_hit.error_free_frames = 1u;
    }

    return org_hit;
}

@group(2)@binding(0)
var moments: texture_storage_2d_array<rgba32uint, read_write>;

//
// --- Stable resampling
//

struct Sample {
    dir: vec3<f32>,
    present: bool,
    weight: f32,
    attempts: u32,
}

struct Samples {
    samples: array<Sample, 3>,
}

fn pack_sample(sample: Sample) -> u32 {
    let nfactor = 127.;
    let x = clamp(u32(sample.dir.x * nfactor + 128.), 0u, 255u) << 24u;
    let y = clamp(u32(sample.dir.y * nfactor + 128.), 0u, 255u) << 16u;
    let z = clamp(u32(sample.dir.z * nfactor + 128.), 0u, 255u) << 8u;
    let w = select(0u, 1u << 7u, sample.present) | clamp(u32(sample.weight), 0u, 127u);
    return x | y | z | w;
}

fn unpack_sample(sample: u32, attempts: u32) -> Sample {
    let nfactor = 127.;
    let x = (f32((sample >> 24u) & 0xFFu) - 128.) / nfactor;
    let y = (f32((sample >> 16u) & 0xFFu) - 128.) / nfactor;
    let z = (f32((sample >> 8u) & 0xFFu) - 128.) / nfactor;
    let w = max(f32(sample & 0x7Fu), 1.);
    return Sample(vec3<f32>(x, y, z), (sample & 0x80u) == 0x80u, w, attempts);
}

fn pack_samples(samples: Samples) -> vec4<u32> {
    return vec4<u32>(
        pack_sample(samples.samples[0]),
        pack_sample(samples.samples[1]),
        pack_sample(samples.samples[2]),
        (min(samples.samples[0].attempts, 255u) << 16u) | (min(samples.samples[1].attempts, 255u) << 8u) | min(samples.samples[2].attempts, 255u)
    );
}

fn unpack_samples(samples: vec4<u32>) -> Samples {
    return Samples(array<Sample, 3>(
        unpack_sample(samples.x, (samples.w >> 16u) & 0xFFu),
        unpack_sample(samples.y, (samples.w >> 8u) & 0xFFu),
        unpack_sample(samples.z, samples.w & 0xFFu)
    ));
}

fn load_samples(coords: vec2<i32>) -> Samples {
    let image = select(0, 1, (render_data.frame & 1u) == 0u);
    return unpack_samples(textureLoad(samples, coords, image));
}

fn store_samples(coords: vec2<i32>, s: Samples) {
    let image = select(1, 0, (render_data.frame & 1u) == 0u);
    textureStore(samples, coords, image, pack_samples(s));
}

var<private> prev_moment: MomentInfo;
var<private> prev_screen_coord: vec2<i32>;

fn select_closest_moment(uv: vec2<f32>, to: MomentInfo, index: i32, destination: vec3<f32>) {
    var min_distance = CAST_MAX;
    var moment: MomentInfo;
    var min_uv: vec2<f32> = uv;
    let uv_coords = fragment_to_screen_coords(uv);
    var min_xy: vec2<i32> = uv_coords;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let xy_uv = uv + vec2<f32>(f32(x), f32(y)) / (render_data.screen * 0.5);
            let xy_coords = uv_coords + vec2<i32>(x, y);
            if xy_coords.x < 0 || xy_coords.y < 0 || xy_coords.x > i32(render_data.screen.x) || xy_coords.y > i32(render_data.screen.y) {
                continue;
            }
            let unpacked = load_moment(xy_coords, index);
            let norm_dist = distance(unpacked.normal, to.normal);
            let alb_dist = distance(unpacked.albedo, to.albedo);
            if norm_dist > 0.1 || alb_dist > 0.1 {
                continue;
            }

            let ray = camera_to_ray(render_data.old_camera, xy_uv);
            let dist = distance(destination, ray.position + ray.dir * unpacked.depth);
            if dist < min_distance {
                prev_moment = unpacked;
                prev_screen_coord = xy_coords;
                min_distance = dist;
                min_uv = xy_uv;
            }
        }
    }
}

fn samples_first_bounce_samples(hit: MomentInfo, destination: vec3<f32>) -> Samples {
    let indecies = compute_storage_indecies(render_data.frame, 0u);
    let reconstructed_uv = world_space_to_screen_space(render_data.old_camera, destination);
    select_closest_moment(reconstructed_uv, hit, indecies.history, destination);
    let moment = load_moment(prev_screen_coord, indecies.history); // select_closest_moment(&closest_uv, hit, indecies.history, destination);
    let samples = load_samples(prev_screen_coord);
    let ray = camera_to_ray(render_data.old_camera, reconstructed_uv);
    let pos = ray.position + ray.dir * moment.depth;
    if distance(pos, destination) > 0.02 {
        return unpack_samples(vec4<u32>(0u));
    }
    return samples;
}

struct SampleRecommendation {
    dir: vec3<f32>,
    _random: bool,
}

fn sample_recommended_dir(sample: Sample, normal: vec3<f32>) -> SampleRecommendation {
    if !sample.present {
        return SampleRecommendation(unit_vec_on_hemisphere(normal), false);
    }

    if rand_float() > pow(1. - 1. / sample.weight, 0.5) - 0.01 {
        return SampleRecommendation(unit_vec_on_hemisphere(normal), true);
    }
    return SampleRecommendation(sample.dir, false);
}

fn sample_update(sample: Sample, recommendation: SampleRecommendation, hit: bool) -> Sample {
    var n_sample = sample;

    if hit {
        n_sample.present = true;
        n_sample.dir = recommendation.dir;
        if recommendation._random {
            n_sample.weight = f32(max(n_sample.attempts, 1u));
            n_sample.attempts = 1u;
        }
        return n_sample;
    }

    if recommendation._random {
        n_sample.attempts += 1u;
    } else {
        n_sample.weight += 1.;
        n_sample.present = false;
    }
    return n_sample;
}

fn cast_to_hit(cray: Ray) -> HitResult {
    var ray = cray;
    let start = ray.position;
    var travelled = 0.;
    var query: ChunkQueryResult;
    for (var i = 0; i < MAX_STEPS; i++) {
        query = chunk_query_ray(ray);
        if query.present {
            return HitResult(i, travelled, start + ray.dir * travelled, query, material_color(query.material));
        }
        ray.position += query.distance.distance * ray.dir + query.distance.yank;
        travelled += query.distance.distance;
        if travelled > 100. {
            return HitResult(i, CAST_MAX, ray.position, query, material_color(query.material));
        }
    }
    return HitResult(MAX_STEPS, CAST_MAX, ray.position, query, material_color(query.material));
}

fn samples_finish(coords: vec2<i32>, samples: Samples) {
    store_samples(coords, samples);
}

fn sample_ray_trace(coords: vec2<i32>, ray: Ray) -> TraceResult {
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
        moment.emittance = light_strength;
        moment.irradiance = 0.;
        return TraceResult(moment, hit.destination, true);
    }
    var irradiance = 0.;
    var irradiance_square = 0.;

    var samples = samples_first_bounce_samples(moment, hit.destination);
    let location = hit.destination + hit.query.normal * 0.0003;


    // Copy-Pasta until naga fixes array access
        {
        let idx = 0;
        let sample = samples.samples[idx];
        let ray = sample_recommended_dir(sample, hit.query.normal);
        let bounce = cast_to_hit(Ray(location, ray.dir));
        if material_type(bounce.query.material) == MATERIAL_TYPE_EMISSIVE {
            irradiance += material_attrib(bounce.query.material) / (sample.weight);
            samples.samples[idx] = sample_update(sample, ray, true);
        } else {
            samples.samples[idx] = sample_update(sample, ray, false);
        }
    }

        {
        let idx = 1;
        let sample = samples.samples[idx];
        let ray = sample_recommended_dir(sample, hit.query.normal);
        let bounce = cast_to_hit(Ray(location, ray.dir));
        if material_type(bounce.query.material) == MATERIAL_TYPE_EMISSIVE {
            irradiance += material_attrib(bounce.query.material) / (sample.weight);
            samples.samples[idx] = sample_update(sample, ray, true);
        } else {
            samples.samples[idx] = sample_update(sample, ray, false);
        }
    }

    samples_finish(coords, samples);

    moment.irradiance += irradiance;
    irradiance_square += irradiance * irradiance;

    moment.irradiance /= f32(SECONDARY_RAYS);
    irradiance_square /= f32(SECONDARY_RAYS);

    moment.variance = abs(irradiance_square - moment.irradiance * moment.irradiance);
    return TraceResult(moment, hit.destination, true);
}


@group(2)@binding(1)
var samples: texture_storage_2d_array<rgba32uint, read_write>;


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

fn fragment_to_screen_coords(uv: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(round((uv * 0.5 + 0.5) * render_data.screen));
}

const VERY_ODD_FOV_NUMBER: f32 = 2.0;

fn filmic_tone_mapping(ccolor: vec3<f32>) -> vec3<f32> {
    let color = max(vec3<f32>(0.), ccolor - 0.004);
    return (color * (6.2 * color + 0.5)) / (color * (6.2 * color + 1.7) + 0.06);
}

fn aces_tone_mapping(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return (x * (a * x + b)) / saturate((x * (c * x + d) + e));
}

fn white_preserving_luma_based_reinhard_tone_mapping(ccolor: vec3<f32>) -> vec3<f32> {
    let white = 2.;
    let luma = dot(vec3<f32>(0.2126, 0.7152, 0.0722), ccolor);
    let tone_mapped_luma = luma * (1. + luma / (white * white)) / (1. + luma);
    let color = ccolor * (tone_mapped_luma / luma);
    let inv_gamma = 1. / 2.2;
    return pow(color, vec3<f32>(inv_gamma));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coords = fragment_to_screen_coords(in.position);
    let indecies = compute_storage_indecies(render_data.frame, 3u);
    // let moment = denoise(indecies.current, coords, 8.);
    let moment = load_moment(coords, indecies.current);
    // let col = vec3<f32>(moment.depth / 20.);
    // let col = vec3<f32>(moment.irradiance);

    let col = moment.albedo * (moment.irradiance + moment.emittance);

    return vec4<f32>(select(vec3<f32>(in.position, 0.), col, moment.depth < 100.), 1.0);
}


fn compute_fragment_coords(id: vec3<u32>) -> vec2<f32> {
    return ((vec2<f32>(vec2<i32>(id.xy) - vec2<i32>(render_data.screen * 0.5)) / (render_data.screen * 0.5)));
}

struct StorageIndecies {
    history: i32,
    current: i32,
    next: i32,
}

fn compute_storage_indecies(frame: u32, stage: u32) -> StorageIndecies {
    // 0 - Trace      => H | nH | T
    // 0 - Denoise 1. => H | nH | T
    // 1 - Denoise 2. => N | C/H| T
    // 2 - Denoise 4. => C | H  | N
    // 3 - Denoise 8. => N | H  | C

    let history = frame % 3u;
    if stage == 0u {
        // History | new history | Traced
        // Trace: Direct hits into Traced
        // Denoise 1.: Denoise traced + history into new history
        return StorageIndecies(
            i32((frame + 0u) % 3u),
            i32((frame + 2u) % 3u),
            i32((frame + 1u) % 3u),
        );
    }

    if stage == 1u {
        // next | current&history | no value
        // Denoise 2.: denoise current value into next
        return StorageIndecies(
            i32((frame + 1u) % 3u),
            i32((frame + 1u) % 3u),
            i32((frame + 0u) % 3u),
        );
    }

    if stage == 2u {
        // current | history | next
        // Denoise 3.: denoise current into next
        return StorageIndecies(
            i32((frame + 1u) % 3u),
            i32((frame + 0u) % 3u),
            i32((frame + 2u) % 3u),
        );
    }

    // unused | history | current
    // Denoise 4.: denoise current into the frame
    return StorageIndecies(
        i32((frame + 1u) % 3u),
        i32((frame + 2u) % 3u),
        i32((frame + 0u) % 3u),
    );
}

@compute @workgroup_size(16, 16)
fn compute_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let in_pos = compute_fragment_coords(global_id);
    let coords = vec2<i32>(global_id.xy);
    wang_hash_init(vec2<u32>(coords), render_data.frame);

    let ray = camera_to_ray(render_data.camera, in_pos);
    let hit = sample_ray_trace(coords, ray);
    // let hit = ray_trace(ray);
    let indecies = compute_storage_indecies(render_data.frame, 0u);
    textureStore(moments, coords, indecies.current, packMoment(hit.moment));
}

@compute @workgroup_size(16, 16)
fn compute_denoise1(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let in_pos = compute_fragment_coords(global_id);
    let coords = vec2<i32>(global_id.xy);
    let indecies = compute_storage_indecies(render_data.frame, 0u);
    let hit_moment = load_moment(coords, indecies.current);
    let ray = camera_to_ray(render_data.camera, in_pos);
    let dest = ray.position + ray.dir * hit_moment.depth;
    let moment = temporal_accumulate(hit_moment, indecies, in_pos, dest);
    // textureStore(moments, coords, indecies.next, packMoment(hit_moment));
    textureStore(moments, coords, indecies.next, packMoment(moment));
}

@compute @workgroup_size(16, 16)
fn compute_denoise2(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let indecies = compute_storage_indecies(render_data.frame, 1u);
    textureStore(moments, coords, indecies.next, textureLoad(moments, coords, indecies.current));
    // textureStore(moments, coords, indecies.next, packMoment(denoise(indecies.current, coords, 2.)));
}

@compute @workgroup_size(16, 16)
fn compute_denoise3(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let indecies = compute_storage_indecies(render_data.frame, 2u);
    textureStore(moments, coords, indecies.next, textureLoad(moments, coords, indecies.current));
    // textureStore(moments, coords, indecies.next, packMoment(denoise(indecies.current, coords, 4.)));
}
