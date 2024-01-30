use std::sync::Arc;

use ash::vk;

use crate::Library;

/// A collection of Vulkan functions that have been loaded for a specific [`Instance`].
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
                    concat!(stringify!($name), "\0").as_ptr() as *const std::ffi::c_char,
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
