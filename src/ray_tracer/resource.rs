use crate::wgpu;

use image::GenericImageView;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Param {
    pub display_size: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Stat {
    pub frame_counter: u32,
}

pub trait Layout {
    fn layout(&self, binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry;
    fn binding(&self, binding: u32) -> wgpu::BindGroupEntry<'_>;
}

pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    data: T,
}

impl<T: Default + bytemuck::Pod> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device, label: Option<&str>) -> Self {
        Self::new_with_data(device, T::default(), label)
    }
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub fn new_with_data(device: &wgpu::Device, data: T, label: Option<&str>) -> Self {
        let label = label.map(|l| format!("{l} Uniform Buffer"));
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: label.as_deref(),
        });

        Self { buffer, data }
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }

    pub fn set_data(&mut self, queue: &wgpu::Queue, data: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
        self.data = data;
    }
}

impl<T> Layout for UniformBuffer<T> {
    fn layout(&self, binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn binding(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
}

pub struct StorageBuffer {
    buffer: wgpu::Buffer,
}

impl StorageBuffer {
    pub fn new(device: &wgpu::Device, size: usize, label: Option<&str>) -> Self {
        let label = label.map(|l| format!("{l} Storage Buffer"));
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &vec![0u8; size],
            usage: wgpu::BufferUsages::STORAGE,
            label: label.as_deref(),
        });

        Self { buffer }
    }
}

impl Layout for StorageBuffer {
    fn layout(&self, binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn binding(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
}

pub struct Texture {
    size: wgpu::Extent3d,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        dimension: (u32, u32),
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: dimension.0,
            height: dimension.1,
            depth_or_array_layers: 1,
        };

        let label = label.map(|l| format!("{l} Texture"));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            size,
            texture,
            view,
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Self {
        let img = image::load_from_memory(bytes).unwrap();
        Self::from_image(device, queue, &img, label)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture = Self::new(
            device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            (dimensions.0, dimensions.1),
            label,
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture.size,
        );

        texture
    }
}

impl Layout for Texture {
    fn layout(&self, binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn binding(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}
