use cgmath::{InnerSpace, vec3};

use super::{
    object::{Material, Object, Sphere},
    util::random_range,
};

pub fn random_spheres() -> Vec<Box<dyn Object>> {
    let mut objects: Vec<Box<dyn Object>> = Vec::new();

    objects.push(Box::new(Sphere::new(
        vec3(0.0, -1000.0, 0.0),
        1000.0,
        Material::Diffuse {
            albedo: vec3(0.5, 0.5, 0.5),
        },
    )));

    objects.push(Box::new(Sphere::new(
        vec3(4.0, 1.0, 0.0),
        1.0,
        Material::Reflective {
            albedo: vec3(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
    )));

    objects.push(Box::new(Sphere::new(
        vec3(0.0, 1.0, 0.0),
        1.0,
        Material::Transparent {
            albedo: vec3(1.0, 1.0, 1.0),
            ref_idx: 1.5,
        },
    )));

    objects.push(Box::new(Sphere::new(
        vec3(-4.0, 1.0, 0.0),
        1.0,
        Material::Diffuse {
            albedo: vec3(0.4, 0.2, 0.1),
        },
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_range(0.0..1.0);
            let center = vec3(
                a as f32 + 0.9 * random_range(0.0..1.0),
                0.2,
                b as f32 + 0.9 * random_range(0.0..1.0),
            );
            if (center - vec3(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    objects.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Diffuse {
                            albedo: vec3(
                                random_range(0.0..1.0) * random_range(0.0..1.0),
                                random_range(0.0..1.0) * random_range(0.0..1.0),
                                random_range(0.0..1.0) * random_range(0.0..1.0),
                            ),
                        },
                    )));
                } else if choose_mat < 0.95 {
                    // reflective
                    objects.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Reflective {
                            albedo: vec3(
                                0.5 * (1.0 + random_range(0.0..1.0)),
                                0.5 * (1.0 + random_range(0.0..1.0)),
                                0.5 * (1.0 + random_range(0.0..1.0)),
                            ),
                            fuzz: 0.5 * random_range(0.0..1.0),
                        },
                    )));
                } else {
                    // transparent
                    objects.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Transparent {
                            albedo: vec3(1.0, 1.0, 1.0),
                            ref_idx: 1.5,
                        },
                    )));
                }
            }
        }
    }

    objects
}
