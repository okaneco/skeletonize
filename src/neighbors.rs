//! Struct and utilities for calculating the status of neighboring pixels.

use crate::{Edge, ForegroundColor};

/// Struct with information describing the surrounding pixels.
pub struct NeighborInfo {
    /// The number of neighbor pixels with non-background color.
    pub filled: u8,
    /// The number of surrounding pixels.
    pub neighbors: u8,
    /// An array containing the [status](crate::Edge) of the neighboring pixels.
    pub edge_status: [Edge; 8],
}

impl NeighborInfo {
    /// Calculate and return the number of transitions from
    /// [`Edge::Empty`](crate::Edge::Empty) to
    /// [`Edge::Filled`](crate::Edge::Filled).
    pub fn transitions(&self) -> u8 {
        let [p2, p3, p4, p5, p6, p7, p8, p9] = self.edge_status;

        let mut transitions = 0;
        for pair in [p2, p3, p4, p5, p6, p7, p8, p9, p2].windows(2) {
            if pair[0] == Edge::Empty && pair[1] == Edge::Filled {
                transitions += 1;
            }
        }

        transitions
    }
}

/// Calculate and return a [`NeighborInfo`](crate::neighbors::NeighborInfo)
/// struct which contains the number of occupied neighbor pixels, number of
/// neighbor pixels, and the [edge status](crate::Edge) of the neighboring
/// pixels.
pub fn get_neighbor_info<F: ForegroundColor>(
    img: &image::GrayImage,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> NeighborInfo {
    let mut filled = 0;
    let mut neighbors = 0;

    let p9 = if y > 0 && x > 0 {
        neighbors += 1;
        if img.get_pixel(x - 1, y - 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p2 = if y > 0 {
        neighbors += 1;
        if img.get_pixel(x, y - 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p3 = if y > 0 && x < u32::MAX && x + 1 < width {
        neighbors += 1;
        if img.get_pixel(x + 1, y - 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p8 = if x > 0 {
        neighbors += 1;
        if img.get_pixel(x - 1, y)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p4 = if x < u32::MAX && x + 1 < width {
        neighbors += 1;
        if img.get_pixel(x + 1, y)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p7 = if x > 0 && y < u32::MAX && y + 1 < height {
        neighbors += 1;
        if img.get_pixel(x - 1, y + 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p6 = if y < u32::MAX && y + 1 < height {
        neighbors += 1;
        if img.get_pixel(x, y + 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };
    let p5 = if x < u32::MAX && x + 1 < width && y < u32::MAX && y + 1 < height {
        neighbors += 1;
        if img.get_pixel(x + 1, y + 1)[0] != F::BACKGROUND_COLOR {
            filled += 1;
            Edge::Filled
        } else {
            Edge::Empty
        }
    } else {
        Edge::DoesNotExist
    };

    NeighborInfo {
        filled,
        neighbors,
        edge_status: [p2, p3, p4, p5, p6, p7, p8, p9],
    }
}
