use glam::{Vec3, Affine2, Vec2};

use crate::gpu::ToStd140;

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

/// Window specific uniforms, layout compatible with glsl std140.
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
