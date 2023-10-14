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
    Lost,
    /// The configuration provided is incompatible with the surface.
    InvalidConfig,
}

impl fmt::Display for SurfaceError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::NotSupported => write!(f, "the GPU does the support the surface"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::InvalidConfig => write!(f, "the configuration provided is incompatible with the surface"),
        }
    }
}

impl std::error::Error for SurfaceError {}

/// An error that might occur when presenting an image to the surface.
#[derive(Debug, Clone)]
pub enum PresentError<Contents> {
    /// Vulkan behaved in an unexpected way.
    UnexpectedVulkanBehavior,
    /// The surface has been lost.
    Lost,
    /// The surface is out of date and must be reconfigured.
    OutOfDate,
    /// No image could be acquired within the desired timeout.
    Timeout,
    /// The swapchain has been retired.
    SwapchainRetired,
    /// The [`SurfaceContents`](super::SurfaceContents) implementation failed.
    Contents(Contents),
}

impl<C: fmt::Display> fmt::Display for PresentError<C> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::OutOfDate => write!(f, "the surface is out of date"),
            Self::Timeout => write!(f, "no image could be acquired within the desired timeout"),
            Self::SwapchainRetired => write!(f, "the swapchain is retired and must be recreated"),
            Self::Contents(ref contents) => write!(f, "the surface contents failed: {}", contents),
        }
    }
}

impl<C: std::error::Error> std::error::Error for PresentError<C> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::Contents(ref contents) => Some(contents),
            _ => None,
        }
    }
}
