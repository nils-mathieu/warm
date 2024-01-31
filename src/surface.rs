use std::sync::Arc;

use ash::vk;

use crate::{Instance, Result};

/// Represents a surface that can be rendered to.
pub struct Surface {
    /// The instance that owns this surface.
    instance: Arc<Instance>,

    /// The handle to the surface.
    handle: vk::SurfaceKHR,
}

impl Surface {
    /// Builds a new [`Surface`] instance from the provided handle.
    ///
    /// # Safety
    ///
    /// The provided handle must be valid and belong to the provided instance.
    pub unsafe fn from_handle(instance: Arc<Instance>, handle: vk::SurfaceKHR) -> Arc<Self> {
        Arc::new(Self { instance, handle })
    }

    /// Creates a new [`Surface`] from the provided xlib display and window.
    ///
    /// # Panics
    ///
    /// This function panics if the provided instance does not have the `VK_KHR_xlib_surface`
    pub fn from_xlib_window(
        instance: Arc<Instance>,
        dpy: *mut vk::Display,
        window: vk::Window,
    ) -> Result<Arc<Self>> {
        let create_fn = unsafe {
            (instance.library().fns().get_instance_proc_addr)(
                instance.handle(),
                b"vkCreateXlibSurfaceKHR\0".as_ptr() as *const i8,
            )
        };

        assert!(
            create_fn.is_some(),
            "the VK_KHR_xlib_surface extension is not enabled"
        );

        let create_fn =
            unsafe { std::mem::transmute::<_, vk::PFN_vkCreateXlibSurfaceKHR>(create_fn) };

        let create_info = vk::XlibSurfaceCreateInfoKHR {
            dpy,
            window,
            flags: vk::XlibSurfaceCreateFlagsKHR::empty(),

            p_next: std::ptr::null(),
            s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        };

        let mut handle = vk::SurfaceKHR::null();

        let ret = unsafe {
            create_fn(
                instance.handle(),
                &create_info,
                std::ptr::null(),
                &mut handle,
            )
        };

        if ret != vk::Result::SUCCESS {
            return Err(ret.into());
        }

        Ok(unsafe { Self::from_handle(instance, handle) })
    }

    /// Creates a new [`Surface`] from the provided xcb connection and window.
    ///
    /// # Panics
    ///
    /// This function panics if the provided instance does not have the `VK_KHR_xcb_surface`
    /// extension enabled.
    pub fn from_xcb_window(
        instance: Arc<Instance>,
        connection: *mut vk::xcb_connection_t,
        window: vk::xcb_window_t,
    ) -> Result<Arc<Self>> {
        let create_fn = unsafe {
            (instance.library().fns().get_instance_proc_addr)(
                instance.handle(),
                b"vkCreateXcbSurfaceKHR\0".as_ptr() as *const i8,
            )
        };

        assert!(
            create_fn.is_some(),
            "the VK_KHR_xcb_surface extension is not enabled"
        );

        let create_fn =
            unsafe { std::mem::transmute::<_, vk::PFN_vkCreateXcbSurfaceKHR>(create_fn) };

        let create_info = vk::XcbSurfaceCreateInfoKHR {
            connection,
            window,
            flags: vk::XcbSurfaceCreateFlagsKHR::empty(),

            p_next: std::ptr::null(),
            s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        };

        let mut handle = vk::SurfaceKHR::null();

        let ret = unsafe {
            create_fn(
                instance.handle(),
                &create_info,
                std::ptr::null(),
                &mut handle,
            )
        };

        if ret != vk::Result::SUCCESS {
            return Err(ret.into());
        }

        Ok(unsafe { Self::from_handle(instance, handle) })
    }

    /// Creates a new [`Surface`] from the provided wayland display and surface.
    ///
    /// # Panics
    ///
    /// This function panics if the provided instance does not have the `VK_KHR_wayland_surface`
    /// extension enabled.
    pub fn from_wayland_surface(
        instance: Arc<Instance>,
        display: *mut vk::wl_display,
        surface: *mut vk::wl_surface,
    ) -> Result<Arc<Self>> {
        let create_fn = unsafe {
            (instance.library().fns().get_instance_proc_addr)(
                instance.handle(),
                b"vkCreateWaylandSurfaceKHR\0".as_ptr() as *const i8,
            )
        };

        assert!(
            create_fn.is_some(),
            "the VK_KHR_wayland_surface extension is not enabled"
        );

        let create_fn =
            unsafe { std::mem::transmute::<_, vk::PFN_vkCreateWaylandSurfaceKHR>(create_fn) };

        let create_info = vk::WaylandSurfaceCreateInfoKHR {
            display,
            surface,
            flags: vk::WaylandSurfaceCreateFlagsKHR::empty(),

            p_next: std::ptr::null(),
            s_type: vk::StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
        };

        let mut handle = vk::SurfaceKHR::null();

        let ret = unsafe {
            create_fn(
                instance.handle(),
                &create_info,
                std::ptr::null(),
                &mut handle,
            )
        };

        if ret != vk::Result::SUCCESS {
            return Err(ret.into());
        }

        Ok(unsafe { Self::from_handle(instance, handle) })
    }

    /// Creates a new [`Surface`] from the provided Win32 instance and window.
    ///
    /// # Panics
    ///
    /// This function panics if the provided instance does not have the `VK_KHR_win32_surface`
    /// extension enabled.
    pub fn from_win32_window(
        instance: Arc<Instance>,
        hinstance: vk::HINSTANCE,
        hwnd: vk::HWND,
    ) -> Result<Arc<Self>> {
        let create_fn = unsafe {
            (instance.library().fns().get_instance_proc_addr)(
                instance.handle(),
                b"vkCreateWin32SurfaceKHR\0".as_ptr() as *const i8,
            )
        };

        assert!(
            create_fn.is_some(),
            "the VK_KHR_win32_surface extension is not enabled"
        );

        let create_fn =
            unsafe { std::mem::transmute::<_, vk::PFN_vkCreateWin32SurfaceKHR>(create_fn) };

        let create_info = vk::Win32SurfaceCreateInfoKHR {
            hinstance,
            hwnd,
            flags: vk::Win32SurfaceCreateFlagsKHR::empty(),

            p_next: std::ptr::null(),
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        };

        let mut handle = vk::SurfaceKHR::null();

        let ret = unsafe {
            create_fn(
                instance.handle(),
                &create_info,
                std::ptr::null(),
                &mut handle,
            )
        };

        if ret != vk::Result::SUCCESS {
            return Err(ret.into());
        }

        Ok(unsafe { Self::from_handle(instance, handle) })
    }

    /// Creates a new [`Surface`] from the provided raw window handle.
    #[cfg(feature = "raw-window-handle")]
    pub fn from_raw_window_handle(
        instance: Arc<Instance>,
        rwh: raw_window_handle::RawWindowHandle,
        rdh: raw_window_handle::RawDisplayHandle,
    ) -> Result<Arc<Self>> {
        use raw_window_handle::RawDisplayHandle as Rdh;
        use raw_window_handle::RawWindowHandle as Rwh;
        use std::num::NonZeroIsize;
        use std::ptr::NonNull;

        match (rdh, rwh) {
            (Rdh::Windows(_), Rwh::Win32(handle)) => Self::from_win32_window(
                instance,
                handle.hinstance.map_or(0, NonZeroIsize::get) as _,
                handle.hwnd.get() as _,
            ),
            (Rdh::Xlib(display), Rwh::Xlib(handle)) => Self::from_xlib_window(
                instance,
                display
                    .display
                    .map_or(core::ptr::null_mut(), NonNull::as_ptr) as _,
                handle.window as _,
            ),
            (Rdh::Xcb(connection), Rwh::Xcb(handle)) => Self::from_xcb_window(
                instance,
                connection
                    .connection
                    .map_or(core::ptr::null_mut(), NonNull::as_ptr),
                handle.window.get(),
            ),
            (Rdh::Wayland(display), Rwh::Wayland(handle)) => Self::from_wayland_surface(
                instance,
                display.display.as_ptr(),
                handle.surface.as_ptr(),
            ),
            _ => panic!("unsupported raw window handle"),
        }
    }

    /// Creates a new [`Surface`] from the provided window.
    ///
    /// # Panics
    ///
    /// This function panics if the raw window handle or raw display handle of the window
    /// cannot be retrieved.
    #[cfg(feature = "raw-window-handle")]
    pub fn from_window<W>(instance: Arc<Instance>, window: &W) -> Result<Arc<Self>>
    where
        W: raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle + ?Sized,
    {
        Self::from_raw_window_handle(
            instance,
            window.window_handle().unwrap().as_raw(),
            window.display_handle().unwrap().as_raw(),
        )
    }

    /// Returns the instance that owns this surface.
    #[inline(always)]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    /// Returns the handle to the surface.
    #[inline(always)]
    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            (self.instance.fns().destroy_surface)(
                self.instance.handle(),
                self.handle,
                std::ptr::null(),
            );
        }
    }
}
