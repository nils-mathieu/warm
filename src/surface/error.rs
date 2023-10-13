//! Defines [`SurfaceError`].

use std::fmt;

/// An error that might occur when creating a [`Surface`](super::Surface).
#[derive(Debug, Clone)]
pub enum SurfaceError {
    /// Vulkan behaved in an unexpected way.
    UnexpectedVulkanBehavior,
    /// The GPU does not support the provided surface.
    NotSupported,
}

impl fmt::Display for SurfaceError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
           Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
           Self::NotSupported => write!(f, "the GPU does the support the surface"),
        }
    }
}

impl std::error::Error for SurfaceError {}

/// An error that might occur when configuring a [`Surface`](super::Surface).
#[derive(Debug, Clone)]
pub struct SurfaceConfigureError;
