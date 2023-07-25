use std::{
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use image::GenericImageView;

use crate::{
    gpu::{GpuContext, GpuImage},
    image_info::{ImageInfo, ImageView},
};

pub struct ImagePrebuffer {
    path_rx: Receiver<PrebufferMessage>,
    img_tx: Sender<GpuImage>,
    gpu: Option<Arc<GpuContext>>,
}

pub enum PrebufferMessage {
    InitGpu(Arc<GpuContext>),
    LoadPath(PathBuf),
    Exit,
}

impl ImagePrebuffer {
    pub fn new() -> (Self, (Sender<PrebufferMessage>, Receiver<GpuImage>)) {
        let (img_tx, img_rx) = std::sync::mpsc::channel::<GpuImage>();
        let (path_tx, path_rx) = std::sync::mpsc::channel::<PrebufferMessage>();

        (
            Self {
                path_rx,
                img_tx,
                gpu: None,
            },
            (path_tx, img_rx),
        )
    }

    /// Starts the task that will buffer one image ahead
    /// This never terminates (unless the channel is closed) so
    pub fn run(&mut self) -> anyhow::Result<()> {
        for msg in &self.path_rx {
            let path = match msg {
                PrebufferMessage::InitGpu(g) => {
                    if self.gpu.is_some() {
                        panic!("This class does not support reinitalizing the gpu context");
                    }
                    self.gpu = Some(g);
                    continue;
                }
                PrebufferMessage::LoadPath(p) => p,
                PrebufferMessage::Exit => break,
            };

            let img = image::open(path).unwrap();

            let (w, h) = img.dimensions();

            // let ctype = img.color();
            let color_type = img.color();

            log::warn!("Color type is: {:?}", color_type);

            let buf: Vec<u8> = img.into_bytes();

            let image = match color_type {
                image::ColorType::L8 => todo!(),
                image::ColorType::La8 => todo!(),
                image::ColorType::Rgb8 => {
                    let info = ImageInfo::rgb8(w, h);
                    ImageView::new(info, &buf)
                }
                image::ColorType::Rgba8 => {
                    let info = ImageInfo::rgba8(w, h);
                    ImageView::new(info, &buf)
                }
                image::ColorType::L16 => todo!(),
                image::ColorType::La16 => todo!(),
                image::ColorType::Rgb16 => todo!(),
                image::ColorType::Rgba16 => todo!(),
                image::ColorType::Rgb32F => todo!(),
                image::ColorType::Rgba32F => todo!(),
                _ => todo!(),
            };

            let gpu_im = GpuImage::from_data(
                "basic_dumb_name".into(),
                &self.gpu.as_ref().unwrap().device,
                &self.gpu.as_ref().unwrap().image_bind_group_layout,
                &image,
            );

            _ = self.img_tx.send(gpu_im);
        }

        log::info!("Read ahead buffer sucessfully exited");

        Ok(())
    }
}
