use super::{shader_type, util};

pub trait Object {
    fn as_shader_type(&self, mat_idx: u32) -> shader_type::Object;
    fn material(&self) -> &Material;
    fn bbox(&self) -> &AABB;
}

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

pub struct BVHNode {
    pub bbox: AABB,
    pub left_idx: i32,
    pub right_idx: i32,
    pub object_idx: i32,
}

pub struct Sphere {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
    pub material: Material,
    pub bbox: AABB,
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

impl AABB {
    pub fn new(p: cgmath::Vector3<f32>, q: cgmath::Vector3<f32>) -> Self {
        Self {
            min: cgmath::Vector3::new(p.x.min(q.x), p.y.min(q.y), p.z.min(q.z)),
            max: cgmath::Vector3::new(p.x.max(q.x), p.y.max(q.y), p.z.max(q.z)),
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: cgmath::Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: cgmath::Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn as_shader_type(&self) -> shader_type::AABB {
        shader_type::AABB {
            min: self.min,
            max: self.max,
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: cgmath::vec3(0.0, 0.0, 0.0),
            max: cgmath::vec3(0.0, 0.0, 0.0),
        }
    }
}

impl BVHNode {
    pub fn as_shader_type(&self) -> shader_type::BVHNode {
        shader_type::BVHNode {
            bbox: self.bbox.as_shader_type(),
            left_idx: self.left_idx,
            right_idx: self.right_idx,
            object_idx: self.object_idx,
        }
    }
}

impl Default for BVHNode {
    fn default() -> Self {
        Self {
            bbox: AABB::default(),
            left_idx: -1,
            right_idx: -1,
            object_idx: -1,
        }
    }
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32, material: Material) -> Self {
        let bbox = AABB::new(
            center - cgmath::Vector3::new(radius, radius, radius),
            center + cgmath::Vector3::new(radius, radius, radius),
        );
        Self {
            center,
            radius,
            material,
            bbox,
        }
    }
}

impl Object for Sphere {
    fn as_shader_type(&self, mat_idx: u32) -> shader_type::Object {
        shader_type::Object {
            mat_idx,
            center: self.center,
            radius: self.radius,
        }
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn bbox(&self) -> &AABB {
        &self.bbox
    }
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

pub fn as_shader_type(
    _objects: &[Box<dyn Object>],
) -> (shader_type::Objects, shader_type::Materials) {
    let mut objects = Vec::new();
    let mut materials = Vec::new();

    for object in _objects {
        objects.push(object.as_shader_type(materials.len() as u32));
        materials.push(object.material().as_shader_type());
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

pub fn build_bvh(_objects: &[Box<dyn Object>]) -> shader_type::BVH {
    fn build(objects: &mut [(usize, &AABB)], result: &mut Vec<BVHNode>) -> usize {
        if objects.len() == 1 {
            let (idx, bbox) = objects[0];
            result.push(BVHNode {
                bbox: bbox.clone(),
                object_idx: idx as i32,
                ..Default::default()
            });

            return result.len() - 1;
        }

        // could optimize to split at the longest axis, but for now just pick a
        // random axis
        let axis = util::random_range(0..3);
        objects.sort_by(|a, b| a.1.min[axis].partial_cmp(&b.1.min[axis]).unwrap());

        let mid = objects.len() / 2;
        let left = build(&mut objects[..mid], result);
        let right = build(&mut objects[mid..], result);

        result.push(BVHNode {
            bbox: result[left].bbox.union(&result[right].bbox),
            left_idx: left as i32,
            right_idx: right as i32,
            ..Default::default()
        });

        result.len() - 1
    }

    let mut objects: Vec<(usize, &AABB)> = _objects
        .iter()
        .enumerate()
        .map(|(i, obj)| (i, obj.bbox()))
        .collect();
    let mut result = vec![BVHNode::default()];

    let root_idx = build(&mut objects, &mut result);
    result.swap(0, root_idx);

    shader_type::BVH {
        nodes: result
            .into_iter()
            .map(|node| node.as_shader_type())
            .collect(),
        ..Default::default()
    }
}
