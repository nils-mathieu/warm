use std::ffi::c_char;
use std::path::Path;
use std::sync::Arc;

use ash::vk;

use crate::Result;

/// An error that might occur while creating a [`Library`] instance.
#[derive(Debug)]
pub enum LibraryError {
    /// An error that occurred while loading the library.
    Load(libloading::Error),
    /// The Vulkan entry point was not present in the library.
    MissingEntryPoint,
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Load(err) => write!(f, "failed to load library: {}", err),
            LibraryError::MissingEntryPoint => write!(f, "missing Vulkan entry point"),
        }
    }
}

impl std::error::Error for LibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LibraryError::Load(err) => Some(err),
            LibraryError::MissingEntryPoint => None,
        }
    }
}

/// A collection of Vulkan functions that have been loaded from a loaded Vulkan library.
///
/// The function pointers in this struct are only valid as long as the library they point into
/// remains loaded in the current program's memory.
#[derive(Debug, Clone)]
pub struct LibraryFns {
    pub get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub create_instance: vk::PFN_vkCreateInstance,
}

impl LibraryFns {
    /// Creates a new [`LibraryFns`] instance from the provided `vk::PFN_vkGetInstanceProcAddr`.
    ///
    /// # Safety
    ///
    /// The provided entry point must comply with the Vulkan specification.
    unsafe fn load(ep: vk::PFN_vkGetInstanceProcAddr) -> Self {
        macro_rules! load {
            ($name:ident) => {
                ::std::mem::transmute(ep(
                    vk::Instance::null(),
                    concat!(stringify!($name), "\0").as_ptr() as *const ::std::ffi::c_char,
                ))
            };
        }

        Self {
            get_instance_proc_addr: ep,
            create_instance: load!(vkCreateInstance),
        }
    }
}

/// The Vulkan library, dynamically loaded.
///
/// As long as this struct is alive, the library is loaded into the current program's memory and
/// can be used to create static Vulkan objects and query global Vulkan properties.
///
/// This struct is meant to be used through an [`Arc<T>`] to ensure that it is not dropped while
/// still in use.
pub struct Library {
    /// The inner library object.
    ///
    /// Dropping this field will unload the library from the current program's memory.
    _inner: libloading::Library,

    /// The list of functions loaded from the library.
    fns: LibraryFns,
}

impl Library {
    /// Creates a new [`Library`] instance from the provided [`libloading::Library`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `inner` defines a valid implementation of the Vulkan API (as
    /// defined by the Vulkan specification).
    pub unsafe fn from_library(
        inner: libloading::Library,
    ) -> std::result::Result<Arc<Self>, LibraryError> {
        let ep = *inner
            .get(b"vkGetInstanceProcAddr\0")
            .map_err(|_| LibraryError::MissingEntryPoint)?;
        Ok(Arc::new(Self {
            _inner: inner,
            fns: LibraryFns::load(ep),
        }))
    }

    /// Creates a new [`Library`] instance by loading the Vulkan library located at the provided
    /// path.
    ///
    /// # Safety
    ///
    /// The caller must ensure that, if `path` points to a valid dynamic library, that library
    /// defines a valid implementation of the Vulkan API (as defined by the Vulkan specification).
    pub unsafe fn from_path(
        path: impl AsRef<Path>,
    ) -> std::result::Result<Arc<Self>, LibraryError> {
        let inner = libloading::Library::new(path.as_ref()).map_err(LibraryError::Load)?;
        Self::from_library(inner)
    }

    /// Loads a [`Library`] instance from the default Vulkan path for the current platform.
    ///
    /// # Safety considerations
    ///
    /// Loading an external library is inherently unsafe, as it can lead to arbitrary code
    /// execution. This method is not marked as 'unsafe' because as long as the user does not
    /// have a corrupted or malicious Vulkan library in the default path, this method is safe.
    ///
    /// If, by any chance, the user has a spiked library, they probably have bigger problems than
    /// this method ending up being unsafe.
    pub fn new() -> std::result::Result<Arc<Self>, LibraryError> {
        #[cfg(target_os = "linux")]
        const LIBRARY_PATH: &str = "libvulkan.so.1";

        // SAFETY: see the safety considerations of this method.
        unsafe { Self::from_path(LIBRARY_PATH) }
    }

    /// Returns the version of the underlying Vulkan implementation.
    ///
    /// # Notes
    ///
    /// This function is not available on Vulkan 1.0 (as it has only been introduced in Vulkan
    /// 1.1).
    ///
    /// When the function is not available, this function will return version 1.0.0.
    #[doc(alias = "vkEnumerateInstanceVersion")]
    pub fn enumerate_instance_version(&self) -> Result<u32> {
        let func = unsafe {
            (self.fns.get_instance_proc_addr)(
                vk::Instance::null(),
                b"vkEnumerateInstanceVersion\0".as_ptr() as *const c_char,
            )
        };

        // If the function is not available, we assume that the implementation is Vulkan 1.0.
        if func.is_none() {
            return Ok(vk::API_VERSION_1_0);
        }

        // Otherwise, we can call the function.
        let func: vk::PFN_vkEnumerateInstanceVersion = unsafe { std::mem::transmute(func) };

        let mut version = 0;
        let ret = unsafe { func(&mut version) };

        if ret == vk::Result::SUCCESS {
            Ok(version)
        } else {
            Err(ret.into())
        }
    }

    /// Returns the list of function pointers loaded from the library.
    #[inline(always)]
    pub fn fns(&self) -> &LibraryFns {
        &self.fns
    }
}
