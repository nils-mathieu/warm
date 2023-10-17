//! Functions to create a surface from a window handle.

use ash::vk;

use raw_window_handle::{DisplayHandle, RawDisplayHandle};

use super::{SurfaceError, SurfaceTarget};
use crate::gpu::{Extensions, Gpu};

/// Returns the extensions required by the surface.
///
/// If the surface is not supported by the provided surface, [`None`] is returned.
fn required_extensions(disp: DisplayHandle) -> Option<Extensions> {
    match disp.as_raw() {
        RawDisplayHandle::Windows(_) => Some(Extensions::SURFACE | Extensions::WIN32_SURFACE),
        RawDisplayHandle::Xlib(_) => Some(Extensions::SURFACE | Extensions::XLIB_SURFACE),
        _ => None,
    }
}

/// Creates a surface for the provided surface target.
pub fn create_surface(
    gpu: &Gpu,
    surface: &dyn SurfaceTarget,
) -> Result<vk::SurfaceKHR, SurfaceError> {
    use raw_window_handle::{RawDisplayHandle as Rdh, RawWindowHandle as Rwh};

    let disp = surface
        .display_handle()
        .unwrap_or_else(|_| surface_not_supported());
    let win = surface
        .window_handle()
        .unwrap_or_else(|_| surface_not_supported());

    let required_extensions = required_extensions(disp).unwrap_or_else(|| surface_not_supported());
    if !gpu.extensions().contains(required_extensions) {
        return Err(SurfaceError::NotSupported)?;
    }

    match (disp.as_raw(), win.as_raw()) {
        (Rdh::Windows(disp), Rwh::Win32(win)) => create_win32_surface(gpu, disp, win),
        (Rdh::Xlib(disp), Rwh::Xlib(win)) => create_xlib_surface(gpu, disp, win),
        _ => surface_not_supported(),
    }
}

/// Creates a Win32 surface.
fn create_win32_surface(
    gpu: &Gpu,
    _disp: raw_window_handle::WindowsDisplayHandle,
    win: raw_window_handle::Win32WindowHandle,
) -> Result<vk::SurfaceKHR, SurfaceError> {
    unsafe {
        if !gpu.vk_fns().get_physical_device_win32_presentation_support(
            gpu.vk_physical_device(),
            gpu.vk_queue_family(),
        ) {
            return Err(SurfaceError::NotSupported)?;
        }

        let hinstance = match win.hinstance {
            Some(val) => val.get() as vk::HINSTANCE,
            None => std::ptr::null(),
        };

        let hwnd = win.hwnd.get() as vk::HWND;

        let info = vk::Win32SurfaceCreateInfoKHR {
            hinstance,
            hwnd,
            ..Default::default()
        };

        let surface = gpu
            .vk_fns()
            .create_win32_surface(gpu.vk_instance(), &info)
            .map_err(|_| SurfaceError::UnexpectedVulkanBehavior)?;

        Ok(surface)
    }
}

/// Creates an Xlib surface.
fn create_xlib_surface(
    gpu: &Gpu,
    disp: raw_window_handle::XlibDisplayHandle,
    win: raw_window_handle::XlibWindowHandle,
) -> Result<vk::SurfaceKHR, SurfaceError> {
    unsafe {
        let display = match disp.display {
            Some(val) => val.as_ptr() as *mut vk::Display,
            None => std::ptr::null_mut(),
        };

        if !gpu.vk_fns().get_physical_device_xlib_presentation_support(
            gpu.vk_physical_device(),
            gpu.vk_queue_family(),
            display,
            win.visual_id,
        ) {
            return Err(SurfaceError::NotSupported)?;
        }

        let info = vk::XlibSurfaceCreateInfoKHR {
            dpy: display,
            window: win.window,
            ..Default::default()
        };

        let surface = gpu
            .vk_fns()
            .create_xlib_surface(gpu.vk_instance(), &info)
            .map_err(|_| SurfaceError::UnexpectedVulkanBehavior)?;

        Ok(surface)
    }
}

/// Panics with a message indicating that the provided surface is not supported.
#[cold]
#[track_caller]
#[inline(never)]
fn surface_not_supported() -> ! {
    panic!("the windowing system of the provided surface is not supported by `warm`")
}
