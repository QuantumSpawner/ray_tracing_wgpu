const HIT_BRUTE: u32 = 0;
const HIT_BVH: u32 = 1;

const SHADE_FLAT: u32 = 0;
const SHADE_SMOOTH: u32 = 1;

const OBJ_TRIANGLE: u32 = 0;
const OBJ_SPHERE: u32 = 1;

const MAT_DIFFUSE: u32 = 0;
const MAT_REFLECTIVE: u32 = 1;
const MAT_TRANSPARENT: u32 = 2;

struct Stat {
    frame_counter: u32,
}

struct Param {
    camera: CameraParam,
    window_size: vec2<u32>,
    hit_algorithm: u32,
    shading_algorithm: u32,
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
    obj_type: u32,
    mat_idx: u32,
    v: array<vec3<f32>, 3>,
    n: array<vec3<f32>, 3>,
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
