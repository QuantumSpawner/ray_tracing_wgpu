use super::shader_type;

pub enum Object {
    Sphere {
        center: cgmath::Vector3<f32>,
        radius: f32,
        material: Material,
    },
}

pub fn as_shader_type(_objects: &[Object]) -> (shader_type::Objects, shader_type::Materials) {
    let mut objects = Vec::new();
    let mut materials = Vec::new();

    for object in _objects {
        match object {
            Object::Sphere {
                center,
                radius,
                material,
            } => {
                objects.push(shader_type::Object {
                    center: center.clone(),
                    radius: *radius,
                    mat_idx: materials.len() as u32,
                });
                materials.push(material.as_shader_type());
            }
        }
    }

    let objects = shader_type::Objects {
        objects,
        ..Default::default()
    };

    let materials = shader_type::Materials {
        materials,
        ..Default::default()
    };

    (objects, materials)
}

pub enum Material {
    Diffuse {
        albedo: cgmath::Vector3<f32>,
    },
    Reflective {
        albedo: cgmath::Vector3<f32>,
        fuzz: f32,
    },
    Transparent {
        albedo: cgmath::Vector3<f32>,
        ref_idx: f32,
    },
}

impl Material {
    pub fn as_shader_type(&self) -> shader_type::Material {
        match self {
            Material::Diffuse { albedo } => shader_type::Material {
                mat_type: shader_type::MAT_DIFFUSE,
                albedo: *albedo,
                param1: 0.0,
            },
            Material::Reflective { albedo, fuzz } => shader_type::Material {
                mat_type: shader_type::MAT_REFLECTIVE,
                albedo: *albedo,
                param1: *fuzz,
            },
            Material::Transparent { albedo, ref_idx } => shader_type::Material {
                mat_type: shader_type::MAT_TRANSPARENT,
                albedo: *albedo,
                param1: *ref_idx,
            },
        }
    }
}
