//! Image thinning error enums.

/// Error for edge thinning and edge detection.
#[derive(Copy, Clone, Debug)]
pub enum SkeletonizeError {
    /// Error converting an image to a grayscale image.
    LumaConversion(LumaConversionErrorKind),
    /// The edge thinning algorithm reached the maximum amount of iterations.
    MaxThinningIterations,
}

/// Errors that occur when attempting to convert an image to grayscale.
#[derive(Copy, Clone, Debug)]
pub enum LumaConversionErrorKind {
    /// Error converting an image into grayscale in an edge thinning algorithm.
    ImageThinningLuma,
    /// Error converting an image to grayscale in an edge detection function.
    SobelLuma,
    /// Error converting an image into a mutable grayscale image view in an edge
    /// detection function.
    SobelMutableLuma,
    /// Error converting an image into a mutable grayscale image view for
    /// thresholding.
    ThresholdMutableLuma,
}

impl core::fmt::Display for LumaConversionErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ImageThinningLuma => {
                write!(f, "Could not create a grayscale image in image thinning")
            }
            Self::SobelLuma => write!(f, "Could not create a grayscale image in edge detection"),
            Self::SobelMutableLuma => write!(
                f,
                "Could not create a mutable grayscale image view in edge detection"
            ),
            Self::ThresholdMutableLuma => write!(
                f,
                "Could not create a mutable grayscale image view for thresholding"
            ),
        }
    }
}

impl core::fmt::Display for SkeletonizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LumaConversion(err) => write!(f, "{err}"),
            Self::MaxThinningIterations => {
                write!(f, "Maximum iteration count reached in thinning algorithm")
            }
        }
    }
}

impl std::error::Error for SkeletonizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LumaConversion(_) | Self::MaxThinningIterations => None,
        }
    }
}
