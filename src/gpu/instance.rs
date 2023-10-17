//! Defines the [`create`] function which helps creating a Vulkan instance.

use std::ffi::{c_char, CStr};

use ash::vk;

use super::{Extensions, Fns, GpuError};

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
fn get_instance_extensions(fns: &Fns) -> Result<(Vec<*const i8>, Extensions), GpuError> {
    use ash::extensions::khr;

    const REQUIRED_EXTENSIONS: &[(&CStr, Extensions)] =
        &[(khr::Surface::name(), Extensions::SURFACE)];
    const WANTED_EXTENSIONS: &[(&CStr, Extensions)] = &[
        (khr::Win32Surface::name(), Extensions::WIN32_SURFACE),
        (khr::XlibSurface::name(), Extensions::XLIB_SURFACE),
    ];

    let mut available = Vec::new();
    unsafe {
        match fns.enumerate_instance_extension_properties(&mut available) {
            Ok(()) => (),
            Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT) => return Err(GpuError::Unsupported),
            Err(err) => {
                return Err(GpuError::UnexpectedError(err));
            }
        }
    }

    // Returns whether the provided extension is part of the list of available extensions.
    let has_extension = move |name: &CStr| unsafe {
        available
            .iter()
            .any(move |extension| CStr::from_ptr(extension.extension_name.as_ptr()) == name)
    };

    let mut flags = Extensions::empty();
    let mut extensions = Vec::new();

    for &(ext, flag) in REQUIRED_EXTENSIONS {
        if !has_extension(ext) {
            return Err(GpuError::Unsupported);
        }

        extensions.push(ext.as_ptr());
        flags |= flag;
    }

    for &(ext, flag) in WANTED_EXTENSIONS {
        if has_extension(ext) {
            extensions.push(ext.as_ptr());
            flags |= flag;
        }
    }

    Ok((extensions, flags))
}

/// Stores information about a loaded instance.
pub struct InstanceResult {
    /// The Vulkan instance.
    pub instance: vk::Instance,
    /// The extensions that were enabled.
    pub extensions: Extensions,
}

/// Creates a Vulkan instance.
pub fn create(fns: &Fns) -> Result<InstanceResult, GpuError> {
    let (extensions, extension_flags) = get_instance_extensions(fns)?;

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

    let instance = unsafe { fns.create_instance(&create_info)? };

    Ok(InstanceResult {
        instance,
        extensions: extension_flags,
    })
}
