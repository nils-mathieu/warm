//! Defines the [`Gpu`] type.
//!
//! More information [here](Gpu).

use std::ffi::{c_char, CStr};
use std::fmt;
use std::mem::ManuallyDrop;
use std::ptr::null_mut;
use std::sync::Arc;

use ash::vk;

use crate::utility::ScopeGuard;

mod device;
mod fns;

use self::device::DeviceQuery;
use self::fns::Fns;

mod config;
mod error;

pub use self::config::*;
pub use self::error::*;

/// The type of a graphics processing unit (GPU).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GpuType {
    /// The device is of an unknown type, not covered by any of the other types.
    Unknown = vk::PhysicalDeviceType::OTHER.as_raw(),
    /// The device is typically one embedded in or tightly coupled with the host.
    Integrated = vk::PhysicalDeviceType::INTEGRATED_GPU.as_raw(),
    /// The device is typically a separate processor connected to the host via an interlink.
    Discrete = vk::PhysicalDeviceType::DISCRETE_GPU.as_raw(),
    /// The device is typically a virtual node in a virtualization environment.
    Virtual = vk::PhysicalDeviceType::VIRTUAL_GPU.as_raw(),
    /// The device is typically running on the same processors as the host.
    Cpu = vk::PhysicalDeviceType::CPU.as_raw(),
}

/// Stores information about the GPU.
#[derive(Clone)]
pub struct GpuInfo {
    /// The name of the device.
    pub name: Box<str>,
    /// The type of the device.
    pub device_type: GpuType,
    /// A unique identifier for the vendor of the device.
    pub vendor_id: u32,
    /// A vendor-specific version number for the driver that is installed for the device.
    pub driver_version: u32,
    /// A unique identifier for the device.
    pub device_uuid: [u8; 16],
}

impl fmt::Debug for GpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uuid = u128::from_le_bytes(self.device_uuid);

        let vendor_id = self.vendor_id;
        let vendor_str = match vendor_id {
            0x1002 => "AMD",
            0x1010 => "ImgTec",
            0x10DE => "NVIDIA",
            0x13B5 => "ARM",
            0x5143 => "Qualcomm",
            0x8086 => "INTEL",
            _ => "Unknown",
        };

        f.debug_struct("GpuInfo")
            .field("name", &self.name)
            .field("device_type", &self.device_type)
            .field("vendor_id", &format_args!("{vendor_id:#x} ({vendor_str})"))
            .field("driver_version", &self.driver_version)
            .field("device_uuid", &format_args!("{uuid:#x}"))
            .finish()
    }
}

/// An open connection with a graphics processing unit (GPU).
///
/// This type represents an open connection with a GPU. It is used to submit commands to the GPU,
/// to query information about it, as well as to create GPU-local resources.
pub struct Gpu {
    /// The loaded Vulkan library.
    ///
    /// This library will remain loaded in memory for the lifetime of this field. To make the
    /// release of resources more explicit, we wrap this field in a `ManuallyDrop`.
    library: ManuallyDrop<libloading::Library>,

    /// The handle to the Vulkan instance.
    instance: vk::Instance,
    /// The handle to the Vulkan device.
    device: vk::Device,
    /// The index of the queue family that `queue` is part of.
    queue_family: u32,
    /// A queue that's suitable for graphics operations.
    queue: vk::Queue,

    /// The function pointers associated with our instance and device.
    fns: Fns,

    /// Some information about the GPU.
    info: GpuInfo,
}

impl Gpu {
    /// Creates a new [`Gpu`] instance, initiating a connection with a physical graphics processing
    /// unit and loading the Vulkan library into memory.
    pub fn new(config: GpuConfig) -> Result<Arc<Self>, GpuError> {
        let library = load_vulkan_library()?;

        let mut fns = Fns::default();
        fns._load_static_fns(&library);

        let instance = create_instance(&fns)?;
        fns._load_instance_fns(instance);
        let drop_instance = fns.instance_v1_0.destroy_instance;
        let instance = ScopeGuard::new(instance, move |i| unsafe { drop_instance(i, null_mut()) });

        let device_info = self::device::pick_physical_device(*instance, &fns, &config)?;
        let info = get_gpu_info(device_info.physical_device, &fns)?;
        let device = create_device(&device_info, &fns)?;
        let drop_device = fns.device_v1_0.destroy_device;
        fns._load_device_fns(device);
        let device = ScopeGuard::new(device, move |d| unsafe { drop_device(d, null_mut()) });

        let queue = unsafe { fns.get_device_queue(*device, device_info.queue_family, 0) };

        Ok(Arc::new(Self {
            library: ManuallyDrop::new(library),
            instance: ScopeGuard::defuse(instance),
            device: ScopeGuard::defuse(device),
            queue_family: device_info.queue_family,
            queue,
            fns,
            info,
        }))
    }

    /// Returns information about the selected GPU.
    #[inline(always)]
    pub fn info(&self) -> &GpuInfo {
        &self.info
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        unsafe {
            self.fns.destroy_device(self.device);
            self.fns.destroy_instance(self.instance);
            ManuallyDrop::drop(&mut self.library);
        }
    }
}

impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Gpu").field(&self.info).finish()
    }
}

/// Loads the Vulkan library from the default location and returns a [`libloading::Library`] handle
/// to it.
fn load_vulkan_library() -> Result<libloading::Library, GpuError> {
    #[cfg(target_os = "windows")]
    const LIBRARY_PATH: &str = "vulkan-1.dll";

    #[cfg(target_os = "linux")]
    const LIBRARY_PATH: &str = "libvulkan.so.1";

    #[cfg(target_os = "macos")]
    const LIBRARY_PATH: &str = "libvulkan.dylib";

    // SAFETY:
    //  The Vulkan library is loaded from a well-known location. If the user has a corrupted or
    //  malicious library at this location, then they have bigger problems than having this
    //  program be memory unsafe. Too bad for them :(
    let ret = unsafe { libloading::Library::new(LIBRARY_PATH) };

    match ret {
        Ok(library) => Ok(library),
        Err(_) => Err(GpuError::CantLoadVulkan),
    }
}

/// Returns the name of the project, with a null terminator.
fn get_crate_name() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "\0")
}

/// Returns the version of the crate.
fn get_crate_version() -> u32 {
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap();
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap();

    vk::make_api_version(0, major, minor, patch)
}

/// Returns the list of instance extensions to enable.
///
/// This function will attempt to enable as much extensions as possible on the current platform
/// that are required for the engine to work.
fn get_instance_extensions(fns: &Fns) -> Result<Vec<*const i8>, GpuError> {
    use ash::extensions::khr;

    const REQUIRED_EXTENSIONS: &[&CStr] = &[khr::Surface::name()];
    const WANTED_EXTENSIONS: &[&CStr] = &[khr::Win32Surface::name(), khr::XlibSurface::name()];

    let mut available = Vec::new();
    unsafe {
        fns.enumerate_instance_extension_properties(&mut available)
            .map_err(vk_to_gpu_error)?;
    }

    // Returns whether the provided extension is part of the list of available extensions.
    let has_extension = move |name: &CStr| unsafe {
        available
            .iter()
            .any(move |extension| CStr::from_ptr(extension.extension_name.as_ptr()) == name)
    };

    let mut extensions = Vec::new();
    extensions.extend(REQUIRED_EXTENSIONS.iter().map(|name| name.as_ptr()));

    for ext in WANTED_EXTENSIONS {
        if has_extension(ext) {
            extensions.push(ext.as_ptr());
        }
    }

    Ok(extensions)
}

/// Creates a Vulkan instance.
fn create_instance(fns: &Fns) -> Result<vk::Instance, GpuError> {
    let extensions = get_instance_extensions(fns)?;

    let app_info = vk::ApplicationInfo {
        api_version: vk::HEADER_VERSION_COMPLETE,
        p_engine_name: get_crate_name().as_ptr() as *const c_char,
        engine_version: get_crate_version(),
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        pp_enabled_extension_names: extensions.as_ptr(),
        enabled_extension_count: extensions.len() as u32,
        ..Default::default()
    };

    unsafe { fns.create_instance(&create_info).map_err(vk_to_gpu_error) }
}

/// Opens a connection with the specified physical device.
fn create_device(device_info: &DeviceQuery, fns: &Fns) -> Result<vk::Device, GpuError> {
    let queue_priorities = 1.0;

    let queue_create_info = vk::DeviceQueueCreateInfo {
        queue_family_index: device_info.queue_family,
        queue_count: 1,
        p_queue_priorities: &queue_priorities,
        ..Default::default()
    };

    let create_info = vk::DeviceCreateInfo {
        p_queue_create_infos: &queue_create_info,
        queue_create_info_count: 1,
        p_enabled_features: &*device_info.features,
        pp_enabled_extension_names: device_info.extensions.as_ptr(),
        enabled_extension_count: device_info.extensions.len() as u32,
        ..Default::default()
    };

    unsafe {
        fns.create_device(device_info.physical_device, &create_info)
            .map_err(vk_to_gpu_error)
    }
}

/// Returns information about the GPU.
fn get_gpu_info(physical_device: vk::PhysicalDevice, fns: &Fns) -> Result<GpuInfo, GpuError> {
    let props = unsafe { fns.get_physical_device_properties(physical_device) };
    let name_bytes = unsafe { &*(&props.device_name as *const [c_char; 256] as *const [u8; 256]) };

    let name = CStr::from_bytes_until_nul(name_bytes)
        .map_err(|_| GpuError::UnexpectedVulkanBehavior)?
        .to_str()
        .map_err(|_| GpuError::UnexpectedVulkanBehavior)?
        .into();

    let device_type = match props.device_type {
        vk::PhysicalDeviceType::OTHER => GpuType::Unknown,
        vk::PhysicalDeviceType::INTEGRATED_GPU => GpuType::Integrated,
        vk::PhysicalDeviceType::DISCRETE_GPU => GpuType::Discrete,
        vk::PhysicalDeviceType::VIRTUAL_GPU => GpuType::Virtual,
        vk::PhysicalDeviceType::CPU => GpuType::Cpu,
        _ => return Err(GpuError::UnexpectedVulkanBehavior),
    };

    Ok(GpuInfo {
        name,
        device_type,
        driver_version: props.driver_version,
        vendor_id: props.vendor_id,
        device_uuid: props.pipeline_cache_uuid,
    })
}

/// Converts a Vulkan result to a [`GpuError`].
#[inline]
fn vk_to_gpu_error(_result: vk::Result) -> GpuError {
    GpuError::UnexpectedVulkanBehavior
}
