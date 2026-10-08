//! Choosing and sizing the cover art embedded into downloaded files.

use image::imageops::{self, FilterType};
use image::io::{Limits, Reader};
use image::{Rgb, Rgb32FImage};
use jpeg_encoder::{ColorType, Encoder, SamplingFactor};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::LazyLock;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Decoding holds the source as RGBA8 and then as linear RGB f32, 16 bytes a
/// pixel in all; past this side a cover would need over half a gigabyte.
const MAX_SOURCE_SIDE: u32 = 6000;
/// With 4:4:4 sampling, high enough that re-encoding loss is not visible at
/// cover sizes.
const JPEG_QUALITY: u8 = 92;

/// Largest cover side embedded into downloaded files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoverSize {
    Px400,
    Px500,
    /// Qobuz's own cover size, so its covers are embedded unchanged.
    #[default]
    Px600,
}

impl CoverSize {
    pub fn max_side(self) -> u32 {
        match self {
            CoverSize::Px400 => 400,
            CoverSize::Px500 => 500,
            CoverSize::Px600 => 600,
        }
    }

    /// All sizes, smallest first, for building dropdowns.
    pub const ALL: [CoverSize; 3] = [CoverSize::Px400, CoverSize::Px500, CoverSize::Px600];
}

impl std::fmt::Display for CoverSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} px", self.max_side())
    }
}

/// Fit cover `bytes` within `size`, returning them unchanged when they already
/// fit or cannot be processed, so a cover problem never fails a track.
/// Blocking and CPU-bound.
pub fn prepare_cover(bytes: Vec<u8>, size: CoverSize) -> Vec<u8> {
    match downscale(&bytes, size.max_side()) {
        Ok(Some(smaller)) => smaller,
        Ok(None) => bytes,
        Err(e) => {
            tracing::warn!("cover embedded at its original size: {e}");
            bytes
        }
    }
}

/// A JPEG of the image scaled so its longest side is `cap`, or `None` when it
/// already fits. Resampled in linear light: averaging gamma-encoded values
/// darkens fine high-contrast detail such as small text.
fn downscale(bytes: &[u8], cap: u32) -> Result<Option<Vec<u8>>, BoxError> {
    let (width, height) = reader(bytes)?.into_dimensions()?;
    if width.max(height) <= cap {
        return Ok(None);
    }
    let (new_width, new_height) = fit(width, height, cap);

    let source = reader(bytes)?.decode()?.into_rgba8();
    let linear = Rgb32FImage::from_fn(width, height, |x, y| {
        let [r, g, b, a] = source.get_pixel(x, y).0;
        let alpha = f32::from(a) / 255.0;
        // Transparent areas are composited over white, as on a light page.
        Rgb([r, g, b].map(|c| SRGB_TO_LINEAR[usize::from(c)] * alpha + (1.0 - alpha)))
    });
    let resized = imageops::resize(&linear, new_width, new_height, FilterType::Lanczos3);
    let rgb: Vec<u8> = resized
        .pixels()
        .flat_map(|p| p.0.map(linear_to_srgb))
        .collect();

    let mut jpeg = Vec::new();
    let mut encoder = Encoder::new(&mut jpeg, JPEG_QUALITY);
    encoder.set_sampling_factor(SamplingFactor::R_4_4_4);
    encoder.set_optimized_huffman_tables(true);
    encoder.encode(
        &rgb,
        u16::try_from(new_width)?,
        u16::try_from(new_height)?,
        ColorType::Rgb,
    )?;
    Ok(Some(jpeg))
}

fn reader(bytes: &[u8]) -> Result<Reader<Cursor<&[u8]>>, BoxError> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_SIDE);
    limits.max_image_height = Some(MAX_SOURCE_SIDE);
    let mut reader = Reader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(limits);
    Ok(reader)
}

/// Scale `width`×`height` so the longer side is `cap`, keeping the aspect ratio.
fn fit(width: u32, height: u32, cap: u32) -> (u32, u32) {
    let scale = |side: u32| {
        let scaled = f64::from(side) * f64::from(cap) / f64::from(width.max(height));
        (scaled.round() as u32).max(1)
    };
    (scale(width), scale(height))
}

/// [`srgb_to_linear`] for every 8-bit value, so linearizing a cover costs a
/// lookup per channel instead of a `powf`.
static SRGB_TO_LINEAR: LazyLock<[f32; 256]> =
    LazyLock::new(|| std::array::from_fn(|c| srgb_to_linear(c as u8)));

/// The sRGB transfer function's inverse (IEC 61966-2-1).
fn srgb_to_linear(c: u8) -> f32 {
    let v = f32::from(c) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Lanczos overshoots past [0, 1] at hard edges, so the value is clamped first.
fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let encoded = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageOutputFormat, RgbImage};

    fn png(image: &RgbImage) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, ImageOutputFormat::Png).unwrap();
        out.into_inner()
    }

    fn plain(width: u32, height: u32) -> Vec<u8> {
        png(&RgbImage::from_pixel(width, height, Rgb([200, 30, 30])))
    }

    fn decode(bytes: &[u8]) -> RgbImage {
        image::load_from_memory(bytes).unwrap().into_rgb8()
    }

    /// The horizontal and vertical sampling factors of each JPEG component,
    /// from the baseline start-of-frame segment.
    fn sampling_factors(jpeg: &[u8]) -> Vec<u8> {
        let sof = jpeg
            .windows(2)
            .position(|w| w == [0xFF, 0xC0])
            .expect("baseline JPEG");
        let components = usize::from(jpeg[sof + 9]);
        (0..components).map(|i| jpeg[sof + 11 + 3 * i]).collect()
    }

    #[test]
    fn sizes_are_listed_smallest_first() {
        let labels: Vec<String> = CoverSize::ALL.iter().map(|s| s.to_string()).collect();
        assert_eq!(labels, ["400 px", "500 px", "600 px"]);
    }

    #[test]
    fn qobuz_cover_at_600_is_unchanged() {
        let bytes = plain(600, 600);
        assert_eq!(prepare_cover(bytes.clone(), CoverSize::Px600), bytes);
    }

    #[test]
    fn small_cover_is_not_enlarged() {
        let bytes = plain(450, 450);
        assert_eq!(prepare_cover(bytes.clone(), CoverSize::Px500), bytes);
    }

    #[test]
    fn qobuz_cover_is_reduced() {
        let out = prepare_cover(plain(600, 600), CoverSize::Px400);
        assert_eq!(decode(&out).dimensions(), (400, 400));
    }

    #[test]
    fn non_square_cover_keeps_its_aspect_ratio() {
        let out = prepare_cover(plain(600, 450), CoverSize::Px500);
        assert_eq!(decode(&out).dimensions(), (500, 375));
    }

    #[test]
    fn resamples_in_linear_light() {
        let checkerboard = RgbImage::from_fn(64, 64, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255; 3])
            } else {
                Rgb([0; 3])
            }
        });
        let out = downscale(&png(&checkerboard), 32).unwrap().unwrap();
        let grey = decode(&out).get_pixel(16, 16).0;
        // Half white in linear light is sRGB 188; averaging encoded values
        // would give 128.
        for c in grey {
            assert!((182..=194).contains(&c), "got {grey:?}");
        }
    }

    #[test]
    fn jpeg_has_no_chroma_subsampling() {
        let out = prepare_cover(plain(1200, 1200), CoverSize::Px600);
        assert_eq!(sampling_factors(&out), vec![0x11, 0x11, 0x11]);
    }

    #[test]
    fn unreadable_cover_is_unchanged() {
        let garbage = b"<html>not an image</html>".to_vec();
        assert_eq!(prepare_cover(garbage.clone(), CoverSize::Px600), garbage);
    }

    #[test]
    fn fit_rounds_and_never_reaches_zero() {
        assert_eq!(fit(2000, 1500, 1000), (1000, 750));
        assert_eq!(fit(3000, 1, 1000), (1000, 1));
    }
}
