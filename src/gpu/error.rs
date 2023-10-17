//! Defines [`GpuError`].

use std::fmt;

/// An error that might occur when creating a [`Gpu`](super::Gpu) instance.
#[derive(Debug)]
pub enum GpuError {
    /// The Vulkan dynamic library could not be loaded.
    CantLoadVulkan,
    /// The Vulkan implementation behaved in an unexpected way.
    UnexpectedVulkanBehavior,
    /// No suitable GPU was found on the system.
    NoSuitableGpu,
    /// The Vulkan implementation does not support the features required by the crate.
    Unsupported,
}

impl fmt::Display for GpuError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CantLoadVulkan => write!(f, "could not load Vulkan dynamic library"),
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
            Self::NoSuitableGpu => write!(f, "no suitable GPU was found on the system"),
            Self::Unsupported => write!(f, "the Vulkan implementation is missing features"),
        }
    }
}

impl std::error::Error for GpuError {}
