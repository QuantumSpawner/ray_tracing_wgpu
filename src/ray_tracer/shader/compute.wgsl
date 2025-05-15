/* constant-------------------------------------------------------------------*/
override WORKGROUP_SIZE_X: u32 = 16;
override WORKGROUP_SIZE_Y: u32 = 16;

/* uniform--------------------------------------------------------------------*/
@group(0) @binding(0) var<uniform> param: Param;
@group(0) @binding(1) var<uniform> stat: Stat;

/* buffer---------------------------------------------------------------------*/
@group(0) @binding(2) var<storage, read_write> frame: array<vec3<f32>>;

/* function-------------------------------------------------------------------*/
@compute
@workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy > param.window_size) {
        return;
    }

    let pixel = &frame[id.x + id.y * param.window_size.x];

    if (stat.frame_counter == 0) {
        *pixel = vec3<f32>(0.0, 0.0, 0.0);
    }

    rng_init(id.xy, param.window_size, stat.frame_counter);

    *pixel += rng_unit_disk_f32();
}
