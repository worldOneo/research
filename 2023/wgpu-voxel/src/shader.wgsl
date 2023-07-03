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

// Fragment shader

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

struct Cube {
    position: vec3<f32>,
    size: f32,
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

fn cube_planes_ray_intersection_dist(cube: Cube, ray: Ray) -> f32 {
    return cube_max_marchable_distance(cube, ray.position, ray.dir);
}

struct Ray {
    position: vec3<f32>,
    dir: vec3<f32>,
}

fn create_cube(c: vec3<f32>, size: f32) -> Cube {
    return Cube(c, size);
}

fn distance_to_cube(cube: Cube, point: vec3<f32>) -> f32 {
    var d = abs((cube.position + cube.size / 2.) - point) - cube.size / 2.;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, vec3<f32>(0.0)));
}


const eps = 1.e-4;
const cast_max = 1.e20;
const MAX_STEPS = 5;

struct HitResult {
    jumps: i32,
    distance: f32,
    destination: vec3<f32>,
}

fn cast_to_hit(cube: Cube, ray: Ray) -> HitResult {
    var ray = ray;
    let start = ray.position;
    var travelled = 0.;
    var dist = cube_planes_ray_intersection_dist(cube, ray);
    for (var i = 0; i < MAX_STEPS; i++) {
        travelled += dist;
        if distance_to_cube(cube, ray.position) < eps {
            return HitResult(MAX_STEPS, length(start - ray.position), ray.position);
        }
        if travelled > 1000. {
            return HitResult(i, cast_max, ray.position);
        }
        ray.position += (dist + 1.e-3) * ray.dir;
        dist = cube_planes_ray_intersection_dist(cube, ray);
    }
    return HitResult(MAX_STEPS, cast_max, ray.position);
}

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

    let dir = normalize(vec3<f32>(5.0, streched_xy_rot.x, streched_xy_rot.y));
    let rot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), render_data.rotations.y);
    let rot_dir = quaternion_rotate(rot, dir);
    let rot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), render_data.rotations.x);
    let rot_dir2 = quaternion_rotate(rot2, rot_dir);
    let ray = Ray(render_data.camera, rot_dir2);
    let hit = cast_to_hit(cube, ray);
    var col = vec3<f32>(pow(hit.distance / 10., 4.));

    return vec4<f32>(col, 1.0);
}