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

impl From<UnexpectedVulkanBehavior> for SurfaceError {
    #[inline(always)]
    fn from(_: UnexpectedVulkanBehavior) -> Self {
        Self::UnexpectedVulkanBehavior
    }
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
pub enum PresentError {
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
}

impl From<UnexpectedVulkanBehavior> for PresentError {
    #[inline(always)]
    fn from(_: UnexpectedVulkanBehavior) -> Self {
        Self::UnexpectedVulkanBehavior
    }
}

impl fmt::Display for PresentError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::OutOfDate => write!(f, "the surface is out of date"),
            Self::Timeout => write!(f, "no image could be acquired within the desired timeout"),
            Self::SwapchainRetired => write!(f, "the swapchain is retired and must be recreated"),
        }
    }
}

impl std::error::Error for PresentError {}

/// An error that might occur when interacting with the Vulkan API.
#[derive(Debug, Clone, Copy)]
pub struct UnexpectedVulkanBehavior;

impl fmt::Display for UnexpectedVulkanBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vulkan implementation behaved unexpectedly")
    }
}

impl std::error::Error for UnexpectedVulkanBehavior {}
