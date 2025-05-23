use cgmath::{Deg, EuclideanSpace, SquareMatrix, Transform};

use super::{shader_type, util};

pub trait Object {
    fn as_prims(&self) -> Vec<Box<dyn ObjPrim>>;
    fn material(&self) -> &Material;
}

pub trait ObjPrim {
    fn as_shader_type(&self, mat_idx: u32) -> shader_type::Object;
    fn bbox(&self) -> &AABB;
}

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct BVHNode {
    pub bbox: AABB,
    pub left_idx: i32,
    pub right_idx: i32,
    pub object_idx: i32,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    vertices: Vec<cgmath::Vector3<f32>>,
    normals: Vec<cgmath::Vector3<f32>>,
    indices: Vec<(usize, usize)>,
    transform: cgmath::Matrix4<f32>,
    material: Material,
}

#[derive(Debug, Clone)]
pub struct Sphere {
    prim: SpherePrim,
    material: Material,
}

#[derive(Debug, Clone)]
struct Triangle {
    pub v: [cgmath::Vector3<f32>; 3],
    pub n: [cgmath::Vector3<f32>; 3],
    pub bbox: AABB,
}

#[derive(Debug, Clone)]
struct SpherePrim {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
    pub bbox: AABB,
}

#[derive(Debug, Clone)]
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
            min: cgmath::vec3(p.x.min(q.x), p.y.min(q.y), p.z.min(q.z)),
            max: cgmath::vec3(p.x.max(q.x), p.y.max(q.y), p.z.max(q.z)),
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: cgmath::vec3(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: cgmath::vec3(
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

impl Mesh {
    pub fn load_obj(model: &str, material: Material) -> Self {
        let (models, _) = tobj::load_obj(
            model,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();

        let mesh = &models[0].mesh;
        let vertices = mesh
            .positions
            .chunks(3)
            .map(|v| cgmath::vec3(v[0], v[1], v[2]))
            .collect();
        let normals = mesh
            .normals
            .chunks(3)
            .map(|n| cgmath::vec3(n[0], n[1], n[2]))
            .collect();
        let indices = mesh
            .indices
            .iter()
            .zip(mesh.normal_indices.iter())
            .map(|(i, j)| (*i as usize, *j as usize))
            .collect();

        Self {
            vertices,
            normals,
            indices,
            transform: cgmath::Matrix4::identity(),
            material,
        }
    }

    pub fn translate(mut self, translation: cgmath::Vector3<f32>) -> Self {
        self.transform = cgmath::Matrix4::from_translation(translation) * self.transform;

        self
    }

    pub fn rotate(mut self, rotation: cgmath::Euler<Deg<f32>>) -> Self {
        self.transform = cgmath::Matrix4::from(rotation) * self.transform;

        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.transform = cgmath::Matrix4::from_scale(scale) * self.transform;

        self
    }

    fn transform_point(&self, v: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        self.transform
            .transform_point(cgmath::point3(v.x, v.y, v.z))
            .to_vec()
    }
}

impl Object for Mesh {
    fn as_prims(&self) -> Vec<Box<dyn ObjPrim>> {
        self.indices
            .chunks(3)
            .map(|idx| {
                let v = [
                    self.transform_point(self.vertices[idx[0].0]),
                    self.transform_point(self.vertices[idx[1].0]),
                    self.transform_point(self.vertices[idx[2].0]),
                ];
                let n = [
                    self.transform.transform_vector(self.normals[idx[0].1]),
                    self.transform.transform_vector(self.normals[idx[1].1]),
                    self.transform.transform_vector(self.normals[idx[2].1]),
                ];
                let bbox = AABB::new(v[0], v[1]).union(&AABB::new(v[0], v[2]));

                Box::new(Triangle { v, n, bbox }) as Box<dyn ObjPrim>
            })
            .collect()
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32, material: Material) -> Self {
        let bbox = AABB::new(
            center - cgmath::vec3(radius, radius, radius),
            center + cgmath::vec3(radius, radius, radius),
        );
        Self {
            prim: SpherePrim {
                center,
                radius,
                bbox,
            },
            material,
        }
    }
}

impl Object for Sphere {
    fn as_prims(&self) -> Vec<Box<dyn ObjPrim>> {
        vec![Box::new(self.prim.clone())]
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

impl ObjPrim for Triangle {
    fn as_shader_type(&self, mat_idx: u32) -> shader_type::Object {
        shader_type::Object {
            obj_type: shader_type::OBJ_TRIANGLE,
            mat_idx,
            v: self.v,
            n: self.n,
        }
    }

    fn bbox(&self) -> &AABB {
        &self.bbox
    }
}

impl ObjPrim for SpherePrim {
    fn as_shader_type(&self, mat_idx: u32) -> shader_type::Object {
        shader_type::Object {
            obj_type: shader_type::OBJ_SPHERE,
            mat_idx,
            v: [
                self.center,
                cgmath::vec3(self.radius, 0.0, 0.0),
                cgmath::vec3(0.0, 0.0, 0.0),
            ],
            n: [
                cgmath::vec3(0.0, 0.0, 0.0),
                cgmath::vec3(0.0, 0.0, 0.0),
                cgmath::vec3(0.0, 0.0, 0.0),
            ],
        }
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

pub fn as_shader_types(
    _objects: &[Box<dyn Object>],
) -> (
    shader_type::BVH,
    shader_type::Objects,
    shader_type::Materials,
) {
    let mut primitives = Vec::new();
    let mut objects = Vec::new();
    let mut materials = Vec::new();

    for object in _objects {
        let prims = object.as_prims();

        objects.extend(
            prims
                .iter()
                .map(|p| p.as_shader_type(materials.len() as u32)),
        );
        primitives.extend(prims);
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

    (build_bvh(&primitives), objects, materials)
}

fn build_bvh(_objects: &[Box<dyn ObjPrim>]) -> shader_type::BVH {
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
