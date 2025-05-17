/* constant-------------------------------------------------------------------*/
override WORKGROUP_SIZE_X: u32 = 16;
override WORKGROUP_SIZE_Y: u32 = 16;

/* uniform--------------------------------------------------------------------*/
@group(0) @binding(1) var<uniform> stat: Stat;
@group(0) @binding(2) var<uniform> param: Param;

/* buffer---------------------------------------------------------------------*/
@group(0) @binding(0) var<storage, read_write> frame: array<vec3<f32>>;

/* function-------------------------------------------------------------------*/
@compute
@workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy > param.window_size) {
        return;
    }

    rng_init(id.xy, param.window_size, stat.frame_counter);

    let pixel = &frame[id.x + id.y * param.window_size.x];
    if (stat.frame_counter == 0) {
        *pixel = vec3<f32>(0.0, 0.0, 0.0);
    }

    let tex_coord = vec2<f32>((f32(id.x) + rng_f32()) / f32(param.window_size.x),
        (f32(id.y) + rng_f32()) / f32(param.window_size.y));
    let ray = camera_get_ray(param.camera, tex_coord);

    let a = 0.5 * (ray.direction.y + 1.0);
    let color = (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
    *pixel += color;
}

fn camera_get_ray(camera: Camera, tex_coord: vec2<f32>) -> Ray {
    let rd = camera.lens_radius * rng_unit_disk_f32();
    let ray_origin = camera.position + rd.x * camera.horizontal + rd.y * camera.vertical;
    let look_at = camera.start + tex_coord.x * camera.vx + tex_coord.y * camera.vy;
    return Ray(ray_origin, look_at - ray_origin);
}
