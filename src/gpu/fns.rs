//! Defines [`Fns`].

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_void, CStr};
use std::mem::{transmute, MaybeUninit};
use std::ptr::null_mut;

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
    pub device_v1_0: vk::DeviceFnV1_0,
}

impl Fns {
    pub fn _load_static_fns(&mut self, library: &libloading::Library) {
        self.static_fn = vk::StaticFn::load(|name| symbol_to_ptr(library, name));

        let get_entry_fn = unsafe {
            let f = self.static_fn.get_instance_proc_addr;
            move |name: &CStr| transmute(f(vk::Instance::null(), name.as_ptr()))
        };

        self.entry_v1_0 = vk::EntryFnV1_0::load(get_entry_fn);
    }

    pub fn _load_instance_fns(&mut self, instance: vk::Instance) {
        let get_instance_fn = unsafe {
            let f = self.static_fn.get_instance_proc_addr;
            move |name: &CStr| transmute(f(instance, name.as_ptr()))
        };

        self.instance_v1_0 = vk::InstanceFnV1_0::load(get_instance_fn);
    }

    pub fn _load_device_fns(&mut self, device: vk::Device) {
        let get_device_fn = unsafe {
            let f = self.instance_v1_0.get_device_proc_addr;
            move |name: &CStr| transmute(f(device, name.as_ptr()))
        };

        self.device_v1_0 = vk::DeviceFnV1_0::load(get_device_fn);
    }

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

        match (self.entry_v1_0.create_instance)(info, null_mut(), instance.as_mut_ptr()) {
            vk::Result::SUCCESS => Ok(instance.assume_init()),
            err => Err(err),
        }
    }

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
        match (self.instance_v1_0.create_device)(
            physical_device,
            info,
            null_mut(),
            device.as_mut_ptr(),
        ) {
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
        (self.instance_v1_0.destroy_instance)(instance, null_mut());
    }

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
        (self.device_v1_0.destroy_device)(device, null_mut());
    }
}

impl Default for Fns {
    fn default() -> Self {
        let fail_to_load = |_: &CStr| std::ptr::null();

        Self {
            static_fn: vk::StaticFn::load(fail_to_load),
            entry_v1_0: vk::EntryFnV1_0::load(fail_to_load),
            instance_v1_0: vk::InstanceFnV1_0::load(fail_to_load),
            device_v1_0: vk::DeviceFnV1_0::load(fail_to_load),
        }
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
