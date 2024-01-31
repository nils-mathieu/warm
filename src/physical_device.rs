use std::sync::Arc;

use ash::vk;

use crate::Instance;

/// A physical device.
#[derive(Debug, Clone)]
pub struct PhysicalDevice {
    /// The instance that owns this physical device.
    instance: Arc<Instance>,

    /// The handle to the physical device.
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    /// Creates a new [`PhysicalDevice`] instance.
    ///
    /// # Safety
    ///
    /// The provided handle must be valid and belong to the provided instance.
    #[inline]
    pub unsafe fn from_handle(instance: Arc<Instance>, handle: vk::PhysicalDevice) -> Self {
        Self { instance, handle }
    }

    /// Returns the instance that owns this physical device.
    #[inline(always)]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    /// Returns the Vulkan handle for this physical device.
    #[inline(always)]
    pub fn vk_handle(&self) -> vk::PhysicalDevice {
        self.handle
    }
}
