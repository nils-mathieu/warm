//! Defines [`Error`]/

use std::fmt;

use crate::VulkanError;

/// An error that might occur when interacting with a [`RenderPass`].
#[derive(Debug, Clone)]
pub enum RenderPassError {
    /// Vulkan implementation returned an unexpected error.
    UnexpectedError(VulkanError),
    /// An attachment was requested by a subpass but was not provided.
    MissingAttachment,
}

impl From<VulkanError> for RenderPassError {
    #[inline(always)]
    fn from(value: VulkanError) -> Self {
        Self::UnexpectedError(value)
    }
}

impl fmt::Display for RenderPassError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedError(err) => write!(f, "unexpected Vulkan error: {err}"),
            Self::MissingAttachment => write!(f, "an attachment was requested by a subpass but was not provided"),
        }
    }
}

impl std::error::Error for RenderPassError {}
