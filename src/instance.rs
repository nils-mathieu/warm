use bitflags::bitflags;
use smallvec::SmallVec;
use std::ffi::{c_char, CStr};
use std::sync::Arc;

use ash::vk;

use crate::{Error, Library, Result};

/// The parameters passed to the [`Vulkan::new`] function.
#[derive(Debug, Clone)]
pub struct InstanceDesc<'a> {
    /// The name of the application creating the instance.
    pub application_name: &'a str,
    /// The version of the application creating the instance.
    ///
    /// The number is not encoded in any particular format.
    pub application_version: u32,
    /// The name of the engine creating the instance.
    pub engine_name: &'a str,
    /// The version of the engine creating the instance.
    ///
    /// The number is not encoded in any particular format.
    pub engine_version: u32,
    /// The list of instance extensions to enable.
    ///
    /// Note that attempting to enable an extension that is not supported by the underlying
    /// implementation will result in an error.
    pub extensions: InstanceExtensions,
}

bitflags! {
    /// A set of Vulkan extensions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InstanceExtensions: u64 {
        /// The `VK_KHR_surface` extension.
        const SURFACE = 1 << 0;
        /// The `VK_KHR_xcb_surface` extension.
        const XCB_SURFACE = 1 << 1;
        /// The `VK_KHR_xlib_surface` extension.
        const XLIB_SURFACE = 1 << 2;
        /// The `VK_KHR_wayland_surface` extension.
        const WAYLAND_SURFACE = 1 << 3;
        /// The `VK_KHR_win32_surface` extension.
        const WIN32_SURFACE = 1 << 4;
    }
}

impl InstanceExtensions {
    /// Returns the extension name of the current extension.
    ///
    /// # Panics
    ///
    /// If multiple extension bits are set, this function panics.
    pub fn name(self) -> &'static CStr {
        match self {
            Self::SURFACE => ash::extensions::khr::Surface::name(),
            Self::XCB_SURFACE => ash::extensions::khr::XcbSurface::name(),
            Self::XLIB_SURFACE => ash::extensions::khr::XlibSurface::name(),
            Self::WAYLAND_SURFACE => ash::extensions::khr::WaylandSurface::name(),
            Self::WIN32_SURFACE => ash::extensions::khr::Win32Surface::name(),
            _ => panic!("multiple extension bits are set"),
        }
    }
}

/// A collection of Vulkan functions that have been loaded for a specific [`Instance`].
#[derive(Debug)]
pub struct InstanceFns {
    destroy_instance: vk::PFN_vkDestroyInstance,
}

impl InstanceFns {
    /// Loads function pointers using the provided Vulkan entry point.
    ///
    /// # Safety
    ///
    /// The provided instance and entry point must come from the same Vulkan implementation. The
    /// handle must be valid.
    unsafe fn load(handle: vk::Instance, ep: vk::PFN_vkGetInstanceProcAddr) -> Self {
        macro_rules! load {
            ($name:ident) => {
                ::std::mem::transmute(ep(
                    handle,
                    concat!(stringify!($name), "\0").as_ptr() as *const ::std::ffi::c_char,
                ))
            };
        }

        Self {
            destroy_instance: load!(vkDestroyInstance),
        }
    }
}

/// A Vulkan instance, responsible for creating and managing Vulkan objects for the current
/// application.
///
/// This instance is used by the underlying Vulkan implementation to track the resources
/// that it uses, as well as potentially enabling certain per-application features or
/// optimizations.
pub struct Instance {
    /// The Vulkan instance handle.
    handle: vk::Instance,
    /// The functions that have been loaded for this instance.
    fns: InstanceFns,

    /// The parent library of this instance.
    library: Arc<Library>,
}

impl Instance {
    /// Creates a new [`Instance`] from the provided [`Library`] and Vulkan instance handle.
    ///
    /// # Safety
    ///
    /// The provided handle must be valid. The created [`Instance`] will take care of destroying
    /// it when it is dropped.
    pub unsafe fn from_handle(library: Arc<Library>, handle: vk::Instance) -> Arc<Self> {
        Arc::new(Self {
            handle,
            fns: InstanceFns::load(handle, library.fns().get_instance_proc_addr),

            library,
        })
    }

    /// Creates a new Vulkan instance.
    #[doc(alias = "vkCreateInstance")]
    pub fn new(library: Arc<Library>, create_info: &InstanceDesc) -> Result<Arc<Self>> {
        // Determine the name of the extensions that were requested.
        let enabled_extensions = create_info
            .extensions
            .iter()
            .map(|e| e.name().as_ptr())
            .collect::<SmallVec<[*const c_char; 4]>>();

        // Determine the API version to request.
        //
        // There's two cases:
        //
        // 1. If the `vkEnumerateInstanceVersion` function is not available, then we can assume
        //    that the current API version is 1.0.0. In that case, we request exactly that
        //    version. Any other value would cause an error.
        //
        // 2. If the `vkEnumerateInstanceVersion` function is available, then we can request
        //    the highest version of Vulkan that we know of and the underlying implementation will
        //    attempt to create an instance with that version.
        let api_version = library.enumerate_instance_version()?;
        let requested_api_version = if api_version == vk::API_VERSION_1_0 {
            vk::API_VERSION_1_0
        } else {
            vk::HEADER_VERSION_COMPLETE
        };

        let application_info = vk::ApplicationInfo {
            p_application_name: create_info.application_name.as_ptr() as *const c_char,
            application_version: create_info.application_version,
            api_version: requested_api_version,
            p_engine_name: create_info.engine_name.as_ptr() as *const c_char,
            engine_version: create_info.engine_version,

            p_next: std::ptr::null(),
            s_type: vk::StructureType::APPLICATION_INFO,
        };

        let create_info = vk::InstanceCreateInfo {
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &application_info,

            p_next: std::ptr::null(),
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        };

        let mut handle = vk::Instance::null();
        let ret =
            unsafe { (library.fns().create_instance)(&create_info, std::ptr::null(), &mut handle) };
        if ret != vk::Result::SUCCESS {
            return Err(Error::from(ret));
        }

        Ok(unsafe { Self::from_handle(library, handle) })
    }

    /// Returns a reference to the parent [`Library`] of this instance.
    #[inline(always)]
    pub fn library(&self) -> &Arc<Library> {
        &self.library
    }

    /// Returns the list of functions that have been loaded for this instance.
    #[inline(always)]
    pub fn fns(&self) -> &InstanceFns {
        &self.fns
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            (self.fns.destroy_instance)(self.handle, std::ptr::null());
        }
    }
}
