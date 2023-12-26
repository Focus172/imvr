use image::GenericImageView;

use crate::prelude::*;

pub struct RawImage {
    pub color: image::ColorType,
    pub size: (u32, u32),
    pub data: Box<[u8]>,
}

impl fmt::Debug for RawImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawImage")
            .field("color", &self.color)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

// impl fmt::Debug for RawImage {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "RawImage {{ .. }}")
//     }
// }

impl From<image::DynamicImage> for RawImage {
    fn from(value: image::DynamicImage) -> Self {
        let color = value.color();
        let size = value.dimensions();
        let data = value.into_rgb8().into_vec().into_boxed_slice();
        RawImage { color, size, data }
    }
}
