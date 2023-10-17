//! Defines [`Error`]/

use std::fmt;

/// An error that might occur when interacting with a [`RenderPass`].
#[derive(Debug, Clone)]
pub enum RenderPassError {
    /// The Vulkan API behaved unexpectedly.
    UnexpectedVulkanBehavior,
}

impl fmt::Display for RenderPassError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedVulkanBehavior => write!(f, "Vulkan implementation behaved unexpectedly"),
        }
    }
}

impl std::error::Error for RenderPassError {}
