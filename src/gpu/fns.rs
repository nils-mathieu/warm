//! Defines [`Fns`].

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_void, CStr};
use std::fmt;
use std::mem::{transmute, MaybeUninit};
use std::ptr::null;

use ash::prelude::VkResult;
use ash::vk;

use crate::utility::VectorLike;

/// Contains function pointers loaded from the Vulkan dynamic library.
///
/// # Note on function pointers.
///
/// Most functions defined by the Vulkan API are not directly available within the dynamic
/// library that we load. Instead, function pointers to the functions are loaded using an
/// "entry point" function.
///
/// To save the cost of loading these functions every time we need them, we store all function
/// pointers inside of a struct.
///
/// # Function Locality
///
/// The function pointers stored here are only valid for a single and instance and device handle.
///
/// Using those pointers with a different instance or device will result in undefined behavior.
pub struct Fns {
    pub static_fn: vk::StaticFn,
    pub entry_v1_0: vk::EntryFnV1_0,
    pub instance_v1_0: vk::InstanceFnV1_0,
    pub surface: vk::KhrSurfaceFn,
    pub win32_surface: vk::KhrWin32SurfaceFn,
    pub xlib_surface: vk::KhrXlibSurfaceFn,
    pub device_v1_0: vk::DeviceFnV1_0,
    pub swapchain: vk::KhrSwapchainFn,
}

impl Fns {
    pub(crate) fn _load_static_fns(&mut self, library: &libloading::Library) {
        self.static_fn = vk::StaticFn::load(|name| symbol_to_ptr(library, name));

        let get_entry_fn = unsafe {
            let f = self.static_fn.get_instance_proc_addr;
            move |name: &CStr| transmute(f(vk::Instance::null(), name.as_ptr()))
        };

        self.entry_v1_0 = vk::EntryFnV1_0::load(get_entry_fn);
    }

    pub(crate) fn _load_instance_fns(&mut self, instance: vk::Instance) {
        let get_instance_fn = unsafe {
            let f = self.static_fn.get_instance_proc_addr;
            move |name: &CStr| transmute(f(instance, name.as_ptr()))
        };

        self.instance_v1_0 = vk::InstanceFnV1_0::load(get_instance_fn);
        self.surface = vk::KhrSurfaceFn::load(get_instance_fn);
        self.win32_surface = vk::KhrWin32SurfaceFn::load(get_instance_fn);
        self.xlib_surface = vk::KhrXlibSurfaceFn::load(get_instance_fn);
    }

    #[doc(hidden)]
    pub(crate) fn _load_device_fns(&mut self, device: vk::Device) {
        let get_device_fn = unsafe {
            let f = self.instance_v1_0.get_device_proc_addr;
            move |name: &CStr| transmute(f(device, name.as_ptr()))
        };

        self.device_v1_0 = vk::DeviceFnV1_0::load(get_device_fn);
        self.swapchain = vk::KhrSwapchainFn::load(get_device_fn);
    }

    //
    // STATIC AND ENTRY FUNCTIONS
    //

    pub unsafe fn enumerate_instance_extension_properties<C>(&self, ret: &mut C) -> VkResult<()>
    where
        C: VectorLike<Item = vk::ExtensionProperties>,
    {
        let f = self.entry_v1_0.enumerate_instance_extension_properties;
        let read = move |count, data| f(std::ptr::null(), count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn create_instance(&self, info: &vk::InstanceCreateInfo) -> VkResult<vk::Instance> {
        let mut instance = MaybeUninit::uninit();

        match (self.entry_v1_0.create_instance)(info, null(), instance.as_mut_ptr()) {
            vk::Result::SUCCESS => Ok(instance.assume_init()),
            err => Err(err),
        }
    }

    //
    // INSTANCE FUNCTIONS
    //

    pub unsafe fn enumerate_physical_devices<C>(
        &self,
        instance: vk::Instance,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::PhysicalDevice>,
    {
        let f = self.instance_v1_0.enumerate_physical_devices;
        let read = move |count, data| f(instance, count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn enumerate_device_extension_properties<C>(
        &self,
        physical_device: vk::PhysicalDevice,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::ExtensionProperties>,
    {
        let f = self.instance_v1_0.enumerate_device_extension_properties;
        let read = move |count, data| f(physical_device, std::ptr::null(), count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn get_physical_device_queue_family_properties<C>(
        &self,
        physical_device: vk::PhysicalDevice,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::QueueFamilyProperties>,
    {
        let f = self
            .instance_v1_0
            .get_physical_device_queue_family_properties;
        let read = move |count, data| {
            f(physical_device, count, data);
            vk::Result::SUCCESS
        };

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        info: &vk::DeviceCreateInfo,
    ) -> VkResult<vk::Device> {
        let mut device = MaybeUninit::uninit();
        let ret =
            (self.instance_v1_0.create_device)(physical_device, info, null(), device.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(device.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn get_physical_device_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceProperties {
        let mut properties = MaybeUninit::uninit();
        (self.instance_v1_0.get_physical_device_properties)(
            physical_device,
            properties.as_mut_ptr(),
        );
        properties.assume_init()
    }

    pub unsafe fn destroy_instance(&self, instance: vk::Instance) {
        (self.instance_v1_0.destroy_instance)(instance, null());
    }

    //
    // WIN32 SURFACE FUNCTIONS
    //

    pub unsafe fn create_win32_surface(
        &self,
        instance: vk::Instance,
        info: &vk::Win32SurfaceCreateInfoKHR,
    ) -> VkResult<vk::SurfaceKHR> {
        let mut surface = MaybeUninit::uninit();
        let ret = (self.win32_surface.create_win32_surface_khr)(
            instance,
            info,
            null(),
            surface.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok(surface.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn get_physical_device_win32_presentation_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        (self
            .win32_surface
            .get_physical_device_win32_presentation_support_khr)(
            physical_device,
            queue_family_index,
        ) != vk::FALSE
    }

    //
    // XLIB SURFACE FUNCTIONS
    //

    pub unsafe fn create_xlib_surface(
        &self,
        intance: vk::Instance,
        info: &vk::XlibSurfaceCreateInfoKHR,
    ) -> VkResult<vk::SurfaceKHR> {
        let mut surface = MaybeUninit::uninit();
        let ret = (self.xlib_surface.create_xlib_surface_khr)(
            intance,
            info,
            null(),
            surface.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok(surface.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn get_physical_device_xlib_presentation_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        display: *mut ash::vk::Display,
        visual_id: ash::vk::VisualID,
    ) -> bool {
        (self
            .xlib_surface
            .get_physical_device_xlib_presentation_support_khr)(
            physical_device,
            queue_family_index,
            display,
            visual_id,
        ) != vk::FALSE
    }

    //
    // SURFACE FUNCTIONS
    //

    pub unsafe fn destroy_surface(&self, instance: vk::Instance, surface: vk::SurfaceKHR) {
        (self.surface.destroy_surface_khr)(instance, surface, null());
    }

    pub unsafe fn get_physical_device_surface_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        let mut capabilities = MaybeUninit::uninit();
        let ret = (self.surface.get_physical_device_surface_capabilities_khr)(
            physical_device,
            surface,
            capabilities.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok(capabilities.assume_init()),
            error => Err(error),
        }
    }

    pub unsafe fn get_physical_device_surface_formats<C>(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::SurfaceFormatKHR>,
    {
        let f = self.surface.get_physical_device_surface_formats_khr;
        let read = move |count, data| f(physical_device, surface, count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn get_physical_device_surface_present_modes<C>(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::PresentModeKHR>,
    {
        let f = self.surface.get_physical_device_surface_present_modes_khr;
        let read = move |count, data| f(physical_device, surface, count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    //
    // DEVICE FUNCTIONS
    //

    pub unsafe fn get_device_queue(
        &self,
        device: vk::Device,
        family: u32,
        index: u32,
    ) -> vk::Queue {
        let mut queue = MaybeUninit::uninit();
        (self.device_v1_0.get_device_queue)(device, family, index, queue.as_mut_ptr());
        queue.assume_init()
    }

    pub unsafe fn destroy_device(&self, device: vk::Device) {
        (self.device_v1_0.destroy_device)(device, null());
    }

    //
    // SWAPCHAIN FUNCTIONS
    //

    pub unsafe fn create_swapchain(
        &self,
        device: vk::Device,
        info: &vk::SwapchainCreateInfoKHR,
    ) -> VkResult<vk::SwapchainKHR> {
        let mut swapchain = MaybeUninit::uninit();
        let ret =
            (self.swapchain.create_swapchain_khr)(device, info, null(), swapchain.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(swapchain.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_swapchain(&self, device: vk::Device, swapchain: vk::SwapchainKHR) {
        (self.swapchain.destroy_swapchain_khr)(device, swapchain, null());
    }
}

impl Default for Fns {
    fn default() -> Self {
        let fail_to_load = |_: &CStr| std::ptr::null();

        Self {
            static_fn: vk::StaticFn::load(fail_to_load),
            entry_v1_0: vk::EntryFnV1_0::load(fail_to_load),
            instance_v1_0: vk::InstanceFnV1_0::load(fail_to_load),
            surface: vk::KhrSurfaceFn::load(fail_to_load),
            win32_surface: vk::KhrWin32SurfaceFn::load(fail_to_load),
            xlib_surface: vk::KhrXlibSurfaceFn::load(fail_to_load),
            device_v1_0: vk::DeviceFnV1_0::load(fail_to_load),
            swapchain: vk::KhrSwapchainFn::load(fail_to_load),
        }
    }
}

impl fmt::Debug for Fns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fns").finish_non_exhaustive()
    }
}

/// Reads the symbol of name `symbol` from the provided library and returns a pointer to it.
fn symbol_to_ptr(library: &libloading::Library, symbol: &CStr) -> *const c_void {
    unsafe {
        match library.get(symbol.to_bytes_with_nul()) {
            Ok(ptr) => *ptr,
            Err(_) => std::ptr::null(),
        }
    }
}
