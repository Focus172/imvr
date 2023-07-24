use std::path::PathBuf;

pub enum Request {
    NextImage,
    OpenWindow,
    ShowImage(PathBuf),
    Exit,
}
