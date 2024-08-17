
//! Color conversions and types.
//!
//! This module provides functionality for converting between different color representations and manipulating colors.
//! Use [`Color32`] for a compact color representation.
//! Use [`Rgba`] if you need to work with RGBA colors directly.
//! Use [`HsvaGamma`] for manipulating colors in a way that is more intuitive for human perception of colors.
//!
//! ## Feature Flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::wrong_self_convention)]

#[cfg(feature = "cint")]
mod cint_impl;

mod color32;
pub use color32::*;

mod hsva_gamma;
pub use hsva_gamma::*;

mod hsva;
pub use hsva::*;

#[cfg(feature = "color-hex")]
mod hex_color_macro;
#[cfg(feature = "color-hex")]
#[doc(hidden)]
pub use color_hex;

mod rgba;
pub use rgba::*;

mod hex_color_runtime;
pub use hex_color_runtime::*;

// ----------------------------------------------------------------------------
// Color Conversion Implementations:

/// Converts a [`Color32`] to an [`Rgba`].
impl From<Color32> for Rgba {
    fn from(srgba: Color32) -> Self {
        Self([
            linear_f32_from_gamma_u8(srgba.0[0]),
            linear_f32_from_gamma_u8(srgba.0[1]),
            linear_f32_from_gamma_u8(srgba.0[2]),
            linear_f32_from_linear_u8(srgba.0[3]),
        ])
    }
}

/// Converts an [`Rgba`] to a [`Color32`].
impl From<Rgba> for Color32 {
    fn from(rgba: Rgba) -> Self {
        Self([
            gamma_u8_from_linear_f32(rgba.0[0]),
            gamma_u8_from_linear_f32(rgba.0[1]),
            gamma_u8_from_linear_f32(rgba.0[2]),
            linear_u8_from_linear_f32(rgba.0[3]),
        ])
    }
}

/// Converts a gamma-corrected color channel value [0, 255] to linear space [0, 1].
pub fn linear_f32_from_gamma_u8(s: u8) -> f32 {
    if s <= 10 {
        s as f32 / 3294.6
    } else {
        ((s as f32 + 14.025) / 269.025).powf(2.4)
    }
}

/// Converts a linear color channel value [0, 255] to linear space [0, 1].
/// This is especially useful for the alpha channel.
#[inline(always)]
pub fn linear_f32_from_linear_u8(a: u8) -> f32 {
    a as f32 / 255.0
}

/// Converts a linear color channel value [0, 1] to gamma-corrected [0, 255] (values are clamped).
/// Values outside this range are clamped to the nearest boundary.
pub fn gamma_u8_from_linear_f32(l: f32) -> u8 {
    if l <= 0.0 {
        0
    } else if l <= 0.0031308 {
        fast_round(3294.6 * l)
    } else if l <= 1.0 {
        fast_round(269.025 * l.powf(1.0 / 2.4) - 14.025)
    } else {
        255
    }
}

/// Converts a linear color channel value [0, 1] to [0, 255] (values are clamped).
/// Useful for alpha-channel conversion.
#[inline(always)]
pub fn linear_u8_from_linear_f32(a: f32) -> u8 {
    fast_round(a * 255.0)
}

fn fast_round(r: f32) -> u8 {
    (r + 0.5) as _ // Performs a rounding operation with a saturating cast.
}

#[test]
pub fn test_srgba_conversion() {
    for b in 0..=255 {
        let l = linear_f32_from_gamma_u8(b);
        assert!(0.0 <= l && l <= 1.0);
        assert_eq!(gamma_u8_from_linear_f32(l), b);
    }
}

/// Converts gamma-corrected color values [0, 1] to linear space [0, 1] (not clamped).
/// This function handles numbers outside the [0, 1] range, including negative values.
pub fn linear_from_gamma(gamma: f32) -> f32 {
    if gamma < 0.0 {
        -linear_from_gamma(-gamma)
    } else if gamma <= 0.04045 {
        gamma / 12.92
    } else {
        ((gamma + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts linear color values [0, 1] to gamma space [0, 1] (not clamped).
/// This function also handles numbers outside the [0, 1] range, including negative values.
pub fn gamma_from_linear(linear: f32) -> f32 {
    if linear < 0.0 {
        -gamma_from_linear(-linear)
    } else if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

// ----------------------------------------------------------------------------

/// Adjusts a color towards a target color to create a "grayed out" effect.
/// This is often used to indicate disabled or inactive states in UI elements.
pub fn tint_color_towards(color: Color32, target: Color32) -> Color32 {
    let [mut r, mut g, mut b, mut a] = color.to_array();

    if a == 0 {
        r /= 2;
        g /= 2;
        b /= 2;
    } else if a < 170 {
        // A simple approximation for a grayed-out effect.
        // Suitable for grid stripes and similar use cases.
        let div = (2 * 255 / a as i32) as u8;
        r = r / 2 + target.r() / div;
        g = g / 2 + target.g() / div;
        b = b / 2 + target.b() / div;
        a /= 2;
    } else {
        r = r / 2 + target.r() / 2;
        g = g / 2 + target.g() / 2;
        b = b / 2 + target.b() / 2;
    }
    Color32::from_rgba_premultiplied(r, g, b, a)
}
