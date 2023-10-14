//! Defines [`SurfaceError`].

use std::fmt;

/// An error that might occur when creating or interacting with a [`Surface`](super::Surface).
#[derive(Debug, Clone)]
pub enum SurfaceError<C = std::convert::Infallible> {
    /// Vulkan behaved in an unexpected way.
    UnexpectedVulkanBehavior,
    /// The GPU does not support the provided surface.
    NotSupported,
    /// The surface has been lost.
    Lost,
    /// The configuration provided is incompatible with the surface.
    InvalidConfig,
    /// The [`SurfaceContents`](super::SurfaceContents) implementation failed.
    Contents(C),
}

impl SurfaceError {
    /// Casts this [`SurfaceError`] to another [`SurfaceError<C>`] with any type parameter
    /// `C`.
    pub fn cast_contents<C>(self) -> SurfaceError<C> {
        match self {
            Self::UnexpectedVulkanBehavior => SurfaceError::UnexpectedVulkanBehavior,
            Self::NotSupported => SurfaceError::NotSupported,
            Self::Lost => SurfaceError::Lost,
            Self::InvalidConfig => SurfaceError::InvalidConfig,
            Self::Contents(never) => match never {},
        }
    }
}

impl<C: fmt::Display> fmt::Display for SurfaceError<C> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::NotSupported => write!(f, "the GPU does the support the surface"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::InvalidConfig => write!(f, "the configuration provided is incompatible with the surface"),
            Self::Contents(ref contents) => fmt::Display::fmt(contents, f),
        }
    }
}

impl<C: std::error::Error> std::error::Error for SurfaceError<C> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::Contents(ref contents) => Some(contents),
            _ => None,
        }
    }
}

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
