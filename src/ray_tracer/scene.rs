use std::sync::{Mutex, OnceLock};

use cgmath::{InnerSpace, vec3};
use rand::prelude::*;

use super::resource::{MAT_DIFFUSE, MAT_REFLECTIVE, MAT_TRANSPARENT, Material, Sphere};

pub fn random_spheres() -> Vec<Sphere> {
    let mut spheres: Vec<Sphere> = Vec::new();

    spheres.push(Sphere {
        center: vec3(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: diffuse([0.5, 0.5, 0.5]),
    });
    spheres.push(Sphere {
        center: vec3(4.0, 1.0, 0.0),
        radius: 1.0,
        material: reflective([0.7, 0.6, 0.5], 0.0),
    });
    spheres.push(Sphere {
        center: vec3(0.0, 1.0, 0.0),
        radius: 1.0,
        material: transparent([1.0, 1.0, 1.0], 1.5),
    });
    spheres.push(Sphere {
        center: vec3(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: diffuse([0.4, 0.2, 0.1]),
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random();
            let center = vec3(a as f32 + 0.9 * random(), 0.2, b as f32 + 0.9 * random());
            if (center - vec3(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        material: diffuse([
                            random() * random(),
                            random() * random(),
                            random() * random(),
                        ]),
                    });
                } else if choose_mat < 0.95 {
                    // reflective
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        material: reflective(
                            [
                                0.5 * (1.0 + random()),
                                0.5 * (1.0 + random()),
                                0.5 * (1.0 + random()),
                            ],
                            0.5 * random(),
                        ),
                    });
                } else {
                    // transparent
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        material: transparent([1.0, 1.0, 1.0], 1.5),
                    });
                }
            }
        }
    }

    spheres
}

pub fn random() -> f32 {
    static RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();
    if RNG.get().is_none() {
        RNG.set(Mutex::new(StdRng::from_os_rng())).unwrap();
    }

    RNG.get().unwrap().lock().unwrap().random_range(0.0..1.0)
}

fn diffuse(albedo: [f32; 3]) -> Material {
    Material {
        mat_type: MAT_DIFFUSE,
        albedo: albedo.into(),
        param1: 0.0,
    }
}

fn reflective(color: [f32; 3], fuzz: f32) -> Material {
    Material {
        mat_type: MAT_REFLECTIVE,
        albedo: color.into(),
        param1: fuzz,
    }
}

fn transparent(albedo: [f32; 3], refr_idx: f32) -> Material {
    Material {
        mat_type: MAT_TRANSPARENT,
        albedo: albedo.into(),
        param1: refr_idx,
    }
}
