mod buffer;
mod object;
mod scene;
mod shader_type;
mod util;

use std::{
    collections::HashMap,
    mem::size_of,
    time::{Duration, Instant},
};

use crate::wgpu;

const WORKGROUP_SIZE_X: u32 = 16;
const WORKGROUP_SIZE_Y: u32 = 16;

const MAX_WINDOW_SIZE_X: u32 = 1920;
const MAX_WINDOW_SIZE_Y: u32 = 1080;

pub struct RayTracer {
    stat: Stat,
    param: Param,

    stat_uniform: buffer::UniformBuffer<shader_type::Stat>,
    param_uniform: buffer::UniformBuffer<shader_type::Param>,

    render_pipeline: wgpu::RenderPipeline,
    render_uniform_bind_group: wgpu::BindGroup,
    render_storage_bind_group: wgpu::BindGroup,

    compute_pipeline: wgpu::ComputePipeline,
    compute_uniform_bind_group: wgpu::BindGroup,
    compute_storage_bind_group: wgpu::BindGroup,
}

#[derive(Debug, Clone)]
pub struct Stat {
    pub is_rendering: bool,
    pub frame_counter: u32,
    pub time_spent: Duration,
    pub time_start: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub camera: CameraParam,
    pub display_size: cgmath::Vector2<u32>,
    pub hit_algorithm: HitAlgorithm,
    pub shading_algorithm: ShadingAlgorithm,
    pub max_sample: u32,
    pub max_bounce: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitAlgorithm {
    Brute,
    BVH,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadingAlgorithm {
    Flat,
    Smooth,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraParam {
    pub position: cgmath::Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aperture: f32,
    pub focus_distance: f32,
}

impl RayTracer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target: wgpu::ColorTargetState,
    ) -> Self {
        let stat = Stat::default();
        let param = Param::default();

        let objects = scene::random_spheres();
        let (bvh, objects, materials) = object::as_shader_types(&objects);

        /* resource-----------------------------------------------------------*/
        let stat_uniform =
            buffer::UniformBuffer::new(device, &stat.as_shader_type(), Some("Ray Tracer State"));
        let param_uniform = buffer::UniformBuffer::new(
            device,
            &param.clone().as_shader_type(),
            Some("Ray Tracer Parameter"),
        );
        let frame_buffer_storage = buffer::StorageBuffer::<false>::new_with_size(
            device,
            MAX_WINDOW_SIZE_X as usize * MAX_WINDOW_SIZE_Y as usize * 4 * size_of::<f32>(),
            Some("Ray Tracer Frame Buffer"),
        );
        let bvh_storage = buffer::StorageBuffer::<true>::new(device, &bvh, Some("Ray Tracer BVH"));
        let objects_storage =
            buffer::StorageBuffer::<true>::new(device, &objects, Some("Ray Tracer Objects"));
        let material_storage =
            buffer::StorageBuffer::<true>::new(device, &materials, Some("Ray Tracer Materials"));

        /* render shader------------------------------------------------------*/
        let render_shader_source = [
            include_str!("shader/type.wgsl"),
            include_str!("shader/render.wgsl"),
        ]
        .join("\n");
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracer Render Shader"),
            source: wgpu::ShaderSource::Wgsl(render_shader_source.into()),
        });

        let (render_uniform_bind_group_layout, render_uniform_bind_group) = create_bind_group(
            device,
            &[&stat_uniform, &param_uniform],
            wgpu::ShaderStages::FRAGMENT,
            Some("Ray Tracer Render Uniform"),
        );

        let (render_storage_bind_group_layout, render_storage_bind_group) = create_bind_group(
            device,
            &[&frame_buffer_storage],
            wgpu::ShaderStages::FRAGMENT,
            Some("Ray Tracer Render Storage"),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracer Render Pipeline Layout"),
                bind_group_layouts: &[
                    &render_uniform_bind_group_layout,
                    &render_storage_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ray Tracer Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        /* compute shader-----------------------------------------------------*/
        let compute_shader_source = [
            include_str!("shader/type.wgsl"),
            include_str!("shader/util.wgsl"),
            include_str!("shader/graphics.wgsl"),
            include_str!("shader/compute.wgsl"),
        ]
        .join("\n");
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracer Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(compute_shader_source.into()),
        });

        let (compute_uniform_bind_group_layout, compute_uniform_bind_group) = create_bind_group(
            device,
            &[&stat_uniform, &param_uniform],
            wgpu::ShaderStages::COMPUTE,
            Some("Ray Tracer Comupte"),
        );

        let (compute_storage_bind_group_layout, compute_storage_bind_group) = create_bind_group(
            device,
            &[
                &frame_buffer_storage,
                &bvh_storage,
                &objects_storage,
                &material_storage,
            ],
            wgpu::ShaderStages::COMPUTE,
            Some("Ray Tracer Compute Storage"),
        );

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracer Compute Pipeline Layout"),
                bind_group_layouts: &[
                    &compute_uniform_bind_group_layout,
                    &compute_storage_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let compute_shader_constants = HashMap::from([
            ("WORKGROUP_SIZE_X".to_string(), WORKGROUP_SIZE_X as f64),
            ("WORKGROUP_SIZE_Y".to_string(), WORKGROUP_SIZE_Y as f64),
        ]);

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracer Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &compute_shader_constants,
                ..Default::default()
            },
            cache: None,
        });

        Self {
            stat,
            param,

            stat_uniform,
            param_uniform,

            render_pipeline,
            render_uniform_bind_group,
            render_storage_bind_group,

            compute_pipeline,
            compute_uniform_bind_group,
            compute_storage_bind_group,
        }
    }

    pub fn get_params(&self) -> &Param {
        &self.param
    }

    pub fn set_params(&mut self, queue: &wgpu::Queue, param: &Param) {
        if self.param == *param {
            return;
        }

        self.param_uniform.set_data(queue, &param.as_shader_type());
        self.param = param.clone();

        self.reset();
    }

    pub fn get_stat(&self) -> &Stat {
        &self.stat
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.stat.is_rendering {
            return;
        }

        if self.stat.frame_counter >= self.param.max_sample {
            self.stat.is_rendering = false;
            self.stat.time_spent = self.stat.time_start.elapsed();
            return;
        }

        self.stat_uniform
            .set_data(queue, &self.stat.as_shader_type());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Ray Tracer Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracer Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_uniform_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.compute_storage_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.param.display_size.x / WORKGROUP_SIZE_X + 1,
                self.param.display_size.y / WORKGROUP_SIZE_Y + 1,
                1,
            );
        }

        queue.submit(Some(encoder.finish()));

        self.stat.frame_counter += 1;
        self.stat.time_spent = self.stat.time_start.elapsed();
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.render_storage_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn reset(&mut self) {
        self.stat = Stat::default();
    }
}

impl Stat {
    fn as_shader_type(&self) -> shader_type::Stat {
        shader_type::Stat {
            frame_counter: self.frame_counter,
        }
    }
}

impl Default for Stat {
    fn default() -> Self {
        Self {
            is_rendering: true,
            frame_counter: 0,
            time_spent: Duration::ZERO,
            time_start: Instant::now(),
        }
    }
}

impl Param {
    pub fn as_shader_type(&self) -> shader_type::Param {
        shader_type::Param {
            camera: self
                .camera
                .as_shader_type(self.display_size.x as f32 / self.display_size.y as f32),
            display_size: self.display_size,
            hit_algorithm: match self.hit_algorithm {
                HitAlgorithm::Brute => shader_type::HIT_BRUTE,
                HitAlgorithm::BVH => shader_type::HIT_BVH,
            },
            shading_algorithm: match self.shading_algorithm {
                ShadingAlgorithm::Flat => shader_type::SHADE_FLAT,
                ShadingAlgorithm::Smooth => shader_type::SHADE_SMOOTH,
            },
            max_bounce: self.max_bounce,
        }
    }
}

impl Default for Param {
    fn default() -> Self {
        Self {
            camera: CameraParam::default(),
            display_size: cgmath::Vector2::new(1, 1),
            hit_algorithm: HitAlgorithm::BVH,
            shading_algorithm: ShadingAlgorithm::Smooth,
            max_sample: 256,
            max_bounce: 8,
        }
    }
}

impl CameraParam {
    pub fn as_shader_type(&self, aspect_ratio: f32) -> shader_type::Camera {
        let rot_matrix = cgmath::Matrix3::from_angle_y(cgmath::Deg(self.yaw))
            * cgmath::Matrix3::from_angle_x(cgmath::Deg(self.pitch));
        let w = rot_matrix * cgmath::vec3(0.0, 0.0, 1.0);
        let u = rot_matrix * cgmath::vec3(1.0, 0.0, 0.0);
        let v = rot_matrix * cgmath::vec3(0.0, 1.0, 0.0);

        let fov = self.fov.to_radians();
        let height = self.focus_distance * (fov / 2.0).tan();
        let width = height * aspect_ratio;

        let start = self.position - width * u + height * v - self.focus_distance * w;

        shader_type::Camera {
            position: self.position,
            horizontal: u,
            vertical: v,

            start,
            vx: (2.0 * width * u),
            vy: (-2.0 * height * v),

            lens_radius: self.aperture / 2.0,
        }
    }
}

impl Default for CameraParam {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(13.0, 2.0, 3.0),
            yaw: 80.0,
            pitch: -5.0,
            fov: 20.0,
            aperture: 0.1,
            focus_distance: 10.0,
        }
    }
}

fn create_bind_group(
    device: &wgpu::Device,
    buffers: &[&dyn buffer::Layout],
    visibility: wgpu::ShaderStages,
    label: Option<&str>,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let bind_group_layout_label = label.map(|l| format!("{l} Bind Group Layout"));
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| buffer.layout(i as u32, visibility))
            .collect::<Vec<_>>(),
        label: bind_group_layout_label.as_deref(),
    });

    let bind_group_label = label.map(|l| format!("{l} Bind Group"));
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| buffer.binding(i as u32))
            .collect::<Vec<_>>(),
        label: bind_group_label.as_deref(),
    });

    (bind_group_layout, bind_group)
}
