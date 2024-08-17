use super::*;
use cint::{Alpha, ColorInterop, EncodedSrgb, Hsv, LinearSrgb, PremultipliedAlpha};

// ---- Color32 Conversions ----

/// Converts an `Alpha<EncodedSrgb<u8>>` (which includes an alpha value and color components in gamma space) to a `Color32`.
impl From<Alpha<EncodedSrgb<u8>>> for Color32 {
    fn from(srgba: Alpha<EncodedSrgb<u8>>) -> Self {
        let Alpha {
            color: EncodedSrgb { r, g, b },
            alpha: a,
        } = srgba;

        Self::from_rgba_unmultiplied(r, g, b, a)
    }
}

// No conversion is implemented from `Color32` to `Alpha<_>` as `Color32` uses premultiplied alpha.

/// Converts a `PremultipliedAlpha<EncodedSrgb<u8>>` (which includes an alpha value and color components in gamma space) to a `Color32`.
impl From<PremultipliedAlpha<EncodedSrgb<u8>>> for Color32 {
    fn from(srgba: PremultipliedAlpha<EncodedSrgb<u8>>) -> Self {
        let PremultipliedAlpha {
            color: EncodedSrgb { r, g, b },
            alpha: a,
        } = srgba;

        Self::from_rgba_premultiplied(r, g, b, a)
    }
}

/// Converts a `Color32` to `PremultipliedAlpha<EncodedSrgb<u8>>`, which stores color components and alpha in gamma space.
impl From<Color32> for PremultipliedAlpha<EncodedSrgb<u8>> {
    fn from(col: Color32) -> Self {
        let (r, g, b, a) = col.to_tuple();

        Self {
            color: EncodedSrgb { r, g, b },
            alpha: a,
        }
    }
}

/// Converts a `PremultipliedAlpha<EncodedSrgb<f32>>` (which includes color and alpha components in linear space) to a `Color32`.
impl From<PremultipliedAlpha<EncodedSrgb<f32>>> for Color32 {
    fn from(srgba: PremultipliedAlpha<EncodedSrgb<f32>>) -> Self {
        let PremultipliedAlpha {
            color: EncodedSrgb { r, g, b },
            alpha: a,
        } = srgba;

        // Convert linear space values to gamma space values for use with `Color32`.
        let r = linear_u8_from_linear_f32(r);
        let g = linear_u8_from_linear_f32(g);
        let b = linear_u8_from_linear_f32(b);
        let a = linear_u8_from_linear_f32(a);

        Self::from_rgba_premultiplied(r, g, b, a)
    }
}

/// Converts a `Color32` to `PremultipliedAlpha<EncodedSrgb<f32>>`, which stores color and alpha components in linear space.
impl From<Color32> for PremultipliedAlpha<EncodedSrgb<f32>> {
    fn from(col: Color32) -> Self {
        let (r, g, b, a) = col.to_tuple();

        // Convert gamma space values to linear space values.
        let r = linear_f32_from_linear_u8(r);
        let g = linear_f32_from_linear_u8(g);
        let b = linear_f32_from_linear_u8(b);
        let a = linear_f32_from_linear_u8(a);

        Self {
            color: EncodedSrgb { r, g, b },
            alpha: a,
        }
    }
}

/// Defines the color interoperability type for `Color32` as `PremultipliedAlpha<EncodedSrgb<u8>>`.
impl ColorInterop for Color32 {
    type CintTy = PremultipliedAlpha<EncodedSrgb<u8>>;
}

// ---- Rgba Conversions ----

/// Converts a `PremultipliedAlpha<LinearSrgb<f32>>` (which includes color and alpha components in linear space) to an `Rgba`.
impl From<PremultipliedAlpha<LinearSrgb<f32>>> for Rgba {
    fn from(srgba: PremultipliedAlpha<LinearSrgb<f32>>) -> Self {
        let PremultipliedAlpha {
            color: LinearSrgb { r, g, b },
            alpha: a,
        } = srgba;

        Self([r, g, b, a])
    }
}

/// Converts an `Rgba` to `PremultipliedAlpha<LinearSrgb<f32>>`, which stores color and alpha components in linear space.
impl From<Rgba> for PremultipliedAlpha<LinearSrgb<f32>> {
    fn from(col: Rgba) -> Self {
        let (r, g, b, a) = col.to_tuple();

        Self {
            color: LinearSrgb { r, g, b },
            alpha: a,
        }
    }
}

/// Defines the color interoperability type for `Rgba` as `PremultipliedAlpha<LinearSrgb<f32>>`.
impl ColorInterop for Rgba {
    type CintTy = PremultipliedAlpha<LinearSrgb<f32>>;
}

// ---- Hsva Conversions ----

/// Converts an `Alpha<Hsv<f32>>` (which includes an alpha value and HSV color components) to an `Hsva`.
impl From<Alpha<Hsv<f32>>> for Hsva {
    fn from(srgba: Alpha<Hsv<f32>>) -> Self {
        let Alpha {
            color: Hsv { h, s, v },
            alpha: a,
        } = srgba;

        Self::new(h, s, v, a)
    }
}

/// Converts an `Hsva` (which includes HSV color components and alpha value) to `Alpha<Hsv<f32>>`.
impl From<Hsva> for Alpha<Hsv<f32>> {
    fn from(col: Hsva) -> Self {
        let Hsva { h, s, v, a } = col;

        Self {
            color: Hsv { h, s, v },
            alpha: a,
        }
    }
}

/// Defines the color interoperability type for `Hsva` as `Alpha<Hsv<f32>>`.
impl ColorInterop for Hsva {
    type CintTy = Alpha<Hsv<f32>>;
}

// ---- HsvaGamma Conversions ----

/// Defines the color interoperability type for `HsvaGamma` as `Alpha<Hsv<f32>>`.
impl ColorInterop for HsvaGamma {
    type CintTy = Alpha<Hsv<f32>>;
}

/// Converts an `Alpha<Hsv<f32>>` (which includes an alpha value and HSV color components) to an `HsvaGamma`.
impl From<Alpha<Hsv<f32>>> for HsvaGamma {
    fn from(srgba: Alpha<Hsv<f32>>) -> Self {
        let Alpha {
            color: Hsv { h, s, v },
            alpha: a,
        } = srgba;

        Hsva::new(h, s, v, a).into()
    }
}

/// Converts an `HsvaGamma` (which includes HSV color components and alpha value) to `Alpha<Hsv<f32>>`.
impl From<HsvaGamma> for Alpha<Hsv<f32>> {
    fn from(col: HsvaGamma) -> Self {
        let Hsva { h, s, v, a } = col.into();

        Self {
            color: Hsv { h, s, v },
            alpha: a,
        }
    }
}
