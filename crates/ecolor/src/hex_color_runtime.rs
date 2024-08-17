//! Convert colors to and from the hex-color string format at runtime
//!
//! Supports the 3, 4, 6, and 8-digit formats, according to the specification in
//! <https://drafts.csswg.org/css-color-4/#hex-color>

use std::{fmt::Display, str::FromStr};

use crate::Color32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
/// Enum representing hexadecimal color formats.
/// Each variant holds a Color32 instance.
pub enum HexColor {
    /// 3 hexadecimal digits (RGB format)
    Hex3(Color32),

    /// 4 hexadecimal digits (RGBA format)
    Hex4(Color32),

    /// 6 hexadecimal digits (RGB format with 2 digits per channel)
    Hex6(Color32),

    /// 8 hexadecimal digits (RGBA format with 2 digits per channel)
    Hex8(Color32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Enum representing possible errors when parsing a hexadecimal color string.
pub enum ParseHexColorError {
    /// The hexadecimal color string is missing the leading '#'
    MissingHash,

    /// The length of the string does not match the expected length for valid formats
    InvalidLength,

    /// Error parsing integer values from the hexadecimal string
    InvalidInt(std::num::ParseIntError),
}

impl FromStr for HexColor {
    type Err = ParseHexColorError;

    /// Parses a string into a HexColor instance.
    /// Removes the leading '#' if present and delegates to `from_str_without_hash`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('#')
            .ok_or(ParseHexColorError::MissingHash)
            .and_then(Self::from_str_without_hash)
    }
}

impl Display for HexColor {
    /// Formats the HexColor instance as a hexadecimal color string.
    /// Handles different formats (3, 4, 6, 8-digit) and adjusts the output accordingly.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex3(color) => {
                let [r, g, b, _] = color.to_srgba_unmultiplied().map(|u| u >> 4);
                f.write_fmt(format_args!("#{r:x}{g:x}{b:x}"))
            }
            Self::Hex4(color) => {
                let [r, g, b, a] = color.to_srgba_unmultiplied().map(|u| u >> 4);
                f.write_fmt(format_args!("#{r:x}{g:x}{b:x}{a:x}"))
            }
            Self::Hex6(color) => {
                let [r, g, b, _] = color.to_srgba_unmultiplied();
                let u = u32::from_be_bytes([0, r, g, b]);
                f.write_fmt(format_args!("#{u:06x}"))
            }
            Self::Hex8(color) => {
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                let u = u32::from_be_bytes([r, g, b, a]);
                f.write_fmt(format_args!("#{u:08x}"))
            }
        }
    }
}

impl HexColor {
    /// Retrieves the inner Color32 instance.
    #[inline]
    pub fn color(&self) -> Color32 {
        match self {
            Self::Hex3(color) | Self::Hex4(color) | Self::Hex6(color) | Self::Hex8(color) => *color,
        }
    }

    /// Parses a hexadecimal color string without the leading '#' character.
    /// Handles different lengths and converts the string into a Color32 instance based on the format.
    #[inline]
    pub fn from_str_without_hash(s: &str) -> Result<Self, ParseHexColorError> {
        match s.len() {
            3 => {
                let [r, gb] = u16::from_str_radix(s, 16)
                    .map_err(ParseHexColorError::InvalidInt)?
                    .to_be_bytes();
                let [r, g, b] = [r, gb >> 4, gb & 0x0f].map(|u| u << 4 | u);
                Ok(Self::Hex3(Color32::from_rgb(r, g, b)))
            }
            4 => {
                let [r_g, b_a] = u16::from_str_radix(s, 16)
                    .map_err(ParseHexColorError::InvalidInt)?
                    .to_be_bytes();
                let [r, g, b, a] = [r_g >> 4, r_g & 0x0f, b_a >> 4, b_a & 0x0f].map(|u| u << 4 | u);
                Ok(Self::Hex4(Color32::from_rgba_unmultiplied(r, g, b, a)))
            }
            6 => {
                let [_, r, g, b] = u32::from_str_radix(s, 16)
                    .map_err(ParseHexColorError::InvalidInt)?
                    .to_be_bytes();
                Ok(Self::Hex6(Color32::from_rgb(r, g, b)))
            }
            8 => {
                let [r, g, b, a] = u32::from_str_radix(s, 16)
                    .map_err(ParseHexColorError::InvalidInt)?
                    .to_be_bytes();
                Ok(Self::Hex8(Color32::from_rgba_unmultiplied(r, g, b, a)))
            }
            _ => Err(ParseHexColorError::InvalidLength)?,
        }
    }
}

impl Color32 {
    /// Parses a color from a hex string.
    /// Supports the 3, 4, 6, and 8-digit formats.
    /// Returns an error if the string does not match these formats or contains non-hex characters.
    pub fn from_hex(hex: &str) -> Result<Self, ParseHexColorError> {
        HexColor::from_str(hex).map(|h| h.color())
    }

    /// Formats the color as an 8-digit hex string.
    /// Uses the 8-digit format which is lossless.
    /// For other formats, see HexColor.
    #[inline]
    pub fn to_hex(&self) -> String {
        HexColor::Hex8(*self).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_string_formats() {
        use Color32 as C;
        use HexColor as H;
        let cases = [
            (H::Hex3(C::RED), "#f00"),
            (H::Hex4(C::RED), "#f00f"),
            (H::Hex6(C::RED), "#ff0000"),
            (H::Hex8(C::RED), "#ff0000ff"),
            (H::Hex3(C::GREEN), "#0f0"),
            (H::Hex4(C::GREEN), "#0f0f"),
            (H::Hex6(C::GREEN), "#00ff00"),
            (H::Hex8(C::GREEN), "#00ff00ff"),
            (H::Hex3(C::BLUE), "#00f"),
            (H::Hex4(C::BLUE), "#00ff"),
            (H::Hex6(C::BLUE), "#0000ff"),
            (H::Hex8(C::BLUE), "#0000ffff"),
            (H::Hex3(C::WHITE), "#fff"),
            (H::Hex4(C::WHITE), "#ffff"),
            (H::Hex6(C::WHITE), "#ffffff"),
            (H::Hex8(C::WHITE), "#ffffffff"),
            (H::Hex3(C::BLACK), "#000"),
            (H::Hex4(C::BLACK), "#000f"),
            (H::Hex6(C::BLACK), "#000000"),
            (H::Hex8(C::BLACK), "#000000ff"),
            (H::Hex4(C::TRANSPARENT), "#0000"),
            (H::Hex8(C::TRANSPARENT), "#00000000"),
        ];
        for (color, string) in cases {
            assert_eq!(color.to_string(), string, "{color:?} <=> {string}");
            assert_eq!(
                H::from_str(string).unwrap(),
                color,
                "{color:?} <=> {string}"
            );
        }
    }

    #[test]
    fn hex_string_round_trip() {
        use Color32 as C;
        let cases = [
            C::from_rgba_unmultiplied(10, 20, 30, 0),
            C::from_rgba_unmultiplied(10, 20, 30, 40),
            C::from_rgba_unmultiplied(10, 20, 30, 255),
            C::from_rgba_unmultiplied(0, 20, 30, 0),
            C::from_rgba_unmultiplied(10, 0, 30, 40),
            C::from_rgba_unmultiplied(10, 20, 0, 255),
        ];
        for color in cases {
            assert_eq!(C::from_hex(color.to_hex().as_str()), Ok(color));
        }
    }
}
