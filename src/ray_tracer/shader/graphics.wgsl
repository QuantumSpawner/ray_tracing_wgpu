struct Interval {
    min: f32,
    max: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitRecord {
    point: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    object_idx: i32,
}

fn interval_contains(interval: Interval, value: f32) -> bool {
    return value >= interval.min && value <= interval.max;
}

fn interval_overlaps(interval1: Interval, interval2: Interval) -> bool {
    return interval1.min <= interval2.max && interval2.min <= interval1.max;
}

fn ray_new(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    return Ray(origin, normalize(direction));
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn camera_get_ray(camera: CameraParam, tex_coord: vec2<f32>) -> Ray {
    let rd = camera.lens_radius * rng_unit_disk_f32();
    let ray_origin = camera.position + rd.x * camera.horizontal + rd.y * camera.vertical;
    let look_at = camera.start + tex_coord.x * camera.vx + tex_coord.y * camera.vy;
    return ray_new(ray_origin, look_at - ray_origin);
}

fn bbox_hit(bbox: AABB, ray: Ray, interval: Interval) -> bool {
    let t0 = (bbox.min - ray.origin) / ray.direction;
    let t1 = (bbox.max - ray.origin) / ray.direction;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    let tmin_max = max(tmin.x, max(tmin.y, tmin.z));
    let tmax_min = min(tmax.x, min(tmax.y, tmax.z));
    return tmin_max < tmax_min && interval_overlaps(interval, Interval(tmin_max, tmax_min));
}

fn object_hit(object: Object, ray: Ray, interval: Interval, hit: ptr<function, HitRecord>) -> bool {
    return sphere_hit(object, ray, interval, hit);
}

fn sphere_hit(object: Object, ray: Ray, interval: Interval, hit: ptr<function, HitRecord>) -> bool {
    let oc = ray.origin - object.center;
    let b = dot(oc, ray.direction);
    let c = dot(oc, oc) - object.radius * object.radius;
    let disc = b * b - c;

    if disc < 0.0 {
        return false;
    }

    let disc_sqrt = sqrt(disc);
    var root = -b - disc_sqrt;
    if !interval_contains(interval, root) {
        root = -b + disc_sqrt;
        if !interval_contains(interval, root) {
            return false;
        }
    }

    (*hit).point = ray_at(ray, root);
    (*hit).t = root;
    (*hit).normal = ((*hit).point - object.center) / object.radius;
    
    return true;
}

fn material_scatter(material: Material, insert: Ray, hit: HitRecord) -> Ray {
    switch material.mat_type {
        case MAT_DIFFUSE: {
            return diffuse_scatter(material, insert, hit);
        }

        case MAT_REFLECTIVE: {
            return reflective_scatter(material, insert, hit);
        }

        case MAT_TRANSPARENT: {
            return transparent_scatter(material, insert, hit);
        }

        default: {
            return Ray(vec3<f32>(0.0), vec3<f32>(0.0));
        }
    }
}

fn diffuse_scatter(material: Material, insert: Ray, hit: HitRecord) -> Ray {
    let diffused = hit.normal + rng_unit_sphere_f32();
    return ray_new(hit.point, diffused);
}

fn reflective_scatter(material: Material, insert: Ray, hit: HitRecord) -> Ray {
    let reflected = reflect(insert.direction, hit.normal);
    return ray_new(hit.point, reflected + material.param1 * rng_unit_sphere_f32());
}

fn transparent_scatter(material: Material, insert: Ray, hit: HitRecord) -> Ray {    
    var normal_out: vec3<f32>;
    var ref_ratio: f32;
    var cos: f32;

    // hitting form the inside
    if dot(insert.direction, hit.normal) > 0.0 {
        normal_out = -hit.normal;
        ref_ratio = material.param1;
        cos = material.param1 * dot(insert.direction, hit.normal) / length(insert.direction);
    } else {
        normal_out = hit.normal;
        ref_ratio = 1.0 / material.param1;
        cos = -dot(insert.direction, hit.normal) / length(insert.direction);
    }

    var direction = refract(insert.direction, normal_out, ref_ratio);
    if (all(direction == vec3<f32>(0.0)) || schlick(cos, ref_ratio) > rng_f32()) {
        direction = reflect(insert.direction, normal_out);
    }

    return ray_new(hit.point, direction);
}
