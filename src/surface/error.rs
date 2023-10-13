//! Defines [`SurfaceError`].

use std::fmt;

/// An error that might occur when creating or interacting with a [`Surface`](super::Surface).
#[derive(Debug, Clone)]
pub enum SurfaceError {
    /// Vulkan behaved in an unexpected way.
    UnexpectedVulkanBehavior,
    /// The GPU does not support the provided surface.
    NotSupported,
    /// The surface has been lost.
    SurfaceLost,
    /// The configuration provided is incompatible with the surface.
    InvalidConfig,
}

impl fmt::Display for SurfaceError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::NotSupported => write!(f, "the GPU does the support the surface"),
            Self::SurfaceLost => write!(f, "the surface has been lost"),
            Self::InvalidConfig => write!(f, "the configuration provided is incompatible with the surface"),
        }
    }
}

impl std::error::Error for SurfaceError {}
