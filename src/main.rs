#![feature(exitcode_exit_method)]

mod backend;
mod background_thread;
mod error;
mod event;
mod image;
mod image_info;
mod oneshot;
mod rectangle;
mod color;

pub use self::backend::*;
pub use self::image::*;
pub use self::image_info::*;
pub use self::rectangle::Rectangle;

pub use winit::window::WindowId;
use std::time::Duration;
pub use self::color::Color;

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

