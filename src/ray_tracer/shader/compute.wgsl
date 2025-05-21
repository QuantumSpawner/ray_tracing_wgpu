/* constant-------------------------------------------------------------------*/
override WORKGROUP_SIZE_X: u32 = 16;
override WORKGROUP_SIZE_Y: u32 = 16;

/* uniform--------------------------------------------------------------------*/
@group(0) @binding(0) var<uniform> stat: Stat;
@group(0) @binding(1) var<uniform> param: Param;

/* buffer---------------------------------------------------------------------*/
@group(1) @binding(0) var<storage, read_write> frame: array<vec3<f32>>;
@group(1) @binding(1) var<storage, read> objects: Objects;
@group(1) @binding(2) var<storage, read> materials: Materials;

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
    let color = color(ray, param.max_bounce);

    *pixel += color;
}

fn color(_ray: Ray, max_bounce: u32) -> vec3<f32> {
    var ray = _ray;

    var albedo = vec3<f32>(1.0, 1.0, 1.0);
    var color: vec3<f32>;

    for (var i: u32 = 0; i < max_bounce; i += 1) {
        var hit: HitRecord;

        if (spheres_hit(ray, 0.001, 1000.0, &hit)) {
            let material = materials.materials[objects.objects[hit.sphere_idx].mat_idx];

            albedo *= material.albedo;
            ray = material_scatter(material, ray, hit);
        } else {
            // ray missed, output background color
            let t = 0.5 * (ray.direction.y + 1.0);
            color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
        
            break;
        }
    }

    return albedo * color;
}

fn spheres_hit(ray: Ray, tmin: f32, tmax: f32, _hit: ptr<function, HitRecord>) -> bool {
    var closest = tmax;
    var hit = false;

    for (var i: u32 = 0; i < objects.num_objects; i += 1) {
        if (sphere_hit(objects.objects[i], ray, tmin, closest, _hit)) {
            (*_hit).sphere_idx = i;

            closest = (*_hit).t;
            hit = true;
        }
    }

    return hit;
}
