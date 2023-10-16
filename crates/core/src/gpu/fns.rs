//! Defines [`Fns`].

#![allow(unsafe_op_in_unsafe_fn, missing_docs, clippy::missing_safety_doc)]

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

    pub unsafe fn device_wait_idle(&self, device: vk::Device) -> VkResult<()> {
        match (self.device_v1_0.device_wait_idle)(device) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
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
        (self.device_v1_0.destroy_device)(device, null());
    }

    pub unsafe fn create_semaphore(
        &self,
        device: vk::Device,
        info: &vk::SemaphoreCreateInfo,
    ) -> VkResult<vk::Semaphore> {
        let mut semaphore = MaybeUninit::uninit();
        let ret = (self.device_v1_0.create_semaphore)(device, info, null(), semaphore.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(semaphore.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_semaphore(&self, device: vk::Device, semaphore: vk::Semaphore) {
        (self.device_v1_0.destroy_semaphore)(device, semaphore, null());
    }

    pub unsafe fn create_fence(
        &self,
        device: vk::Device,
        info: &vk::FenceCreateInfo,
    ) -> VkResult<vk::Fence> {
        let mut fence = MaybeUninit::uninit();
        let ret = (self.device_v1_0.create_fence)(device, info, null(), fence.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(fence.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_fence(&self, device: vk::Device, fence: vk::Fence) {
        (self.device_v1_0.destroy_fence)(device, fence, null());
    }

    pub unsafe fn create_command_pool(
        &self,
        device: vk::Device,
        info: &vk::CommandPoolCreateInfo,
    ) -> VkResult<vk::CommandPool> {
        let mut command_pool = MaybeUninit::uninit();
        let ret =
            (self.device_v1_0.create_command_pool)(device, info, null(), command_pool.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(command_pool.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_command_pool(&self, device: vk::Device, command_pool: vk::CommandPool) {
        (self.device_v1_0.destroy_command_pool)(device, command_pool, null());
    }

    pub unsafe fn allocate_command_buffers(
        &self,
        device: vk::Device,
        info: &vk::CommandBufferAllocateInfo,
        output: *mut vk::CommandBuffer,
    ) -> VkResult<()> {
        match (self.device_v1_0.allocate_command_buffers)(device, info, output) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn free_command_buffers(
        &self,
        device: vk::Device,
        command_pool: vk::CommandPool,
        buffers: &[vk::CommandBuffer],
    ) {
        (self.device_v1_0.free_command_buffers)(
            device,
            command_pool,
            buffers.len() as u32,
            buffers.as_ptr(),
        );
    }

    pub unsafe fn wait_for_fences(
        &self,
        device: vk::Device,
        fences: &[vk::Fence],
        wait_all: bool,
        timeout: u64,
    ) -> VkResult<()> {
        match (self.device_v1_0.wait_for_fences)(
            device,
            fences.len() as u32,
            fences.as_ptr(),
            wait_all as u32,
            timeout,
        ) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn reset_fences(&self, device: vk::Device, fences: &[vk::Fence]) -> VkResult<()> {
        match (self.device_v1_0.reset_fences)(device, fences.len() as u32, fences.as_ptr()) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn create_render_pass(
        &self,
        device: vk::Device,
        info: &vk::RenderPassCreateInfo,
    ) -> VkResult<vk::RenderPass> {
        let mut render_pass = MaybeUninit::uninit();
        let ret =
            (self.device_v1_0.create_render_pass)(device, info, null(), render_pass.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(render_pass.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_render_pass(&self, device: vk::Device, render_pass: vk::RenderPass) {
        (self.device_v1_0.destroy_render_pass)(device, render_pass, null());
    }

    pub unsafe fn create_image_view(
        &self,
        device: vk::Device,
        info: &vk::ImageViewCreateInfo,
    ) -> VkResult<vk::ImageView> {
        let mut image_view = MaybeUninit::uninit();
        let ret =
            (self.device_v1_0.create_image_view)(device, info, null(), image_view.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(image_view.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_image_view(&self, device: vk::Device, image_view: vk::ImageView) {
        (self.device_v1_0.destroy_image_view)(device, image_view, null());
    }

    pub unsafe fn create_framebuffer(
        &self,
        device: vk::Device,
        info: &vk::FramebufferCreateInfo,
    ) -> VkResult<vk::Framebuffer> {
        let mut framebuffer = MaybeUninit::uninit();
        let ret =
            (self.device_v1_0.create_framebuffer)(device, info, null(), framebuffer.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(framebuffer.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_framebuffer(&self, device: vk::Device, framebuffer: vk::Framebuffer) {
        (self.device_v1_0.destroy_framebuffer)(device, framebuffer, null());
    }

    pub unsafe fn create_shader_module(
        &self,
        device: vk::Device,
        info: &vk::ShaderModuleCreateInfo,
    ) -> VkResult<vk::ShaderModule> {
        let mut shader_module = MaybeUninit::uninit();
        let ret = (self.device_v1_0.create_shader_module)(
            device,
            info,
            null(),
            shader_module.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok(shader_module.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_shader_module(
        &self,
        device: vk::Device,
        shader_module: vk::ShaderModule,
    ) {
        (self.device_v1_0.destroy_shader_module)(device, shader_module, null());
    }

    pub unsafe fn create_pipeline_layout(
        &self,
        device: vk::Device,
        info: &vk::PipelineLayoutCreateInfo,
    ) -> VkResult<vk::PipelineLayout> {
        let mut pipeline_layout = MaybeUninit::uninit();
        let ret = (self.device_v1_0.create_pipeline_layout)(
            device,
            info,
            null(),
            pipeline_layout.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok(pipeline_layout.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_pipeline_layout(
        &self,
        device: vk::Device,
        pipeline_layout: vk::PipelineLayout,
    ) {
        (self.device_v1_0.destroy_pipeline_layout)(device, pipeline_layout, null());
    }

    pub unsafe fn create_graphics_pipelines<C>(
        &self,
        device: vk::Device,
        cache: vk::PipelineCache,
        infos: &[vk::GraphicsPipelineCreateInfo],
        ret: *mut vk::Pipeline,
    ) -> VkResult<()> {
        match (self.device_v1_0.create_graphics_pipelines)(
            device,
            cache,
            infos.len() as u32,
            infos.as_ptr(),
            null(),
            ret,
        ) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_pipeline(&self, device: vk::Device, pipeline: vk::Pipeline) {
        (self.device_v1_0.destroy_pipeline)(device, pipeline, null());
    }

    pub unsafe fn create_buffer(
        &self,
        device: vk::Device,
        info: &vk::BufferCreateInfo,
    ) -> VkResult<vk::Buffer> {
        let mut buffer = MaybeUninit::uninit();
        let ret = (self.device_v1_0.create_buffer)(device, info, null(), buffer.as_mut_ptr());

        match ret {
            vk::Result::SUCCESS => Ok(buffer.assume_init()),
            err => Err(err),
        }
    }

    pub unsafe fn destroy_buffer(&self, device: vk::Device, buffer: vk::Buffer) {
        (self.device_v1_0.destroy_buffer)(device, buffer, null());
    }

    pub unsafe fn get_buffer_memory_requirements(
        &self,
        device: vk::Device,
        buffer: vk::Buffer,
    ) -> vk::MemoryRequirements {
        let mut requirements = MaybeUninit::uninit();
        (self.device_v1_0.get_buffer_memory_requirements)(
            device,
            buffer,
            requirements.as_mut_ptr(),
        );
        requirements.assume_init()
    }

    //
    // COMMAND BUFFER FUNCTIONS
    //

    pub unsafe fn reset_command_buffer(
        &self,
        buffer: vk::CommandBuffer,
        flags: vk::CommandBufferResetFlags,
    ) -> VkResult<()> {
        match (self.device_v1_0.reset_command_buffer)(buffer, flags) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn begin_command_buffer(
        &self,
        buffer: vk::CommandBuffer,
        info: &vk::CommandBufferBeginInfo,
    ) -> VkResult<()> {
        match (self.device_v1_0.begin_command_buffer)(buffer, info) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn end_command_buffer(&self, buffer: vk::CommandBuffer) -> VkResult<()> {
        match (self.device_v1_0.end_command_buffer)(buffer) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn cmd_pipeline_barrier(
        &self,
        buffer: vk::CommandBuffer,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        dependencies: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        (self.device_v1_0.cmd_pipeline_barrier)(
            buffer,
            src_stage,
            dst_stage,
            dependencies,
            memory_barriers.len() as u32,
            memory_barriers.as_ptr(),
            buffer_memory_barriers.len() as u32,
            buffer_memory_barriers.as_ptr(),
            image_memory_barriers.len() as u32,
            image_memory_barriers.as_ptr(),
        );
    }

    //
    // QUEUE FUNCTIONS
    //

    pub unsafe fn queue_submit(
        &self,
        queue: vk::Queue,
        info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> VkResult<()> {
        match (self.device_v1_0.queue_submit)(queue, info.len() as u32, info.as_ptr(), fence) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn queue_wait_idle(&self, queue: vk::Queue) -> VkResult<()> {
        match (self.device_v1_0.queue_wait_idle)(queue) {
            vk::Result::SUCCESS => Ok(()),
            err => Err(err),
        }
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

    pub unsafe fn acquire_next_image(
        &self,
        device: vk::Device,
        swapchain: vk::SwapchainKHR,
        timeout: u64,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> VkResult<(u32, bool)> {
        let mut image_index = MaybeUninit::uninit();
        let ret = (self.swapchain.acquire_next_image_khr)(
            device,
            swapchain,
            timeout,
            semaphore,
            fence,
            image_index.as_mut_ptr(),
        );

        match ret {
            vk::Result::SUCCESS => Ok((image_index.assume_init(), true)),
            vk::Result::SUBOPTIMAL_KHR => Ok((image_index.assume_init(), false)),
            error => Err(error),
        }
    }

    pub unsafe fn queue_present(
        &self,
        queue: vk::Queue,
        info: &vk::PresentInfoKHR,
    ) -> VkResult<()> {
        match (self.swapchain.queue_present_khr)(queue, info) {
            vk::Result::SUCCESS => Ok(()),
            error => Err(error),
        }
    }

    pub unsafe fn get_swapchain_images<C>(
        &self,
        device: vk::Device,
        swapchain: vk::SwapchainKHR,
        ret: &mut C,
    ) -> VkResult<()>
    where
        C: VectorLike<Item = vk::Image>,
    {
        let f = self.swapchain.get_swapchain_images_khr;
        let read = move |count, data| f(device, swapchain, count, data);

        match crate::utility::read_into_vector(ret, read) {
            vk::Result::SUCCESS => Ok(()),
            error => Err(error),
        }
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
