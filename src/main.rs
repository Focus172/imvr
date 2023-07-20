#![feature(exitcode_exit_method)]

use std::time::Duration;

fn main() {
    run_context(run);
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let img = image_out::open("/Users/evan/Pictures/anime/cat.jpg").unwrap();
    let mut buf = Vec::new();
    let h = img.height();
    let w = img.width();
    // let ctype = img.color();
    img.to_rgb8().pixels().for_each(|p| {
        p.0.iter().for_each(|v| {
            buf.push(*v)
        });
    });

    let image = ImageView::new(ImageInfo::rgb8(w, h), &buf);

    // Create a window with default options and display the image.
    let window = create_window("image", Default::default())?;
    window.set_image("image-001", image)?;

    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}

mod backend;
mod background_thread;
pub mod error;
pub mod event;
mod image;
mod image_info;
mod oneshot;
mod rectangle;

pub use self::backend::*;
pub use self::image::*;
pub use self::image_info::*;
pub use self::rectangle::Rectangle;

pub use winit;
pub use winit::window::WindowId;

pub use glam;

/// An RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    /// The red component in the range 0 to 1.
    pub red: f64,

    /// The green component in the range 0 to 1.
    pub green: f64,

    /// The blue component in the range 0 to 1.
    pub blue: f64,

    /// The alpha component in the range 0 to 1.
    pub alpha: f64,
}

impl Color {
    /// Create a new fully opaque color from the RGB components.
    pub const fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Create a new color from the RGBA components.
    pub const fn rgba(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Self { red, green, blue, alpha }
    }

    /// Get a color representing fully opaque black.
    pub const fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    /// Get a color representing fully opaque white.
    pub const fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }
}

