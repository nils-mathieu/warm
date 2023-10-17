//! Defines [`SurfaceError`].

use std::fmt;

use crate::VulkanError;

/// An error that might occur when creating or interacting with a [`Surface`](super::Surface).
#[derive(Debug, Clone)]
pub enum SurfaceError {
    /// The Vulkan implementation returned an unexpected error.
    UnexpectedError(VulkanError),
    /// The GPU does not support the provided surface.
    NotSupported,
    /// The surface has been lost.
    Lost,
    /// The configuration provided is incompatible with the surface.
    InvalidConfig,
}

impl From<VulkanError> for SurfaceError {
    #[inline(always)]
    fn from(value: VulkanError) -> Self {
        Self::UnexpectedError(value)
    }
}

impl fmt::Display for SurfaceError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedError(err) => write!(f, "unexpected Vulkan error: {err}"),
            Self::NotSupported => write!(f, "the GPU does the support the surface"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::InvalidConfig => write!(f, "the configuration provided is incompatible with the surface"),
        }
    }
}

impl std::error::Error for SurfaceError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::UnexpectedError(ref err) => Some(err),
            _ => None,
        }
    }
}

/// An error that might occur when presenting an image to the surface.
#[derive(Debug, Clone)]
pub enum PresentError {
    /// An unexpected error occurred.
    UnexpectedError(VulkanError),
    /// The surface has been lost.
    Lost,
    /// The surface is out of date and must be reconfigured.
    OutOfDate,
    /// No image could be acquired within the desired timeout.
    Timeout,
    /// The swapchain has been retired.
    SwapchainRetired,
}

impl From<VulkanError> for PresentError {
    #[inline(always)]
    fn from(error: VulkanError) -> Self {
        Self::UnexpectedError(error)
    }
}

impl fmt::Display for PresentError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedError(err) => write!(f, "unexpected Vulkan error: {err}"),
            Self::Lost => write!(f, "the surface has been lost"),
            Self::OutOfDate => write!(f, "the surface is out of date"),
            Self::Timeout => write!(f, "no image could be acquired within the desired timeout"),
            Self::SwapchainRetired => write!(f, "the swapchain is retired and must be recreated"),
        }
    }
}

impl std::error::Error for PresentError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::UnexpectedError(ref err) => Some(err),
            _ => None,
        }
    }
}
