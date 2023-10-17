//! Defines [`GpuError`].

use std::fmt;

use crate::VulkanError;

/// An error that might occur when creating a [`Gpu`](super::Gpu) instance.
#[derive(Debug)]
pub enum GpuError {
    /// The Vulkan dynamic library could not be loaded.
    CantLoadVulkan,
    /// The Vulkan implementation behaved in an unexpected way.
    UnexpectedBehavior,
    /// The Vulkan implementation returned an unexpected error.
    UnexpectedError(VulkanError),
    /// No suitable GPU was found on the system.
    NoSuitableGpu,
    /// The Vulkan implementation does not support the features required by the crate.
    Unsupported,
}

impl From<VulkanError> for GpuError {
    #[inline(always)]
    fn from(value: VulkanError) -> Self {
        Self::UnexpectedError(value)
    }
}

impl fmt::Display for GpuError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CantLoadVulkan => write!(f, "could not load Vulkan dynamic library"),
            Self::UnexpectedBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::UnexpectedError(err) => write!(f, "unexpected Vulkan error: {err}"),
            Self::NoSuitableGpu => write!(f, "no suitable GPU was found on the system"),
            Self::Unsupported => write!(f, "the Vulkan implementation is missing features"),
        }
    }
}

impl std::error::Error for GpuError {}
