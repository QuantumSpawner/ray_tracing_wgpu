struct HitRecord {
    point: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    sphere_idx: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_new(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    return Ray(origin, normalize(direction));
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

fn camera_get_ray(camera: Camera, tex_coord: vec2<f32>) -> Ray {
    let rd = camera.lens_radius * rng_unit_disk_f32();
    let ray_origin = camera.position + rd.x * camera.horizontal + rd.y * camera.vertical;
    let look_at = camera.start + tex_coord.x * camera.vx + tex_coord.y * camera.vy;
    return ray_new(ray_origin, look_at - ray_origin);
}

fn sphere_hit(sphere: Sphere, ray: Ray, tmin: f32, tmax: f32, hit: ptr<function, HitRecord>) -> bool {
    let oc = ray.origin - sphere.center;
    let b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let disc = b * b - c;

    if disc < 0.0 {
        return false;
    }

    let disc_sqrt = sqrt(disc);
    var root = -b - disc_sqrt;
    if root > tmax || root < tmin {
        root = -b + disc_sqrt;
        if root > tmax || root < tmin {
            return false;
        }
    }

    (*hit).point = ray_at(ray, root);
    (*hit).t = root;
    (*hit).normal = ((*hit).point - sphere.center) / sphere.radius;
    
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
