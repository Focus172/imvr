use crate::prelude::*;

use crate::render::uniforms::{Std140, WindowUniforms};
// use crate::ImageInfo;
// use crate::ImageView;
use core::num::NonZeroU64;

use super::image::GpuImageUniforms;

#[derive(Debug)]
/// A (per window?) context to render the surface on the gpu
pub struct GpuContext {
    /// The wgpu device to use.
    pub device: wgpu::Device,

    /// The wgpu command queue to use.
    pub queue: wgpu::Queue,

    /// The bind group layout for the window specific bindings.
    pub window_bind_group_layout: wgpu::BindGroupLayout,

    /// The bind group layout for the image specific bindings.
    pub image_bind_group_layout: wgpu::BindGroupLayout,

    /// The render pipeline to use for windows.
    pub window_pipeline: wgpu::RenderPipeline,
}

impl GpuContext {
    pub fn new(
        instance: &wgpu::Instance,
        swap_chain_format: wgpu::TextureFormat,
        surface: &wgpu::Surface,
    ) -> Result<Self, GpuContextError> {
        let (device, queue) = futures::executor::block_on(get_device(instance, surface))?;
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Unhandled WGPU error: {}", error);
        }));

        let window_bind_group_layout = create_window_bind_group_layout(&device);
        let image_bind_group_layout = create_image_bind_group_layout(&device);

        let vertex_shader =
            device.create_shader_module(wgpu::include_spirv!("../../../shaders/shader.vert.spv"));
        let fragment_shader_unorm8 =
            device.create_shader_module(wgpu::include_spirv!("../../../shaders/unorm8.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("show-image-pipeline-layout"),
            bind_group_layouts: &[&window_bind_group_layout, &image_bind_group_layout],
            push_constant_ranges: &[],
        });

        let window_pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            &vertex_shader,
            &fragment_shader_unorm8,
            swap_chain_format,
        );

        Ok(Self {
            device,
            queue,
            window_bind_group_layout,
            image_bind_group_layout,
            window_pipeline,
        })
    }
}

/// Get a wgpu device to use.
async fn get_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface,
) -> Result<(wgpu::Device, wgpu::Queue), GpuContextError> {
    // Find a suitable display adapter.
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| {
            Report::new(GpuContextError).attach_printable("No suitable gpu adapter found")
        })?;

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("show-image"),
                limits: wgpu::Limits::default(),
                features: wgpu::Features::default(),
            },
            None,
        )
        .await
        .attach_printable_lazy(|| {
            format!("Failed to get a device that could show the image for adapter {adapter:?}.")
        })
        .change_context(GpuContextError)?;

    Ok((device, queue))
}

/// Create the bind group layout for the window specific bindings.
fn create_window_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("window_bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            count: None,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: const { Some(NonZeroU64::new(WindowUniforms::SIZE).unwrap()) },
            },
        }],
    })
}

/// Create the bind group layout for the image specific bindings.
fn create_image_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("image_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZeroU64::new(std::mem::size_of::<GpuImageUniforms>() as u64).unwrap(),
                    ),
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    })
}

/// Create a render pipeline with the specified device, layout, shaders and swap chain format.
fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("show-image-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

// Trait for data in Std140 compatible layout.
//
// # Safety
// Implementing this trait indicates that the data is in Std140 compatible layout.
// If that is not true, the GPU may perform illegal memory access.
// pub unsafe trait ToStd140 {
//     type Output: Copy;
//
//     const STD140_SIZE: u64 = std::mem::size_of::<Self::Output>() as u64;
//
//     fn to_std140(&self) -> Self::Output;
// }

/// A buffer holding uniform data and matching bind group.
///
/// The buffer can be marked as dirty to indicate the contents need to be updated.
/// The contents can be updated with [`Self::update_from`],
/// which will also clear the dirty flag.
#[derive(Debug)]
pub struct UniformsBuffer<T>
where
    T: Std140,
{
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    dirty: bool,
    _data: std::marker::PhantomData<T>,
}

impl<T: Std140> UniformsBuffer<T> {
    /// Create a new UniformsBuffer from the given value and bind group layout.
    ///
    /// The bind group layout must have exactly 1 binding for a buffer at index 0.
    pub fn from_value(device: &wgpu::Device, value: &T, layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = super::buffer::with_value(
            device,
            None,
            value,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms_bind_group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None, // Use entire buffer.
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            dirty: false,
            _data: std::marker::PhantomData,
        }
    }

    /// Get the bind group for the uniforms.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Check if the uniforms are marked as dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the uniforms as dirty.
    pub fn mark_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// Update the buffer contents using the provided command encoder and clear the dirty flag.
    pub fn update_from(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        value: &T,
    ) {
        let buffer = super::buffer::with_value(device, None, value, wgpu::BufferUsages::COPY_SRC);
        encoder.copy_buffer_to_buffer(&buffer, 0, &self.buffer, 0, T::SIZE as wgpu::BufferAddress);
        self.mark_dirty(false);
    }
}

#[derive(Debug)]
pub struct GpuContextError;

impl fmt::Display for GpuContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Gpu context encountered and error.")
    }
}

impl Context for GpuContextError {}
