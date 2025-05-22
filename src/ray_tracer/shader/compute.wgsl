/* constant-------------------------------------------------------------------*/
const BVH_MAX_STACK: u32 = 32;

override WORKGROUP_SIZE_X: u32 = 16;
override WORKGROUP_SIZE_Y: u32 = 16;

/* uniform--------------------------------------------------------------------*/
@group(0) @binding(0) var<uniform> stat: Stat;
@group(0) @binding(1) var<uniform> param: Param;

/* buffer---------------------------------------------------------------------*/
@group(1) @binding(0) var<storage, read_write> frame: array<vec3<f32>>;
@group(1) @binding(1) var<storage, read> bvh: BVH;
@group(1) @binding(2) var<storage, read> objects: Objects;
@group(1) @binding(3) var<storage, read> materials: Materials;

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

        if (hit(ray, Interval(0.001, 1000.0), &hit)) {
            let object = objects.objects[hit.object_idx];
            let material = materials.materials[object.mat_idx];

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

fn hit(ray: Ray, interval: Interval, hit: ptr<function, HitRecord>) -> bool {
    switch param.hit_algorithm {
        case HIT_BRUTE: {
            return brute_hit(ray, interval, hit);
        }

        case HIT_BVH: {
            return bvh_hit(ray, interval, hit);
        }

        default: {
            return false;
        }
    }
}

fn brute_hit(ray: Ray, _interval: Interval, _hit: ptr<function, HitRecord>) -> bool {
    var interval = _interval;
    var hit = false;

    for (var i = 0; i < i32(objects.num_object); i++) {
        if (object_hit(objects.objects[i], ray, interval, _hit)) {
            (*_hit).object_idx = i;

            interval.max = (*_hit).t;
            hit = true;
        }
    }

    return hit;
}

fn bvh_hit(ray: Ray, _interval: Interval, _hit: ptr<function, HitRecord>) -> bool {
    var interval = _interval;
    var hit = false;

    var stack: array<i32, BVH_MAX_STACK>;
    stack[0] = 0;
    var stack_top = 0;

    while (stack_top >= 0) {
        let node = bvh.nodes[stack[stack_top]];
        stack_top--;

        if (!bbox_hit(node.bbox, ray, interval)) {
            continue;
        }

        if (node.object_idx >= 0) {
            if (object_hit(objects.objects[node.object_idx], ray, interval, _hit)) {
                (*_hit).object_idx = node.object_idx;

                interval.max = (*_hit).t;
                hit = true;
            }

            // !object_idx < 0 only for leaf nodes
            continue;
        }

        if (node.left_idx >= 0) {
            stack_top++;
            stack[stack_top] = node.left_idx;
        }
        if (node.right_idx >= 0) {
            stack_top++;
            stack[stack_top] = node.right_idx;
        }
    }

    return hit;
}
