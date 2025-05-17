use cgmath::InnerSpace;

use super::resource;

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub camera: CameraParam,
    pub display_size: cgmath::Vector2<u32>,
    pub max_sample: u32,
    pub max_bounce: u32,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            camera: CameraParam::default(),
            display_size: cgmath::Vector2::new(1, 1),
            max_sample: 256,
            max_bounce: 8,
        }
    }
}

impl Param {
    pub fn into_param(self) -> resource::Param {
        resource::Param {
            camera: self
                .camera
                .into_camera(self.display_size.x as f32 / self.display_size.y as f32),
            display_size: self.display_size,
            max_bounce: self.max_bounce,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraParam {
    pub position: cgmath::Vector3<f32>,
    pub look_at: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub fov: f32,
    pub aperture: f32,
    pub focus_distance: f32,
}

impl Default for CameraParam {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            look_at: cgmath::Vector3::new(0.0, 0.0, -1.0),
            up: cgmath::Vector3::new(0.0, 1.0, 0.0),
            fov: 90.0,
            aperture: 0.0,
            focus_distance: 1.0,
        }
    }
}

impl CameraParam {
    pub fn into_camera(self, aspect_ratio: f32) -> resource::Camera {
        let w = (self.position - self.look_at).normalize();
        let u = self.up.cross(w).normalize();
        let v = w.cross(u).normalize();

        let fov = self.fov.to_radians();
        let height = self.focus_distance * (fov / 2.0).tan();
        let width = height * aspect_ratio;

        let start = self.position - width * u + height * v - self.focus_distance * w;

        let result = resource::Camera {
            position: self.position,
            horizontal: u,
            vertical: v,

            start,
            vx: (2.0 * width * u),
            vy: (-2.0 * height * v),

            lens_radius: self.aperture / 2.0,
        };

        result
    }
}
