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
pub struct Objects {
    pub num_object: encase::ArrayLength,

    #[size(runtime)]
    pub objects: Vec<Object>,
}

impl Default for Objects {
    fn default() -> Self {
        Self {
            num_object: encase::ArrayLength,
            objects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Object {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
    pub mat_idx: u32,
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Materials {
    pub num_material: encase::ArrayLength,

    #[size(runtime)]
    pub materials: Vec<Material>,
}

impl Default for Materials {
    fn default() -> Self {
        Self {
            num_material: encase::ArrayLength,
            materials: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Material {
    pub mat_type: u32,
    pub albedo: cgmath::Vector3<f32>,

    // fuzz for reflective, refractive index for transparent
    pub param1: f32,
}
