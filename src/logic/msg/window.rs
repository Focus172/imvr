use image::DynamicImage;

use super::{Msg, ReturnAddress};

/// A message that closely resemblems the final Requested.
/// Must be [`Send`] and relativly small on the stack
pub enum WindowMsg {
    ShowImage { image: DynamicImage, id: u64 },
    OpenWindow { resp: ReturnAddress },
}

impl std::fmt::Debug for WindowMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShowImage { id, .. } => f
                .debug_struct("ShowImage")
                .field("image", &"{ .. }")
                .field("id", id)
                .finish(),
            Self::OpenWindow { resp } => f.debug_struct("OpenWindow").field("resp", resp).finish(),
        }
    }
}

impl Msg {
    pub fn as_window(&mut self) -> Option<WindowMsg> {
        match self {
            Msg::ShowImage { path, id } => {
                let id = id.as_id()?;
                let image = image::open(path).unwrap();
                Some(WindowMsg::ShowImage { image, id })
            }
            Msg::OpenWindow { resp } => {
                let resp = resp.take()?;
                Some(WindowMsg::OpenWindow { resp })
            }
        }
    }
}
