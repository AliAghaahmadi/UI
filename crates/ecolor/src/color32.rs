use crate::{
    fast_round, gamma_u8_from_linear_f32, linear_f32_from_gamma_u8, linear_f32_from_linear_u8, Rgba,
};

/// This format is used for space-efficient color representation (32 bits).
///
/// Instead of manipulating this directly, it is often better
/// to first convert it to either [`Rgba`] or [`crate::Hsva`].
///
/// Internally, this uses 0-255 gamma space `sRGBA` color with premultiplied alpha.
/// The alpha channel is in linear space.
///
/// The special value of alpha=0 means the color is to be treated as an additive color.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct Color32(pub(crate) [u8; 4]);

impl std::ops::Index<usize> for Color32 {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Color32 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.0[index]
    }
}

impl Color32 {
    // Named colors based on common CSS color names:

    pub const TRANSPARENT: Self = Self::from_rgba_premultiplied(0, 0, 0, 0);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const DARK_GRAY: Self = Self::from_rgb(96, 96, 96);
    pub const GRAY: Self = Self::from_rgb(160, 160, 160);
    pub const LIGHT_GRAY: Self = Self::from_rgb(220, 220, 220);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);

    pub const BROWN: Self = Self::from_rgb(165, 42, 42);
    pub const DARK_RED: Self = Self::from_rgb(0x8B, 0, 0);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const LIGHT_RED: Self = Self::from_rgb(255, 128, 128);

    pub const YELLOW: Self = Self::from_rgb(255, 255, 0);
    pub const LIGHT_YELLOW: Self = Self::from_rgb(255, 255, 0xE0);
    pub const KHAKI: Self = Self::from_rgb(240, 230, 140);

    pub const DARK_GREEN: Self = Self::from_rgb(0, 0x64, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const LIGHT_GREEN: Self = Self::from_rgb(0x90, 0xEE, 0x90);

    pub const DARK_BLUE: Self = Self::from_rgb(0, 0, 0x8B);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);
    pub const LIGHT_BLUE: Self = Self::from_rgb(0xAD, 0xD8, 0xE6);

    pub const GOLD: Self = Self::from_rgb(255, 215, 0);

    pub const DEBUG_COLOR: Self = Self::from_rgba_premultiplied(0, 200, 0, 128);

    /// A placeholder color used as a special key for "no color".
    ///
    /// This color does not correspond to a valid multiplied color,
    /// nor to an additive color.
    /// It is used as a special color key and should be replaced
    /// before finalizing any screen rendering.
    pub const PLACEHOLDER: Self = Self::from_rgba_premultiplied(64, 254, 0, 128);

    #[deprecated = "Renamed to PLACEHOLDER"]
    pub const TEMPORARY_COLOR: Self = Self::PLACEHOLDER;

    #[inline]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    #[inline]
    pub const fn from_rgb_additive(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 0])
    }

    /// Creates a `Color32` from `sRGBA` values with premultiplied alpha.
    #[inline]
    pub const fn from_rgba_premultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    /// Creates a `Color32` from `sRGBA` values without premultiplied alpha.
    #[inline]
    pub fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        if a == 255 {
            Self::from_rgb(r, g, b) // Optimization for common case of fully opaque
        } else if a == 0 {
            Self::TRANSPARENT // Optimization for common case of fully transparent
        } else {
            let r_lin = linear_f32_from_gamma_u8(r);
            let g_lin = linear_f32_from_gamma_u8(g);
            let b_lin = linear_f32_from_gamma_u8(b);
            let a_lin = linear_f32_from_linear_u8(a);

            let r = gamma_u8_from_linear_f32(r_lin * a_lin);
            let g = gamma_u8_from_linear_f32(g_lin * a_lin);
            let b = gamma_u8_from_linear_f32(b_lin * a_lin);

            Self::from_rgba_premultiplied(r, g, b, a)
        }
    }

    #[inline]
    pub const fn from_gray(l: u8) -> Self {
        Self([l, l, l, 255])
    }

    #[inline]
    pub const fn from_black_alpha(a: u8) -> Self {
        Self([0, 0, 0, a])
    }

    #[inline]
    pub fn from_white_alpha(a: u8) -> Self {
        Rgba::from_white_alpha(linear_f32_from_linear_u8(a)).into()
    }

    #[inline]
    pub const fn from_additive_luminance(l: u8) -> Self {
        Self([l, l, l, 0])
    }

    #[inline]
    pub const fn is_opaque(&self) -> bool {
        self.a() == 255
    }

    #[inline]
    pub const fn r(&self) -> u8 {
        self.0[0]
    }

    #[inline]
    pub const fn g(&self) -> u8 {
        self.0[1]
    }

    #[inline]
    pub const fn b(&self) -> u8 {
        self.0[2]
    }

    #[inline]
    pub const fn a(&self) -> u8 {
        self.0[3]
    }

    /// Returns an opaque version of this color.
    #[inline]
    pub fn to_opaque(self) -> Self {
        Rgba::from(self).to_opaque().into()
    }

    /// Returns an additive version of this color.
    #[inline]
    pub const fn additive(self) -> Self {
        let [r, g, b, _] = self.to_array();
        Self([r, g, b, 0])
    }

    /// Checks if the alpha value is 0 (additive color).
    #[inline]
    pub fn is_additive(self) -> bool {
        self.a() == 0
    }

    /// Returns the color as a premultiplied RGBA array.
    #[inline]
    pub const fn to_array(&self) -> [u8; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }

    /// Returns the color as a premultiplied RGBA tuple.
    #[inline]
    pub const fn to_tuple(&self) -> (u8, u8, u8, u8) {
        (self.r(), self.g(), self.b(), self.a())
    }

    /// Converts the color to `sRGBA` values without premultiplying alpha.
    #[inline]
    pub fn to_srgba_unmultiplied(&self) -> [u8; 4] {
        Rgba::from(*self).to_srgba_unmultiplied()
    }

    /// Multiplies the color components by a factor (in gamma space) to adjust opacity.
    ///
    /// This operation is perceptually even and faster than [`Self::linear_multiply`].
    #[inline]
    pub fn gamma_multiply(self, factor: f32) -> Self {
        debug_assert!(0.0 <= factor && factor.is_finite());
        let Self([r, g, b, a]) = self;
        Self([
            (r as f32 * factor + 0.5) as u8,
            (g as f32 * factor + 0.5) as u8,
            (b as f32 * factor + 0.5) as u8,
            (a as f32 * factor + 0.5) as u8,
        ])
    }

    /// Multiplies the color components by a factor (in linear space) to adjust opacity.
    ///
    /// This operation is more computationally expensive due to conversion to and from linear space.
    /// Consider using [`Self::gamma_multiply`] for better performance.
    #[inline]
    pub fn linear_multiply(self, factor: f32) -> Self {
        debug_assert!(0.0 <= factor && factor.is_finite());
        // Conversion to linear space and back due to premultiplied alpha
        Rgba::from(self).multiply(factor).into()
    }

    /// Converts the color to floating point values in the range 0-1 without gamma correction.
    ///
    /// Use this method with caution; in most cases, you should convert to [`Rgba`] instead
    /// to obtain linear space color values.
    #[inline]
    pub fn to_normalized_gamma_f32(self) -> [f32; 4] {
        let Self([r, g, b, a]) = self;
        [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ]
    }

    /// Linearly interpolates between this color and another color by `t` in gamma space.
    pub fn lerp_to_gamma(&self, other: Self, t: f32) -> Self {
        use emath::lerp;

        Self::from_rgba_premultiplied(
            fast_round(lerp((self[0] as f32)..=(other[0] as f32), t)),
            fast_round(lerp((self[1] as f32)..=(other[1] as f32), t)),
            fast_round(lerp((self[2] as f32)..=(other[2] as f32), t)),
            fast_round(lerp((self[3] as f32)..=(other[3] as f32), t)),
        )
    }
}