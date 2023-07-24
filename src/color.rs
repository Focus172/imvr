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
        Self {
            red,
            green,
            blue,
            alpha,
        }
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

impl From<Color> for wgpu::Color {
    fn from(other: Color) -> Self {
        Self {
            r: other.red,
            g: other.green,
            b: other.blue,
            a: other.alpha,
        }
    }
}
