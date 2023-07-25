use crate::gpu::{GpuImage, ToStd140, UniformsBuffer};
use glam::Vec3;
use glam::{Affine2, Vec2};
use serde::{Deserialize, Serialize};
use wgpu::Color;
use winit::window::WindowId;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct WindowIdent {
    // pub name: Option<String>,
    pub index: usize,
}

impl WindowIdent {
    pub fn new(
        // name: Option<String>,
        index: usize,
    ) -> Self {
        Self {
            //name,
            index,
        }
    }

    pub fn any() -> Self {
        Self { index: 0 }
    }
}

/// Window capable of displaying images using wgpu.
pub struct Window {
    /// The winit window.
    pub window: winit::window::Window,

    /// If true, preserve the aspect ratio of images.
    pub preserve_aspect_ratio: bool,

    /// The background color of the window.
    pub background_color: Color,

    /// The wgpu surface to render to.
    pub surface: wgpu::Surface,

    /// The window specific uniforms for the render pipeline.
    pub uniforms: UniformsBuffer<WindowUniforms>,

    /// The image to display (if any).
    pub image: Option<GpuImage>,

    /// Transformation to apply to the image, in virtual window space.
    ///
    /// Virtual window space goes from (0, 0) in the top left to (1, 1) in the bottom right.
    pub user_transform: Affine2,
    // The event handlers for this specific window.
    // pub event_handlers: Vec<Box<DynWindowEventHandler>>,
}

impl Window {
    /// Get the window ID.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Recalculate the uniforms for the render pipeline from the window state.
    pub fn calculate_uniforms(&self) -> WindowUniforms {
        if let Some(image) = &self.image {
            let image_size = image.info().size.as_vec2();
            if !self.preserve_aspect_ratio {
                WindowUniforms::stretch(image_size).pre_apply_transform(self.user_transform)
            } else {
                let window_size = glam::UVec2::new(
                    self.window.inner_size().width,
                    self.window.inner_size().height,
                )
                .as_vec2();
                WindowUniforms::fit(window_size, image_size)
                    .pre_apply_transform(self.user_transform)
            }
        } else {
            WindowUniforms {
                transform: self.user_transform,
                image_size: Vec2::new(0.0, 0.0),
            }
        }
    }
}

/// Options for creating a new window.
#[derive(Debug, Clone)]
pub struct WindowOptions {
    /// Preserve the aspect ratio of the image when scaling.
    pub preserve_aspect_ratio: bool,

    /// The background color for the window.
    ///
    /// This is used to color areas without image data if `preserve_aspect_ratio` is true.
    pub background_color: Color,

    /// Create the window hidden.
    ///
    /// The window can manually be made visible at a later time.
    pub start_hidden: bool,

    /// The initial size of the window in pixel.
    pub size: Option<[u32; 2]>,

    /// If true allow the window to be resized.
    pub resizable: bool,

    /// Make the window borderless.
    pub borderless: bool,

    /// Make the window fullscreen.
    pub fullscreen: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowOptions {
    /// Create new window options with default values.
    pub fn new() -> Self {
        Self {
            preserve_aspect_ratio: true,
            background_color: Color::BLACK,
            start_hidden: false,
            size: None,
            resizable: true,
            borderless: false,
            fullscreen: false,
        }
    }
}

/// The window specific uniforms for the render pipeline.
#[derive(Debug, Copy, Clone)]
pub struct WindowUniforms {
    /// The transformation applied to the image.
    ///
    /// With the identity transform, the image is stretched to the inner window size,
    /// without preserving the aspect ratio.
    pub transform: Affine2,

    /// The size of the image in pixels.
    pub image_size: Vec2,
}

impl WindowUniforms {
    pub fn no_image() -> Self {
        Self::stretch(Vec2::new(0.0, 0.0))
    }

    pub fn stretch(image_size: Vec2) -> Self {
        Self {
            transform: Affine2::IDENTITY,
            image_size,
        }
    }

    pub fn fit(window_size: Vec2, image_size: Vec2) -> Self {
        let ratios = image_size / window_size;

        let w;
        let h;
        if ratios.x >= ratios.y {
            w = 1.0;
            h = ratios.y / ratios.x;
        } else {
            w = ratios.x / ratios.y;
            h = 1.0;
        }

        let transform = Affine2::from_scale_angle_translation(
            Vec2::new(w, h),
            0.0,
            0.5 * Vec2::new(1.0 - w, 1.0 - h),
        );
        Self {
            transform,
            image_size,
        }
    }

    /// Pre-apply a transformation.
    pub fn pre_apply_transform(mut self, transform: Affine2) -> Self {
        self.transform = transform * self.transform;
        self
    }
}

#[repr(C, align(8))]
#[derive(Debug, Copy, Clone)]
struct Vec2A8 {
    pub x: f32,
    pub y: f32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
struct Vec3A16 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Mat3x3 {
    pub cols: [Vec3A16; 3],
}

impl Vec2A8 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vec3A16 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Mat3x3 {
    pub const fn new(col0: Vec3A16, col1: Vec3A16, col2: Vec3A16) -> Self {
        Self {
            cols: [col0, col1, col2],
        }
    }
}

impl From<Vec2> for Vec2A8 {
    fn from(other: Vec2) -> Self {
        Self::new(other.x, other.y)
    }
}

impl From<Vec3> for Vec3A16 {
    fn from(other: Vec3) -> Self {
        Self::new(other.x, other.y, other.z)
    }
}

impl From<Affine2> for Mat3x3 {
    fn from(other: Affine2) -> Self {
        let x_axis = other.matrix2.x_axis;
        let y_axis = other.matrix2.y_axis;
        let z_axis = other.translation;
        Self::new(
            Vec3A16::new(x_axis.x, x_axis.y, 0.0),
            Vec3A16::new(y_axis.x, y_axis.y, 0.0),
            Vec3A16::new(z_axis.x, z_axis.y, 1.0),
        )
    }
}

/// Window specific unfiforms, layout compatible with glsl std140.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WindowUniformsStd140 {
    image_size: Vec2A8,
    transform: Mat3x3,
}

unsafe impl ToStd140 for WindowUniforms {
    type Output = WindowUniformsStd140;

    fn to_std140(&self) -> Self::Output {
        Self::Output {
            image_size: self.image_size.into(),
            transform: self.transform.into(),
        }
    }
}
