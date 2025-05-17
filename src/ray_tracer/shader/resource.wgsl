const MAT_DIFFUSE: u32 = 0;
const MAT_REFLECTIVE: u32 = 1;
const MAT_TRANSPARENT: u32 = 2;

struct Stat {
    frame_counter: u32,
}

struct Param {
    camera: Camera,
    window_size: vec2<u32>,
    max_bounce: u32,
}

struct Camera {
    position: vec3<f32>,
    // unit vectors of the lens plane
    horizontal: vec3<f32>,
    vertical: vec3<f32>,

    start: vec3<f32>,
    // vectors of the focus plane
    vx: vec3<f32>,
    vy: vec3<f32>,

    lens_radius: f32,
}

struct Scene {
    num_sphere: u32,
    spheres: array<Sphere>,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}

struct Material {
    mat_type: u32,
    albedo: vec3<f32>,
    // fuzz for reflective, refractive index for transparent
    param1: f32,
}
