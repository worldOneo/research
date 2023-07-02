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

const eps = 1.e-3;
const cast_max = 1.e20;

fn cast_to_hit(cube: Cube, ray: Ray) -> f32 {
    var ray = ray;
    let start = ray.position;
    var dist = distance_to_cube(cube, ray.position);
    for (var i = 0; i < 40; i++) {
        if dist < eps {
            return length(start - ray.position);
        }
        ray.position += (dist + 1.e-2) * ray.dir;
        dist = distance_to_cube(cube, ray.position);
    }
    return cast_max;
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
    // let initial_dir = normalize(vec3<f32>(5.0, streched_xy_rot));
    // let rotate_z = create_quaternion_rotation(vec3<f32>(0., 0., 1.), render_data.rotations.x);
    // let z_rotated = quaternion_rotate(rotate_z, initial_dir);
    // let rotate_up = create_quaternion_rotation(cross(initial_dir, vec3<f32>(0., 0., 1.)), -render_data.rotations.y);
    // let up_rotated = quaternion_rotate(rotate_up, z_rotated / z_rotated.y / rotate_z.x);
    // let dir = normalize(vec3<f32>(big_dir.x, render_data.direction.yz + streched_xy_rot));
    let dir = normalize(vec3<f32>(5.0, streched_xy_rot.x, streched_xy_rot.y));
    let rot = create_quaternion_rotation(vec3<f32>(0.0, 1.0, 0.0), render_data.rotations.y);
    let rot_dir = quaternion_rotate(rot, dir);
    let rot2 = create_quaternion_rotation(vec3<f32>(0.0, 0., 1.0), render_data.rotations.x);
    let rot_dir2 = quaternion_rotate(rot2, rot_dir);
    let ray = Ray(render_data.camera, rot_dir2);
    let dist = cast_to_hit(cube, ray);
    var col = vec3<f32>(pow(dist / 10., 4.));
    // return vec4<f32>((in.position.xy + 1.) / 2., 0., 1.0);
    return vec4<f32>(col, 1.0);
}