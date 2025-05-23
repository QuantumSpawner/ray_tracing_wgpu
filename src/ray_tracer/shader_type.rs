pub const HIT_BRUTE: u32 = 0;
pub const HIT_BVH: u32 = 1;

pub const SHADE_FLAT: u32 = 0;
pub const SHADE_SMOOTH: u32 = 1;

pub const OBJ_TRIANGLE: u32 = 0;
pub const OBJ_SPHERE: u32 = 1;

pub const MAT_DIFFUSE: u32 = 0;
pub const MAT_REFLECTIVE: u32 = 1;
pub const MAT_TRANSPARENT: u32 = 2;

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Stat {
    pub frame_counter: u32,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Param {
    pub camera: Camera,
    pub display_size: cgmath::Vector2<u32>,
    pub hit_algorithm: u32,
    pub shading_algorithm: u32,
    pub max_bounce: u32,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Camera {
    pub position: cgmath::Vector3<f32>,
    pub horizontal: cgmath::Vector3<f32>,
    pub vertical: cgmath::Vector3<f32>,
    pub start: cgmath::Vector3<f32>,
    pub vx: cgmath::Vector3<f32>,
    pub vy: cgmath::Vector3<f32>,
    pub lens_radius: f32,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct AABB {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct BVH {
    pub num_node: encase::ArrayLength,

    #[size(runtime)]
    pub nodes: Vec<BVHNode>,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct BVHNode {
    pub bbox: AABB,
    pub left_idx: i32,
    pub right_idx: i32,
    pub object_idx: i32,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Objects {
    pub num_object: encase::ArrayLength,

    #[size(runtime)]
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Object {
    pub obj_type: u32,
    pub mat_idx: u32,
    // v[0] for the center and v[1].x for the radius of sphere
    pub v: [cgmath::Vector3<f32>; 3],
    pub n: [cgmath::Vector3<f32>; 3],
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Materials {
    pub num_material: encase::ArrayLength,

    #[size(runtime)]
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Material {
    pub mat_type: u32,
    pub albedo: cgmath::Vector3<f32>,

    // fuzz for reflective, refractive index for transparent
    pub param1: f32,
}

impl Default for BVH {
    fn default() -> Self {
        Self {
            num_node: encase::ArrayLength,
            nodes: Vec::new(),
        }
    }
}

impl Default for Materials {
    fn default() -> Self {
        Self {
            num_material: encase::ArrayLength,
            materials: Vec::new(),
        }
    }
}

impl Default for Objects {
    fn default() -> Self {
        Self {
            num_object: encase::ArrayLength,
            objects: Vec::new(),
        }
    }
}
