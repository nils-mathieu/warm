//! Defines the [`VulkanError`] type.

use ash::vk;

/// A raw Vulkan error.
///
/// This type is usually used to store an error that was not expected to occur.
pub type VulkanError = vk::Result;
