use crate::{
    gamma_u8_from_linear_f32, linear_f32_from_gamma_u8, linear_f32_from_linear_u8,
    linear_u8_from_linear_f32, Color32, Rgba,
};

/// Represents a color in the HSV (Hue, Saturation, Value) color space, including alpha.
/// All values are in the range [0, 1]. Alpha is not premultiplied.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsva {
    /// Hue component (0 to 1), representing the color type.
    pub h: f32,

    /// Saturation component (0 to 1), representing the color intensity.
    pub s: f32,

    /// Value component (0 to 1), representing the brightness of the color.
    pub v: f32,

    /// Alpha component (0 to 1), representing the opacity of the color.
    /// A negative value signifies an additive color, and alpha is ignored.
    pub a: f32,
}

impl Hsva {
    /// Creates a new Hsva instance with specified hue, saturation, value, and alpha.
    #[inline]
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self { h, s, v, a }
    }

    /// Converts from `sRGBA` with premultiplied alpha to `Hsva`.
    #[inline]
    pub fn from_srgba_premultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from_rgba_premultiplied(
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
            linear_f32_from_linear_u8(a),
        )
    }

    /// Converts from `sRGBA` without premultiplied alpha to `Hsva`.
    #[inline]
    pub fn from_srgba_unmultiplied([r, g, b, a]: [u8; 4]) -> Self {
        Self::from_rgba_unmultiplied(
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
            linear_f32_from_linear_u8(a),
        )
    }

    /// Converts from linear RGBA with premultiplied alpha to `Hsva`.
    #[inline]
    pub fn from_rgba_premultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        #![allow(clippy::many_single_char_names)]
        if a == 0.0 {
            if r == 0.0 && b == 0.0 && a == 0.0 {
                Self::default()  // Return default value if the color is transparent.
            } else {
                Self::from_additive_rgb([r, g, b])  // Handle additive color.
            }
        } else {
            let (h, s, v) = hsv_from_rgb([r / a, g / a, b / a]);  // Compute HSV values considering alpha.
            Self { h, s, v, a }
        }
    }

    /// Converts from linear RGBA without premultiplied alpha to `Hsva`.
    #[inline]
    pub fn from_rgba_unmultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        #![allow(clippy::many_single_char_names)]
        let (h, s, v) = hsv_from_rgb([r, g, b]);  // Compute HSV values.
        Self { h, s, v, a }
    }

    /// Converts from additive RGB to `Hsva`.
    #[inline]
    pub fn from_additive_rgb(rgb: [f32; 3]) -> Self {
        let (h, s, v) = hsv_from_rgb(rgb);
        Self {
            h,
            s,
            v,
            a: -0.5, // Negative alpha signifies additive color.
        }
    }

    /// Converts from additive sRGB to `Hsva`.
    #[inline]
    pub fn from_additive_srgb([r, g, b]: [u8; 3]) -> Self {
        Self::from_additive_rgb([
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
        ])
    }

    /// Converts from RGB values to `Hsva` with alpha set to 1.0.
    #[inline]
    pub fn from_rgb(rgb: [f32; 3]) -> Self {
        let (h, s, v) = hsv_from_rgb(rgb);
        Self { h, s, v, a: 1.0 }
    }

    /// Converts from sRGB to `Hsva` with alpha set to 1.0.
    #[inline]
    pub fn from_srgb([r, g, b]: [u8; 3]) -> Self {
        Self::from_rgb([
            linear_f32_from_gamma_u8(r),
            linear_f32_from_gamma_u8(g),
            linear_f32_from_gamma_u8(b),
        ])
    }

    // ------------------------------------------------------------------------

    /// Converts the `Hsva` instance to opaque by setting alpha to 1.0.
    #[inline]
    pub fn to_opaque(self) -> Self {
        Self { a: 1.0, ..self }
    }

    /// Converts the `Hsva` instance to linear RGB values.
    #[inline]
    pub fn to_rgb(&self) -> [f32; 3] {
        rgb_from_hsv((self.h, self.s, self.v))
    }

    /// Converts the `Hsva` instance to gamma-corrected sRGB values.
    #[inline]
    pub fn to_srgb(&self) -> [u8; 3] {
        let [r, g, b] = self.to_rgb();
        [
            gamma_u8_from_linear_f32(r),
            gamma_u8_from_linear_f32(g),
            gamma_u8_from_linear_f32(b),
        ]
    }

    /// Converts the `Hsva` instance to linear RGBA with premultiplied alpha.
    #[inline]
    pub fn to_rgba_premultiplied(&self) -> [f32; 4] {
        let [r, g, b, a] = self.to_rgba_unmultiplied();
        let additive = a < 0.0;
        if additive {
            [r, g, b, 0.0]  // For additive colors, alpha is set to 0.
        } else {
            [a * r, a * g, a * b, a]  // Apply alpha to RGB channels.
        }
    }

    /// Converts the `Hsva` instance to linear RGBA without premultiplied alpha.
    #[inline]
    pub fn to_rgba_unmultiplied(&self) -> [f32; 4] {
        let Self { h, s, v, a } = *self;
        let [r, g, b] = rgb_from_hsv((h, s, v));
        [r, g, b, a]
    }

    /// Converts the `Hsva` instance to gamma-corrected sRGBA with premultiplied alpha.
    #[inline]
    pub fn to_srgba_premultiplied(&self) -> [u8; 4] {
        let [r, g, b, a] = self.to_rgba_premultiplied();
        [
            gamma_u8_from_linear_f32(r),
            gamma_u8_from_linear_f32(g),
            gamma_u8_from_linear_f32(b),
            linear_u8_from_linear_f32(a),
        ]
    }

    /// Converts the `Hsva` instance to gamma-corrected sRGBA without premultiplied alpha.
    #[inline]
    pub fn to_srgba_unmultiplied(&self) -> [u8; 4] {
        let [r, g, b, a] = self.to_rgba_unmultiplied();
        [
            gamma_u8_from_linear_f32(r),
            gamma_u8_from_linear_f32(g),
            gamma_u8_from_linear_f32(b),
            linear_u8_from_linear_f32(a.abs()),
        ]
    }
}

impl From<Hsva> for Rgba {
    /// Converts an `Hsva` instance to an `Rgba` instance.
    #[inline]
    fn from(hsva: Hsva) -> Self {
        Self(hsva.to_rgba_premultiplied())
    }
}

impl From<Rgba> for Hsva {
    /// Converts an `Rgba` instance to an `Hsva` instance.
    #[inline]
    fn from(rgba: Rgba) -> Self {
        Self::from_rgba_premultiplied(rgba.0[0], rgba.0[1], rgba.0[2], rgba.0[3])
    }
}

impl From<Hsva> for Color32 {
    /// Converts an `Hsva` instance to a `Color32` instance.
    #[inline]
    fn from(hsva: Hsva) -> Self {
        Self::from(Rgba::from(hsva))
    }
}

impl From<Color32> for Hsva {
    /// Converts a `Color32` instance to an `Hsva` instance.
    #[inline]
    fn from(srgba: Color32) -> Self {
        Self::from(Rgba::from(srgba))
    }
}

/// Converts RGB values to HSV color space.
///
/// All ranges are in [0, 1], and RGB is in linear space.
#[inline]
pub fn hsv_from_rgb([r, g, b]: [f32; 3]) -> (f32, f32, f32) {
    #![allow(clippy::many_single_char_names)]
    let min = r.min(g.min(b));  // Minimum of RGB components.
    let max = r.max(g.max(b));  // Maximum of RGB components (value).

    let range = max - min;

    let h = if max == min {
        0.0 // Hue is undefined when max equals min.
    } else if max == r {
        (g - b) / (6.0 * range)
    } else if max == g {
        (b - r) / (6.0 * range) + 1.0 / 3.0
    } else {
        // max == b
        (r - g) / (6.0 * range) + 2.0 / 3.0
    };
    let h = (h + 1.0).fract(); // Wrap hue to [0, 1].
    let s = if max == 0.0 { 0.0 } else { 1.0 - min / max }; // Saturation.
    (h, s, max) // Return hue, saturation, and value.
}

/// Converts HSV values to RGB color space.
///
/// All ranges are in [0, 1], and RGB values are in linear space.
#[inline]
pub fn rgb_from_hsv((h, s, v): (f32, f32, f32)) -> [f32; 3] {
    #![allow(clippy::many_single_char_names)]
    let h = (h.fract() + 1.0).fract(); // Wrap hue to [0, 1].
    let s = s.clamp(0.0, 1.0); // Clamp saturation.

    let f = h * 6.0 - (h * 6.0).floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match (h * 6.0).floor() as i32 % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => unreachable!(), // Unreachable case due to previous calculations.
    }
}

#[test]
#[ignore] // A bit expensive to run.
fn test_hsv_roundtrip() {
    for r in 0..=255 {
        for g in 0..=255 {
            for b in 0..=255 {
                let srgba = Color32::from_rgb(r, g, b);
                let hsva = Hsva::from(srgba);
                assert_eq!(srgba, Color32::from(hsva)); // Check round-trip conversion.
            }
        }
    }
}