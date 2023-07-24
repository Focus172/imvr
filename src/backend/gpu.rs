use anyhow::anyhow;

use crate::backend::util::ToStd140;
use core::num::NonZeroU64;

use super::window::WindowUniforms;

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
    ) -> anyhow::Result<Self> {
        let (device, queue) = futures::executor::block_on(get_device(instance, surface))?;
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Unhandled WGPU error: {}", error);
        }));

        let window_bind_group_layout = create_window_bind_group_layout(&device);
        let image_bind_group_layout = create_image_bind_group_layout(&device);

        let vertex_shader =
            device.create_shader_module(wgpu::include_spirv!("../../shaders/shader.vert.spv"));
        let fragment_shader_unorm8 =
            device.create_shader_module(wgpu::include_spirv!("../../shaders/unorm8.frag.spv"));

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

fn select_power_preference() -> wgpu::PowerPreference {
    wgpu::PowerPreference::LowPower
}

/// Get a wgpu device to use.
async fn get_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface,
) -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
    // Find a suitable display adapter.
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: select_power_preference(),
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    });

    let adapter = adapter.await.ok_or(anyhow!("no adapter found"))?;

    // Create the logical device and command queue
    let device = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("show-image"),
            limits: wgpu::Limits::default(),
            features: wgpu::Features::default(),
        },
        None,
    );

    let (device, queue) = device.await?;

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
                min_binding_size: Some(NonZeroU64::new(WindowUniforms::STD140_SIZE).unwrap()),
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
                    min_binding_size:
                        Some(
                            NonZeroU64::new(
                                std::mem::size_of::<super::util::GpuImageUniforms>() as u64
                            )
                            .unwrap(),
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
