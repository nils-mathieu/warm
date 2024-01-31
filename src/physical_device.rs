use std::ffi::CStr;
use std::mem::MaybeUninit;
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

    /// Returns information about this [`PhysicalDevice`].
    #[doc(alias = "vkGetPhysicalDeviceProperties")]
    pub fn properties(&self) -> PhysicalDeviceInfo {
        let mut properties = MaybeUninit::<vk::PhysicalDeviceProperties>::uninit();

        unsafe {
            (self.instance.fns().get_physical_device_properties)(
                self.handle,
                properties.as_mut_ptr(),
            );
        }

        let properties = unsafe { properties.assume_init_ref() };

        let name_cstr = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
        let name = name_cstr.to_str().unwrap_or_default().into();

        PhysicalDeviceInfo {
            api_version: properties.api_version,
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: match properties.device_type {
                vk::PhysicalDeviceType::OTHER => DeviceType::Other,
                vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGpu,
                vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGpu,
                vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGpu,
                vk::PhysicalDeviceType::CPU => DeviceType::Cpu,
                unknown => unreachable!("unknown device type: {}", unknown.as_raw()),
            },
            name,
        }
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

/// Stores information about a physical device.
#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    /// The name of the device.
    pub name: Box<str>,
    /// The version of the Vulkan API that the device implements.
    pub api_version: u32,
    /// The version of the driver that the device uses.
    pub driver_version: u32,
    /// The ID of the vendor that created the device.
    pub vendor_id: u32,
    /// The ID of the device.
    pub device_id: u32,
    /// The type of the device.
    pub device_type: DeviceType,
}

/// The type of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// The type of the device is unknown.
    Other,
    /// The GPU is integrated or tightly coupled with the host CPU.
    IntegratedGpu,
    /// The GPU is typically a separate processor connected to the host via an interlink.
    DiscreteGpu,
    /// The GPU is a virtual node in a virtualization environment.
    VirtualGpu,
    /// The device is running on the same processor as the host.
    Cpu,
}
