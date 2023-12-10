use std::mem;

use ext::glam::{Affine2, Vec2, Vec3};

/// Window specific uniforms, layout compatible with glsl std140.
/// Used in the render pipeline.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WindowUniforms {
    /// The transformation applied to the image.
    ///
    /// With the identity transform, the image is stretched to the inner window size,
    /// without preserving the aspect ratio.
    // transform: Affine2,
    transform: Mat3x3,

    /// The size of the image in pixels.
    // image_size: Vec2,
    size: Vec2A8,
}

unsafe impl Std140 for WindowUniforms {}

impl WindowUniforms {
    pub fn new(transform: Affine2, size: Vec2) -> Self {
        Self {
            transform: transform.into(),
            size: size.into(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new_stretched(Vec2::ZERO)
    }

    pub fn new_stretched(size: Vec2) -> Self {
        Self::new(Affine2::IDENTITY, size)
    }

    #[deprecated]
    pub fn get_size(&self) -> Vec2 {
        unsafe { mem::transmute(self.size) }
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
            transform: transform.into(),
            size: image_size.into(),
        }
    }

    // Pre-apply a transformation.
    // pub fn pre_apply_transform(mut self, transform: Affine2) -> Self {
    //     self.transform = transform * self.transform;
    //     self
    // }
}

/// A marker trait that shows a struct only uses 140 types
///
/// # Safety
/// Requires that the data conforms to the format in std 140
///
/// It is notablly safe to impliment this trait on a struct if
/// all its feilds impliment this trait and it is repr C.
pub unsafe trait Std140: Sized + Copy {
    const SIZE: u64 = mem::size_of::<Self>() as u64;
    fn bytes(&self) -> &[u8] {
        // # Saftey
        // the safety of this trait is what makes this call valid
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of_val(self),
            )
        }
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

impl From<[f32; 3]> for Vec3A16 {
    fn from(value: [f32; 3]) -> Self {
        Vec3A16 {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<[[f32; 3]; 3]> for Mat3x3 {
    fn from(value: [[f32; 3]; 3]) -> Self {
        Self {
            cols: value.map(|[x, y, z]| Vec3A16::new(x, y, z)),
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
        Self::from([
            [x_axis.x, x_axis.y, 0.0],
            [y_axis.x, y_axis.y, 0.0],
            [z_axis.x, z_axis.y, 1.0],
        ])
    }
}
