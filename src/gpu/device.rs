//! Provides functions to query and pick physical devices.

use std::ffi::{c_char, CStr};

use ash::vk;

use super::fns::Fns;
use super::{GpuConfig, GpuError};

/// Stores information about a physical device that has been picked.
pub struct DeviceQuery {
    /// The handle itself, which will be used to create a logical device.
    pub physical_device: vk::PhysicalDevice,
    /// The index of the queue family that we will be using for graphics operations.
    pub queue_family: u32,
    /// A list of extensions to enable for the logical device.
    pub extensions: Box<[*const c_char]>,
    /// A list of features to enable for the logical device.
    pub features: Box<vk::PhysicalDeviceFeatures>,
}

/// Queries the physical devices that are suitable for use with this application.
///
/// If an error occur while querying the devices, the returned iterator is empty.
pub unsafe fn query_devices<'a>(
    instance: vk::Instance,
    fns: &'a Fns,
    _config: &'a GpuConfig,
) -> impl 'a + Iterator<Item = DeviceQuery> {
    let mut devices = Vec::new();
    let ret = unsafe { fns.enumerate_physical_devices(instance, &mut devices) };
    if ret.is_err() {
        devices.clear();
    }

    devices.into_iter().filter_map(move |physical_device| {
        query_device(&QueryContext {
            fns,
            physical_device,
        })
    })
}

/// Picks the best suited physical device for this application.
pub fn pick_physical_device(
    instance: vk::Instance,
    fns: &Fns,
    config: &GpuConfig,
) -> Result<DeviceQuery, GpuError> {
    unsafe {
        query_devices(instance, fns, config)
            .next()
            .ok_or(GpuError::NoSuitableGpu)
    }
}

/// Information that's used to query a physical device.
struct QueryContext<'a> {
    /// The loaded functions associated with the instance.
    fns: &'a Fns,
    /// The physical device that's being queried.
    physical_device: vk::PhysicalDevice,
}

/// Queries information about the physical device.
///
/// If the physical device is not suitable for use with this application, returns `None`.
fn query_device(ctx: &QueryContext) -> Option<DeviceQuery> {
    let extensions = get_extensions(ctx)?;
    let features = get_features(ctx)?;
    let queue_family = get_queue_family(ctx)?;

    Some(DeviceQuery {
        physical_device: ctx.physical_device,
        queue_family,
        extensions,
        features,
    })
}

/// Returns the index of a queue family suitable for graphics operations.
///
/// If no suitable queue family is found, [`None`] is returned.
fn get_queue_family(ctx: &QueryContext) -> Option<u32> {
    let mut families = Vec::new();
    unsafe {
        ctx.fns
            .get_physical_device_queue_family_properties(ctx.physical_device, &mut families)
            .ok()?;
    }

    families
        .iter()
        .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .map(|index| index as u32)
}

/// Returns the list of extensions that should be enabled for the logical device.
///
/// If some extensions are missing, [`None`] is returned.
fn get_extensions(ctx: &QueryContext) -> Option<Box<[*const i8]>> {
    let mut available = Vec::new();
    unsafe {
        ctx.fns
            .enumerate_device_extension_properties(ctx.physical_device, &mut available)
            .ok()?;
    }

    // Returns whether any of the available extensions matches `name`.
    let is_available = |name: &CStr| unsafe {
        available
            .iter()
            .any(|ext| CStr::from_ptr(ext.extension_name.as_ptr()) == name)
    };

    const REQUIRED_EXTENSIONS: &[&CStr] = &[ash::extensions::khr::Swapchain::name()];

    for ext in REQUIRED_EXTENSIONS {
        if !is_available(ext) {
            return None;
        }
    }

    let mut extensions = Vec::new();
    extensions.extend(REQUIRED_EXTENSIONS.iter().map(|name| name.as_ptr()));

    // If we have optional extensions later, we can add them here easily.

    Some(extensions.into_boxed_slice())
}

/// Returns the list of features that should be enabled for the logical device.
///
/// If some required features are missing, [`None`] is returned.
fn get_features(_ctx: &QueryContext) -> Option<Box<vk::PhysicalDeviceFeatures>> {
    Some(Box::default())
}
