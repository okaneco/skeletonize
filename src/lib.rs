//! A line thinning library for binary images, including edge detection and
//! threshold functions for preprocessing images into binary images.
//!
//! The goal of thinning is to remove excess pixels from the image until the
//! lines present are one pixel wide, resembling a "skeleton" of the original
//! pattern.
//!
//! The thinning algorithms are based on the papers *Zhang & Suen, 1984* and
//! *Chen & Hsu, 1988*. See [Reference](#reference).
//!
//! ## Usage
//!
//! There are three main workflows for thinning images with this library. The
//! second and third workflows produce binarized images with library functions
//! before thinning the image.
//!
//! The generic [`ForegroundColor`](crate::ForegroundColor) parameter on
//! [`edge_detection::sobel`][sobel], [`edge_detection::sobel4`][sobel4], and
//! [`thin_image_edges`](crate::thin_image_edges) specifies what foreground and
//! background colors the resulting
//! [`thin_image_edges`](crate::thin_image_edges) image will produce. The
//! foreground color is the color of the line to be thinned. A foreground color
//! of white will have a black background and a foreground of black will have a
//! white background. The generic parameters must match when using an edge
//! detection function in combination with the thinning function.
//!
//! [sobel]: crate::edge_detection::sobel
//! [sobel4]: crate::edge_detection::sobel4
//!
//! An example program can be viewed at `/examples/skeletonize.rs`.
//!
//! #### No preprocessing
//!
//! The image is already binarized so the edges can be thinned immediately.
//!
//! ```
//! # fn main() -> Result<(), skeletonize::error::SkeletonizeError> {
//! use skeletonize::{foreground, thin_image_edges, MarkingMethod};
//!
//! # let image_buffer = image::ImageBuffer::from_pixel(1, 1, image::Rgb([255, 255, 255]));
//! # let mut img = image::DynamicImage::ImageRgb8(image_buffer).grayscale();
//! let method = MarkingMethod::Modified;
//!
//! thin_image_edges::<foreground::Black>(&mut img, method, None)?;
//! # Ok(())
//! # }
//! ```
//!
//! If this produces poor results and/or takes a long time to run:
//! - the incorrect foreground color may have been chosen - try using the
//! opposite color, or
//! - the image may not be binary and needs to be thresholded.
//!
//! #### Edge detection
//!
//! Run an edge detection filter on the image and threshold those results before
//! thinning the lines. Note that the foreground color parameters must match on
//! the edge detection function and the thinning function.
//!
//! ```
//! # fn main() -> Result<(), skeletonize::error::SkeletonizeError> {
//! use skeletonize::edge_detection::sobel4;
//! use skeletonize::{foreground, thin_image_edges, MarkingMethod};
//!
//! # let image_buffer = image::ImageBuffer::from_pixel(2, 2, image::Rgb([255, 255, 255]));
//! # let img = image::DynamicImage::ImageRgb8(image_buffer).grayscale();
//! let method = MarkingMethod::Modified;
//! let threshold = Some(0.1);
//!
//! let mut filtered = sobel4::<foreground::White>(&img, threshold)?;
//! thin_image_edges::<foreground::White>(&mut filtered, method, None)?;
//! # Ok(())
//! # }
//! ```
//!
//! #### Thresholding
//!
//! Threshold the image before thinning, e.g., cleaning up a grayscale image.
//!
//! ```
//! # fn main() -> Result<(), skeletonize::error::SkeletonizeError> {
//! use skeletonize::{foreground, thin_image_edges, threshold, MarkingMethod};
//!
//! # let image_buffer = image::ImageBuffer::from_pixel(2, 2, image::Rgb([255, 255, 255]));
//! # let mut img = image::DynamicImage::ImageRgb8(image_buffer).grayscale();
//! let method = MarkingMethod::Modified;
//! let threshold = 0.1;
//!
//! skeletonize::threshold(&mut img, threshold)?;
//! thin_image_edges::<foreground::Black>(&mut img, method, None)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Reference
//!
//! Zhang, T. Y. & Suen, C. Y. (1984). A fast parallel algorithm for thinning
//! digital patterns. Commun. ACM 27, 3 (March 1984), 236–239.
//! [DOI:10.1145/357994.358023](https://doi.org/10.1145/357994.358023)
//!
//! Chen, Yung-Sheng & Hsu, Wen-Hsing. (1988). A modified fast parallel
//! algorithm for thinning digital patterns. Pattern Recognition Letters. 7.
//! 99-106.
//! [DOI:10.1016/0167-8655(88)90124-9](https://doi.org/10.1016/0167-8655(88)90124-9)
#![warn(missing_docs, rust_2018_idioms, unsafe_code)]

pub mod edge_detection;
pub mod error;
pub mod neighbors;
mod thinning;

use error::{LumaConversionErrorKind, SkeletonizeError};
pub use thinning::thin_image_edges;

/// Represents the color of the foreground or features in a binary image. For
/// example, white text on a black background has a white foreground color and
/// black background color.
pub trait ForegroundColor {
    /// The background color of the image for binarization.
    const BACKGROUND_COLOR: u8;
}

/// Implementations of [`ForegroundColor`](crate::ForegroundColor).
pub mod foreground {
    /// Black foreground color, represented as `0`.
    pub struct Black;

    impl crate::ForegroundColor for Black {
        const BACKGROUND_COLOR: u8 = 255;
    }

    /// White foreground color, represented by `255`.
    pub struct White;

    impl crate::ForegroundColor for White {
        const BACKGROUND_COLOR: u8 = 0;
    }
}

/// Classification of pixels in an image used for edge thinning.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Edge {
    /// The pixel does not contain the foreground color.
    Empty = 0,
    /// The pixel contains the foreground color.
    Filled = 1,
    /// The pixel is not a valid location within the image.
    DoesNotExist,
}

impl Edge {
    /// Convert the edge status into a `u8` representation.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Empty | Self::DoesNotExist => 0,
            Self::Filled => 1,
        }
    }
}

/// The algorithm that determines which pixels are removed during the edge
/// thinning process.
///
/// ### Reference
///
/// <span id="standard"></span>Zhang, T. Y. & Suen, C. Y. (1984). A fast
/// parallel algorithm for thinning digital patterns. Commun. ACM 27, 3 (March
/// 1984), 236–239.
/// [DOI:10.1145/357994.358023](https://doi.org/10.1145/357994.358023)
///
/// <span id="modified"></span>Chen, Yung-Sheng & Hsu, Wen-Hsing. (1988). A
/// modified fast parallel algorithm for thinning digital patterns. Pattern
/// Recognition Letters. 7. 99-106.
/// [DOI:10.1016/0167-8655(88)90124-9](https://doi.org/10.1016/0167-8655(88)90124-9)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkingMethod {
    /// An algorithm based on `Zhang and Suen, 1984`.
    ///
    /// See [MarkingMethod](crate::MarkingMethod#standard) for reference.
    Standard,
    /// An improved and slightly more complex algorithm than `Standard` based on
    /// `Chen and Hsu, 1988`. This algorithm improves on the original's
    /// weaknesses with generally thinner lines and better line connectivity.
    ///
    /// See [MarkingMethod](crate::MarkingMethod#modified) for reference.
    Modified,
}

impl Default for MarkingMethod {
    fn default() -> Self {
        Self::Modified
    }
}

/// Create a binary image where values below `threshold` become black and above
/// become white. `threshold` ranges from 0.0 to 1.0.
pub fn threshold(img: &mut image::DynamicImage, threshold: f32) -> Result<(), SkeletonizeError> {
    for pix in img
        .as_mut_luma8()
        .ok_or(SkeletonizeError::LumaConversion(
            LumaConversionErrorKind::ThresholdMutableLuma,
        ))?
        .iter_mut()
    {
        *pix = if *pix < (threshold * 255.0).round() as u8 {
            0
        } else {
            255
        };
    }

    Ok(())
}
