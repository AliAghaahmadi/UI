use crate::{gamma_from_linear, linear_from_gamma, Color32, Hsva, Rgba};

/// Represents a color in the HSV (Hue, Saturation, Value) color space with gamma-corrected brightness.
/// All values are in the range [0, 1]. Alpha is not premultiplied.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsvaGamma {
    /// Hue component (0 to 1), representing the color type.
    pub h: f32,

    /// Saturation component (0 to 1), representing the color intensity.
    pub s: f32,

    /// Value component (0 to 1), representing the brightness of the color, gamma-corrected for perceptual evenness.
    pub v: f32,

    /// Alpha component (0 to 1), representing the opacity of the color.
    /// A negative value signifies an additive color, and alpha is ignored.
    pub a: f32,
}

impl From<HsvaGamma> for Rgba {
    /// Converts an `HsvaGamma` instance to an `Rgba` instance.
    #[inline]
    fn from(hsvag: HsvaGamma) -> Self {
        Hsva::from(hsvag).into()
    }
}

impl From<HsvaGamma> for Color32 {
    /// Converts an `HsvaGamma` instance to a `Color32` instance.
    #[inline]
    fn from(hsvag: HsvaGamma) -> Self {
        Rgba::from(hsvag).into()
    }
}

impl From<HsvaGamma> for Hsva {
    /// Converts an `HsvaGamma` instance to an `Hsva` instance, transforming the gamma-corrected `v` value to linear space.
    #[inline]
    fn from(hsvag: HsvaGamma) -> Self {
        let HsvaGamma { h, s, v, a } = hsvag;
        Self {
            h,
            s,
            v: linear_from_gamma(v), // Convert gamma-corrected brightness to linear space.
            a,
        }
    }
}

impl From<Rgba> for HsvaGamma {
    /// Converts an `Rgba` instance to an `HsvaGamma` instance.
    #[inline]
    fn from(rgba: Rgba) -> Self {
        Hsva::from(rgba).into()
    }
}

impl From<Color32> for HsvaGamma {
    /// Converts a `Color32` instance to an `HsvaGamma` instance.
    #[inline]
    fn from(srgba: Color32) -> Self {
        Hsva::from(srgba).into()
    }
}

impl From<Hsva> for HsvaGamma {
    /// Converts an `Hsva` instance to an `HsvaGamma` instance, transforming the linear `v` value to gamma-corrected space.
    #[inline]
    fn from(hsva: Hsva) -> Self {
        let Hsva { h, s, v, a } = hsva;
        Self {
            h,
            s,
            v: gamma_from_linear(v), // Convert linear brightness to gamma-corrected space.
            a,
        }
    }
}