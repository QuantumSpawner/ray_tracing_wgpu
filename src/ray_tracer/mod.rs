mod camera;
mod resource;

use std::{collections::HashMap, mem::size_of};

use crate::wgpu;

const WORKGROUP_SIZE_X: u32 = 16;
const WORKGROUP_SIZE_Y: u32 = 16;

const MAX_WINDOW_SIZE_X: u32 = 1920;
const MAX_WINDOW_SIZE_Y: u32 = 1080;

#[derive(Debug, Clone, PartialEq)]
pub struct Sampling {
    pub max_samples: u32,
    pub num_bounces: u32,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            max_samples: 256,
            num_bounces: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub sampling: Sampling,
    pub display_size: cgmath::Vector2<u32>,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            sampling: Sampling::default(),
            display_size: cgmath::Vector2::new(1, 1),
        }
    }
}

impl Into<resource::Param> for Param {
    fn into(self) -> resource::Param {
        resource::Param {
            display_size: self.display_size.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Stat {
    pub frame_counter: u32,
}

impl Into<resource::Stat> for Stat {
    fn into(self) -> resource::Stat {
        resource::Stat {
            frame_counter: self.frame_counter,
        }
    }
}

pub struct RayTracer {
    param: Param,
    stat: Stat,

    param_uniform: resource::UniformBuffer<resource::Param>,
    stat_uniform: resource::UniformBuffer<resource::Stat>,
    _frame_buffer: resource::StorageBuffer,

    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,

    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
}

impl RayTracer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target: wgpu::ColorTargetState,
    ) -> Self {
        let param = Param::default();
        let stat = Stat::default();

        /* resource-----------------------------------------------------------*/
        let param_uniform = resource::UniformBuffer::new_with_data(
            device,
            param.clone().into(),
            Some("Ray Tracer Parameter"),
        );
        let stat_uniform = resource::UniformBuffer::new_with_data(
            device,
            stat.clone().into(),
            Some("Ray Tracer State"),
        );
        let frame_buffer = resource::StorageBuffer::new(
            device,
            MAX_WINDOW_SIZE_X as usize * MAX_WINDOW_SIZE_Y as usize * 4 * size_of::<f32>(),
            Some("Ray Tracer Frame Buffer"),
        );

        /* render shader------------------------------------------------------*/
        let render_shader_source = [
            include_str!("shader/resource.wgsl"),
            include_str!("shader/render.wgsl"),
        ]
        .join("\n");
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracer Render Shader"),
            source: wgpu::ShaderSource::Wgsl(render_shader_source.into()),
        });

        let (render_bind_group_layout, render_bind_group) = create_bind_group(
            device,
            &[&param_uniform, &stat_uniform, &frame_buffer],
            wgpu::ShaderStages::FRAGMENT,
            Some("Ray Tracer Render"),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracer Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
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
            include_str!("shader/resource.wgsl"),
            include_str!("shader/rng.wgsl"),
            include_str!("shader/compute.wgsl"),
        ]
        .join("\n");
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracer Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(compute_shader_source.into()),
        });

        let (compute_bind_group_layout, compute_bind_group) = create_bind_group(
            device,
            &[&param_uniform, &stat_uniform, &frame_buffer],
            wgpu::ShaderStages::COMPUTE,
            Some("Ray Tracer Comupte"),
        );

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracer Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
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
            param,
            stat,

            param_uniform,
            stat_uniform,
            _frame_buffer: frame_buffer,

            render_pipeline,
            render_bind_group,

            compute_pipeline,
            compute_bind_group,
        }
    }

    pub fn get_params(&self) -> &Param {
        &self.param
    }

    pub fn set_params(&mut self, queue: &wgpu::Queue, param: Param) {
        if self.param == param {
            return;
        }

        self.param_uniform.set_data(queue, param.clone().into());
        self.param = param;

        self.reset();
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.stat.frame_counter >= self.param.sampling.max_samples {
            return;
        }

        self.stat_uniform.set_data(queue, self.stat.clone().into());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Ray Tracer Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracer Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.param.display_size.x / WORKGROUP_SIZE_X + 1,
                self.param.display_size.y / WORKGROUP_SIZE_Y + 1,
                1,
            );
        }

        queue.submit(Some(encoder.finish()));

        self.stat.frame_counter += 1;
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn reset(&mut self) {
        self.stat.frame_counter = 0;
    }
}

fn create_bind_group(
    device: &wgpu::Device,
    buffers: &[&dyn resource::Layout],
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
