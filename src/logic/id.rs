use ext::parse::MoveIt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SurfaceId {
    Terminal,
    Window(u64),
}

impl SurfaceId {
    pub fn as_id(&self) -> Option<u64> {
        match self {
            SurfaceId::Terminal => None,
            SurfaceId::Window(id) => Some(*id),
        }
    }
}

impl From<winit::window::WindowId> for SurfaceId {
    #[inline]
    fn from(value: winit::window::WindowId) -> Self {
        value.move_it(u64::from).into()
    }
}

impl From<u64> for SurfaceId {
    #[inline]
    fn from(value: u64) -> Self {
        Self::Window(value)
    }
}
