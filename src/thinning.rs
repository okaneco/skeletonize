//! Functions for performing image thinning.

use image::GenericImage;

use crate::error::{LumaConversionErrorKind, SkeletonizeError};
use crate::neighbors::get_neighbor_info;
use crate::{Edge, ForegroundColor, MarkingMethod};

/// Perform image thinning on a binarized image `img` using one of the methods
/// in [`MarkingMethod`](crate::MarkingMethod). Returns the number of iterations
/// needed for thinning on successful completion.
///
/// `iterations` is an optional parameter set to `u32::MAX` if `None`.
#[allow(clippy::collapsible_else_if)]
#[allow(clippy::nonminimal_bool)]
pub fn thin_image_edges<F: ForegroundColor>(
    img: &mut image::DynamicImage,
    method: MarkingMethod,
    iterations: Option<u32>,
) -> Result<u32, SkeletonizeError> {
    let mut pixels_to_remove = Vec::new();
    let mut phase_one = true;
    let iterations = iterations.unwrap_or(u32::MAX);

    for iters in 0..iterations {
        let luma_img = img.as_luma8().ok_or(SkeletonizeError::LumaConversion(
            LumaConversionErrorKind::ImageThinningLuma,
        ))?;
        let (width, height) = luma_img.dimensions();

        // Mark pixels to remove
        for (x, y, p) in luma_img.enumerate_pixels() {
            if *p == image::Luma([F::BACKGROUND_COLOR]) {
                continue;
            }

            let info = get_neighbor_info::<F>(luma_img, width, height, x, y);
            let [p2, p3, p4, p5, p6, p7, p8, p9] = info.edge_status;

            match method {
                MarkingMethod::Standard => {
                    // Zhang and Suen, 1984

                    // Don't mark if we don't have 8 neighbors and 2..=6 aren't filled
                    if !(2..=6).contains(&info.filled) || info.neighbors != 8 {
                        continue;
                    }

                    // Count the number of times an edge transitions from empty to filled
                    let mut transitions = 0;
                    for pair in [p2, p3, p4, p5, p6, p7, p8, p9, p2].windows(2) {
                        if pair[0] == Edge::Empty && pair[1] == Edge::Filled {
                            transitions += 1;
                        }
                    }

                    if transitions != 1 {
                        continue;
                    }

                    if phase_one {
                        if (p2 == Edge::Empty || p4 == Edge::Empty || p6 == Edge::Empty)
                            && (p4 == Edge::Empty || p6 == Edge::Empty || p8 == Edge::Empty)
                        {
                            pixels_to_remove.push((x, y));
                        }
                    } else {
                        if (p2 == Edge::Empty || p4 == Edge::Empty || p8 == Edge::Empty)
                            && (p2 == Edge::Empty || p6 == Edge::Empty || p8 == Edge::Empty)
                        {
                            pixels_to_remove.push((x, y));
                        }
                    }
                }
                MarkingMethod::Modified => {
                    // Chen and Hsu, 1988

                    if !(2..=7).contains(&info.filled) || info.neighbors != 8 {
                        continue;
                    }

                    let mut transitions = 0;
                    for pair in [p2, p3, p4, p5, p6, p7, p8, p9, p2].windows(2) {
                        if pair[0] == Edge::Empty && pair[1] == Edge::Filled {
                            transitions += 1;
                        }
                    }

                    if !(transitions == 1 || transitions == 2) {
                        continue;
                    }

                    if phase_one {
                        if transitions == 1 {
                            if (p2 == Edge::Empty || p4 == Edge::Empty || p6 == Edge::Empty)
                                && (p4 == Edge::Empty || p6 == Edge::Empty || p8 == Edge::Empty)
                            {
                                pixels_to_remove.push((x, y));
                            }
                        } else {
                            if ((p2 == Edge::Filled && p4 == Edge::Filled)
                                && (p6 == Edge::Empty && p7 == Edge::Empty && p8 == Edge::Empty))
                                || ((p4 == Edge::Filled && p6 == Edge::Filled)
                                    && (p2 == Edge::Empty
                                        && p8 == Edge::Empty
                                        && p9 == Edge::Empty))
                            {
                                pixels_to_remove.push((x, y));
                            }
                        }
                    } else {
                        if transitions == 1 {
                            if (p2 == Edge::Empty || p4 == Edge::Empty || p8 == Edge::Empty)
                                && (p2 == Edge::Empty || p6 == Edge::Empty || p8 == Edge::Empty)
                            {
                                pixels_to_remove.push((x, y));
                            }
                        } else {
                            if ((p2 == Edge::Filled && p8 == Edge::Filled)
                                && (p4 == Edge::Empty && p5 == Edge::Empty && p6 == Edge::Empty))
                                || ((p6 == Edge::Filled && p8 == Edge::Filled)
                                    && (p2 == Edge::Empty
                                        && p3 == Edge::Empty
                                        && p4 == Edge::Empty))
                            {
                                pixels_to_remove.push((x, y));
                            }
                        }
                    }
                }
            }
        }

        phase_one = !phase_one;

        // Replace marked pixels with background color to thin the edges
        for &(x, y) in &pixels_to_remove {
            img.put_pixel(x, y, image::Rgba([F::BACKGROUND_COLOR; 4]));
        }

        if pixels_to_remove.is_empty() {
            return Ok(iters);
        }

        pixels_to_remove.clear();
    }

    Err(SkeletonizeError::MaxThinningIterations)
}
