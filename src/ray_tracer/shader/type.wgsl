const HIT_BRUTE: u32 = 0;
const HIT_BVH: u32 = 1;

const MAT_DIFFUSE: u32 = 0;
const MAT_REFLECTIVE: u32 = 1;
const MAT_TRANSPARENT: u32 = 2;

struct Stat {
    frame_counter: u32,
}

struct Param {
    camera: CameraParam,
    hit_algorithm: u32,
    window_size: vec2<u32>,
    max_bounce: u32,
}

struct CameraParam {
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

struct AABB {
    min: vec3f,
    max: vec3f,
}

struct BVH {
    num_node: u32,
    nodes: array<BVHNode>,
}

struct BVHNode {
    bbox: AABB,
    left_idx: i32,
    right_idx: i32,
    object_idx: i32,
}

struct Objects {
    num_object: u32,
    objects: array<Object>,
}

struct Object {
    mat_idx: u32,
    center: vec3<f32>,
    radius: f32,
}

struct Materials {
    num_material: u32,
    materials: array<Material>,
}

struct Material {
    mat_type: u32,
    albedo: vec3<f32>,
    // fuzz for reflective, refractive index for transparent
    param1: f32,
}
