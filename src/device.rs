use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;
use bitflags::bitflags;
use smallvec::SmallVec;

use crate::{Instance, PhysicalDevice, Result};

bitflags! {
    /// A set of device extensions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DeviceExtensions: u32 {
        /// The `VK_KHR_swapchain` extension.
        const SWAPCHAIN = 1 << 0;
    }
}

impl DeviceExtensions {
    /// Returns the name of the extension.
    ///
    /// # Panics
    ///
    /// This function panics if the extension is unknown.
    pub fn name(self) -> &'static CStr {
        match self {
            Self::SWAPCHAIN => vk::KhrSwapchainFn::name(),
            _ => panic!("unknown device extension"),
        }
    }
}

/// Describes a queue family with which a connection must be established.
#[derive(Debug, Clone)]
pub struct QueueFamilyDesc<'a> {
    /// The index of the queue family.
    pub index: u32,
    /// The priority of the queues to create for this family.
    ///
    /// This is also used as the number of queues to create within that family.
    pub priorities: &'a [f32],
}

/// The parameters passed to the [`Device::new`] function in order to create a new [`Device`]
/// instance.
#[derive(Debug, Clone)]
pub struct DeviceDesc<'a> {
    /// A set of device extension that must be enabled for the device.
    pub extensions: DeviceExtensions,
    /// The queue families that must be created for the device.
    pub queue_families: &'a [QueueFamilyDesc<'a>],
}

/// A list of functions that can be called on a [`Device`] instance.
#[derive(Debug, Clone)]
pub struct DeviceFns {
    pub destroy_device: vk::PFN_vkDestroyDevice,
    pub create_swapchain: vk::PFN_vkCreateSwapchainKHR,
    pub destroy_swapchain: vk::PFN_vkDestroySwapchainKHR,
}

impl DeviceFns {
    /// # Safety
    ///
    /// The provided device must be associated with the given instance.
    unsafe fn load(instance: &Instance, device: vk::Device) -> Self {
        macro_rules! load {
            ($name:ident) => {
                ::std::mem::transmute((instance.fns().get_device_proc_addr)(
                    device,
                    concat!(stringify!($name), "\0").as_ptr() as *const i8,
                ))
            };
        }

        Self {
            destroy_device: load!(vkDestroyDevice),
            create_swapchain: load!(vkCreateSwapchainKHR),
            destroy_swapchain: load!(vkDestroySwapchainKHR),
        }
    }
}

/// An open connection to a GPU device.
pub struct Device {
    /// Creates a new [`Device`] instance.
    instance: Arc<Instance>,
    /// The handle to the device.
    handle: vk::Device,
    /// The functions that have been loaded for this device.
    fns: DeviceFns,
}

impl Device {
    /// Creates a new [`Device`] instance from the provided instance and handle.
    ///
    /// # Safety
    ///
    /// The provided handle must be valid and belong to the provided instance.
    pub unsafe fn from_handle(instance: Arc<Instance>, handle: vk::Device) -> Arc<Self> {
        Arc::new(Self {
            fns: DeviceFns::load(&instance, handle),
            instance,
            handle,
        })
    }

    /// Creates a new [`Device`].
    pub fn new(physical_device: PhysicalDevice, desc: DeviceDesc) -> Result<Arc<Self>> {
        let extensions = desc
            .extensions
            .iter()
            .map(|e| e.name().as_ptr())
            .collect::<Vec<_>>();

        let queue_create_infos = desc
            .queue_families
            .iter()
            .map(|f| vk::DeviceQueueCreateInfo {
                queue_family_index: f.index,
                queue_count: f.priorities.len() as u32,
                p_queue_priorities: f.priorities.as_ptr(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                p_next: std::ptr::null(),
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            })
            .collect::<SmallVec<[vk::DeviceQueueCreateInfo; 2]>>();

        let create_info = vk::DeviceCreateInfo {
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_enabled_features: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            p_next: std::ptr::null(),
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
        };

        let mut handle = vk::Device::null();

        let ret = unsafe {
            (physical_device.instance().fns().create_device)(
                physical_device.handle(),
                &create_info,
                std::ptr::null(),
                &mut handle,
            )
        };

        if ret != vk::Result::SUCCESS {
            return Err(ret.into());
        }

        Ok(unsafe { Self::from_handle(physical_device.instance().clone(), handle) })
    }

    /// Returns the parent [`Instance`] of this [`Device`].
    #[inline(always)]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    /// Returns the handle to the device.
    #[inline(always)]
    pub fn handle(&self) -> vk::Device {
        self.handle
    }

    /// Returns the functions that have been loaded for this device.
    #[inline(always)]
    pub fn fns(&self) -> &DeviceFns {
        &self.fns
    }
}
