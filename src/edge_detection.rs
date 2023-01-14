//! Edge detection algorithms for preprocessing images.

use crate::error::{LumaConversionErrorKind, SkeletonizeError};
use crate::ForegroundColor;

/// Sobel vertical `North` gradient operator.
#[rustfmt::skip]
pub const SOBEL_NORTH: [f32; 9] = [
    1.0, 2.0, 1.0,
    0.0, 0.0, 0.0,
    -1.0, -2.0, -1.0,
];
/// Sobel vertical `South` gradient operator.
#[rustfmt::skip]
pub const SOBEL_SOUTH: [f32; 9] = [
    -1.0, -2.0, -1.0,
    0.0, 0.0, 0.0,
    1.0, 2.0, 1.0,
];
/// Sobel horizontal `East` gradient operator.
#[rustfmt::skip]
pub const SOBEL_EAST: [f32; 9] = [
    -1.0, 0.0, 1.0,
    -2.0, 0.0, 2.0,
    -1.0, 0.0, 1.0,
];
/// Sobel horizontal `West` gradient operator.
#[rustfmt::skip]
pub const SOBEL_WEST: [f32; 9] = [
    1.0, 0.0, -1.0,
    2.0, 0.0, -2.0,
    1.0, 0.0, -1.0,
];

/// Detect edges in an image using [`SOBEL_EAST`](SOBEL_EAST) and
/// [`SOBEL_NORTH`](SOBEL_NORTH) gradient operators.
/// The image should not have transparency.
///
/// `threshold` is an optional parameter between 0.0 and 1.0 which is used to
/// binarize the image. Pixels below that `Luma` threshold will be converted
/// to the background color.
pub fn sobel<F: ForegroundColor>(
    img: &image::DynamicImage,
    threshold: Option<f32>,
) -> Result<image::DynamicImage, SkeletonizeError> {
    let mut filter_up = img.filter3x3(&SOBEL_NORTH);
    let filtered_right = img.filter3x3(&SOBEL_EAST);
    let mutable_error = SkeletonizeError::LumaConversion(LumaConversionErrorKind::SobelMutableLuma);
    let immutable_error = SkeletonizeError::LumaConversion(LumaConversionErrorKind::SobelLuma);

    let iter_down = filter_up.as_mut_luma8().ok_or(mutable_error)?.iter_mut();
    let iter_right = filtered_right.as_luma8().ok_or(immutable_error)?.iter();

    for (g_down, g_right) in iter_down.zip(iter_right) {
        let res = (f32::from(*g_down) / 255.0).hypot(f32::from(*g_right) / 255.0);

        if let Some(threshold) = threshold {
            *g_down = if res < threshold {
                F::BACKGROUND_COLOR
            } else {
                !F::BACKGROUND_COLOR
            }
        } else {
            *g_down = (res * 255.0).round() as u8;
        }
    }

    // If ForegroundColor is Black and threshold None, edges would stay white
    // so we need to invert the result before returning it.
    if threshold.is_none() && F::BACKGROUND_COLOR == 255 {
        filter_up.invert()
    }

    Ok(filter_up)
}

/// Detect edges in an image using four Sobel gradient operators:
/// [`SOBEL_NORTH`](SOBEL_NORTH), [`SOBEL_SOUTH`](SOBEL_SOUTH),
/// [`SOBEL_EAST`](SOBEL_EAST), and [`SOBEL_WEST`](SOBEL_WEST).
/// The image should not have transparency.
///
/// `threshold` is an optional parameter between 0.0 and 1.0 which is used to
/// binarize the image. Pixels below that `Luma` threshold will be converted
/// to the background color.
pub fn sobel4<F: ForegroundColor>(
    img: &image::DynamicImage,
    threshold: Option<f32>,
) -> Result<image::DynamicImage, SkeletonizeError> {
    let mut filter_up = img.filter3x3(&SOBEL_NORTH);
    let filter_down = img.filter3x3(&SOBEL_SOUTH);
    let filter_right = img.filter3x3(&SOBEL_EAST);
    let filter_left = img.filter3x3(&SOBEL_WEST);

    let mutable_error = SkeletonizeError::LumaConversion(LumaConversionErrorKind::SobelMutableLuma);
    let immutable_error = SkeletonizeError::LumaConversion(LumaConversionErrorKind::SobelLuma);

    let iter_up = filter_up.as_mut_luma8().ok_or(mutable_error)?.iter_mut();
    let iter_down = filter_down.as_luma8().ok_or(immutable_error)?.iter();
    let iter_right = filter_right.as_luma8().ok_or(immutable_error)?.iter();
    let iter_left = filter_left.as_luma8().ok_or(immutable_error)?.iter();

    for (((g_up, g_down), g_left), g_right) in iter_up.zip(iter_down).zip(iter_right).zip(iter_left)
    {
        let vertical = (f32::from(*g_up) - f32::from(*g_down)) / 255.0;
        let horizontal = (f32::from(*g_right) - f32::from(*g_left)) / 255.0;
        let res = vertical.hypot(horizontal);

        if let Some(threshold) = threshold {
            *g_up = if res < threshold {
                F::BACKGROUND_COLOR
            } else {
                !F::BACKGROUND_COLOR
            }
        } else {
            *g_up = (res * 255.0).round() as u8;
        }
    }

    // If ForegroundColor is Black and threshold None, edges would stay white
    // so we need to invert the result before returning it.
    if threshold.is_none() && F::BACKGROUND_COLOR == 255 {
        filter_up.invert()
    }

    Ok(filter_up)
}
