use std::sync::{Mutex, OnceLock};

use cgmath::{InnerSpace, vec3};
use rand::prelude::*;

use super::object::{Material, Object};

pub fn random_spheres() -> Vec<Object> {
    let mut objects: Vec<Object> = Vec::new();

    objects.push(Object::Sphere {
        center: vec3(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Material::Diffuse {
            albedo: vec3(0.5, 0.5, 0.5),
        },
    });

    objects.push(Object::Sphere {
        center: vec3(4.0, 1.0, 0.0),
        radius: 1.0,
        material: Material::Reflective {
            albedo: vec3(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
    });

    objects.push(Object::Sphere {
        center: vec3(0.0, 1.0, 0.0),
        radius: 1.0,
        material: Material::Transparent {
            albedo: vec3(1.0, 1.0, 1.0),
            ref_idx: 1.5,
        },
    });

    objects.push(Object::Sphere {
        center: vec3(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: Material::Diffuse {
            albedo: vec3(0.4, 0.2, 0.1),
        },
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random();
            let center = vec3(a as f32 + 0.9 * random(), 0.2, b as f32 + 0.9 * random());
            if (center - vec3(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    objects.push(Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Diffuse {
                            albedo: vec3(
                                random() * random(),
                                random() * random(),
                                random() * random(),
                            ),
                        },
                    });
                } else if choose_mat < 0.95 {
                    // reflective
                    objects.push(Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Reflective {
                            albedo: vec3(
                                0.5 * (1.0 + random()),
                                0.5 * (1.0 + random()),
                                0.5 * (1.0 + random()),
                            ),
                            fuzz: 0.5 * random(),
                        },
                    });
                } else {
                    // transparent
                    objects.push(Object::Sphere {
                        center,
                        radius: 0.2,
                        material: Material::Transparent {
                            albedo: vec3(1.0, 1.0, 1.0),
                            ref_idx: 1.5,
                        },
                    });
                }
            }
        }
    }

    objects
}

fn random() -> f32 {
    static RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();
    if RNG.get().is_none() {
        RNG.set(Mutex::new(StdRng::from_os_rng())).unwrap();
    }

    RNG.get().unwrap().lock().unwrap().random_range(0.0..1.0)
}
