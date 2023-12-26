use crate::prelude::*;
use ::image::ColorType;
use image::GenericImageView;

/// Information describing the binary data of an image.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ImageInfo {
    /// The pixel format of the image data.
    pub pixel_format: PixelFormat,

    /// The size of the image in pixels
    pub size: ext::glam::UVec2,

    /// The stride of the image data in bytes for both X and Y.
    pub stride: ext::glam::UVec2,
}

/// Supported pixel formats.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(unused)]
pub enum PixelFormat {
    /// 8-bit monochrome data.
    Mono8,

    /// 8-bit monochrome data with alpha.
    MonoAlpha8(Alpha),

    /// Interlaced 8-bit BGR data.
    Bgr8,

    /// Interlaced 8-bit BGRA data.
    Bgra8(Alpha),

    /// Interlaced 8-bit RGB data.
    Rgb8,

    /// Interlaced 8-bit RGBA data.
    Rgba8(Alpha),
}

impl From<ColorType> for PixelFormat {
    fn from(value: ColorType) -> Self {
        match value {
            ColorType::L8 => Self::Mono8,
            ColorType::La8 => Self::MonoAlpha8(Alpha::Premultiplied),
            ColorType::Rgb8 => Self::Rgb8,
            ColorType::Rgba8 => Self::Bgra8(Alpha::Premultiplied),
            ColorType::L16 => unimplemented!(),
            ColorType::La16 => unimplemented!(),
            ColorType::Rgb16 => unimplemented!(),
            ColorType::Rgba16 => unimplemented!(),
            ColorType::Rgb32F => unimplemented!(),
            ColorType::Rgba32F => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}

/// Possible alpha representations.
///
/// See also: <https://en.wikipedia.org/wiki/Alpha_compositing#Straight_versus_premultiplied>
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(unused)]
pub enum Alpha {
    /// The alpha channel is encoded only in the alpha component of the pixel.
    Unpremultiplied,

    /// The alpha channel is also premultiplied into the other components of the pixel.
    Premultiplied,
}

impl ImageInfo {
    /// Create a new info struct with the given format, width and height.
    ///
    /// The row stride is automatically calculated based on the image width and pixel format.
    /// If you wish to use a different row stride, construct the struct directly.
    pub fn new(pixel_format: PixelFormat, width: u32, height: u32) -> Self {
        let stride_x = u32::from(pixel_format.bytes_per_pixel());
        let stride_y = stride_x * width;
        Self {
            pixel_format,
            size: ext::glam::UVec2::new(width, height),
            stride: ext::glam::UVec2::new(stride_x, stride_y),
        }
    }
}

impl PixelFormat {
    /// Get the number of channels.
    pub fn channels(self) -> u8 {
        match self {
            PixelFormat::Mono8 => 1,
            PixelFormat::MonoAlpha8(_) => 1,
            PixelFormat::Bgr8 => 3,
            PixelFormat::Bgra8(_) => 4,
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8(_) => 4,
        }
    }

    /// Get the bytes per channel.
    const fn byte_depth(self) -> u8 {
        1
    }

    /// Get the bytes per pixel.
    pub fn bytes_per_pixel(self) -> u8 {
        self.byte_depth() * self.channels()
    }
}

/// Trait for borrowing image data from a struct.
pub trait AsImageView {
    type Err;

    /// Get an image view for the object.
    fn as_image_view(&self) -> Result<ImageView, Self::Err>;
}

/// Borrowed view of image data,
#[derive(Debug, Copy, Clone)]
pub struct ImageView<'a> {
    info: ImageInfo,
    data: &'a [u8],
}

impl<'a> ImageView<'a> {
    /// Create a new image view from image information and a data slice.
    pub fn new(info: ImageInfo, data: &'a [u8]) -> Self {
        Self { info, data }
    }

    /// Get the image information.
    pub fn info(&self) -> ImageInfo {
        self.info
    }

    /// Get the image data as byte slice.
    pub fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> AsImageView for ImageView<'a> {
    type Err = ();

    fn as_image_view(&self) -> Result<ImageView, Self::Err> {
        Ok(*self)
    }
}

/// A GPU image buffer ready to be used with the rendering pipeline.
#[derive(Debug)]
#[allow(dead_code)] // TODO: look at this bc i think this can be removed
pub struct GpuImage {
    name: String,
    info: ImageInfo,
    bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    data: wgpu::Buffer,
}

/// The uniforms associated with a [`GpuImage`].
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GpuImageUniforms {
    format: u32,
    width: u32,
    height: u32,
    stride_x: u32,
    stride_y: u32,
}
unsafe impl crate::render::uniforms::Std140 for GpuImageUniforms {}

impl GpuImage {
    /// Create a [`GpuImage`] from an image buffer.
    pub fn from_data(
        name: String,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        image: &ImageView,
    ) -> Self {
        let info = image.info();

        let format = match info.pixel_format {
            PixelFormat::Mono8 => 0,
            PixelFormat::MonoAlpha8(Alpha::Unpremultiplied) => 1,
            PixelFormat::MonoAlpha8(Alpha::Premultiplied) => 2,
            PixelFormat::Bgr8 => 3,
            PixelFormat::Bgra8(Alpha::Unpremultiplied) => 4,
            PixelFormat::Bgra8(Alpha::Premultiplied) => 5,
            PixelFormat::Rgb8 => 6,
            PixelFormat::Rgba8(Alpha::Unpremultiplied) => 7,
            PixelFormat::Rgba8(Alpha::Premultiplied) => 8,
        };

        let uniforms = GpuImageUniforms {
            format,
            width: info.size.x,
            height: info.size.y,
            stride_x: info.stride.x,
            stride_y: info.stride.y,
        };

        let uniforms = super::buffer::with_value(
            device,
            Some(&format!("{}_uniforms_buffer", name)),
            &uniforms,
            wgpu::BufferUsages::UNIFORM,
        );

        use wgpu::util::DeviceExt;

        let data = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_image_buffer", name)),
            contents: image.data(),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", name)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniforms,
                        offset: 0,
                        size: None, // Use entire buffer.
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &data,
                        offset: 0,
                        size: None, // Use entire buffer.
                    }),
                },
            ],
        });

        Self {
            name,
            info,
            bind_group,
            uniforms,
            data,
        }
    }

    /// Get the image info.
    pub fn info(&self) -> &ImageInfo {
        &self.info
    }

    /// Get the bind group that should be used to render the image with the rendering pipeline.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
