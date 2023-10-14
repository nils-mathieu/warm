//! Defines [`Surface`].

use std::fmt;
use std::ptr::null;
use std::sync::Arc;

use ash::vk;
use bitflags::bitflags;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::gpu::Gpu;
use crate::utility::ScopeGuard;

mod error;
mod surface_contents;

pub use self::error::*;
pub use self::surface_contents::*;

mod semaphore_pool;
mod swapchain_info;
mod window;

use self::semaphore_pool::SemaphorePool;
use self::swapchain_info::SwapchainInfo;

/// A trait for surfaces on which we can render.
pub trait SurfaceTarget: HasWindowHandle + HasDisplayHandle {}
impl<T: HasWindowHandle + HasDisplayHandle> SurfaceTarget for T {}

/// A presentation mode that can be used for a [`Surface`].
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PresentMode {
    /// The presentation engine won't wait for the vertical blanking period of the display to
    /// update the current image.
    ///
    /// This *may* result in visible tearing.
    ///
    /// No queuing operation are performed by the presentation engine as requests are
    /// immediately processed.
    Immediate = vk::PresentModeKHR::IMMEDIATE.as_raw() as _,

    /// The presentation engine waits for the next vertical blanking period of the display to
    /// update the current image.
    ///
    /// Tearing *cannot* be observed.
    ///
    /// An internal single-entry queue is kept to hold the latest presentation request. If the
    /// queue is full when a new presentation request is received, the new request replaces the
    /// existing entry and resources associated with the existing entry become available for
    /// re-use by the application.
    ///
    /// On each vertical blanking period of the display, the presentation engine removes the
    /// entry at the front of the queue and uses it to update the current image.
    Mailbox = vk::PresentModeKHR::MAILBOX.as_raw() as _,

    /// The presentation engine waits for the next vertical blanking period of the display to
    /// update the current image.
    ///
    /// Tearing *cannot* be observed.
    ///
    /// An internal queue is used to hold pending presentation requests. New requests are
    /// appended to the end of the queue, and one request is removed from the beginning of the
    /// queue and processed during each vertical blanking period in which the queue is
    /// non-empty.
    Fifo = vk::PresentModeKHR::FIFO.as_raw() as _,

    /// The presentation engine generally behaves like [`Fifo`](Self::Fifo), except when the
    /// queue becomes empty *before* the next vertical blanking period. In that case, the
    /// presentation engine will immediately update the current image without waiting.
    ///
    /// This *may* result in visible tearing in that case.
    ///
    /// New requests are appended to the end of the queue, and one is request is removed from
    /// the beginning of the queue and processed during each vertical blanking period in which
    /// the queue is non-empty.
    FifoRelaxed = vk::PresentModeKHR::FIFO_RELAXED.as_raw() as _,
}

bitflags! {
    /// A set of [`PresentMode`]s.
    #[derive(Debug, Clone, Copy)]
    pub struct PresentModes: u32 {
        /// See [`PresentMode::Immediate`].
        const IMMEDIATE = 1 << PresentMode::Immediate as u32;
        /// See [`PresentMode::Mailbox`].
        const MAILBOX = 1 << PresentMode::Mailbox as u32;
        /// See [`PresentMode::Fifo`].
        const FIFO = 1 << PresentMode::Fifo as u32;
        /// See [`PresentMode::FifoRelaxed`].
        const FIFO_RELAXED = 1 << PresentMode::FifoRelaxed as u32;
    }
}

impl From<PresentMode> for PresentModes {
    #[inline]
    fn from(value: PresentMode) -> Self {
        Self::from_bits_retain(1 << value as u32)
    }
}

/// The surface configuration passed to [`Surface::configure`] when re-configuring the surface
/// for a new swapchain.
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    /// The width of the surface.
    pub width: u32,
    /// The height of the surface.
    pub height: u32,
    /// The presentation mode used by presentation engine.
    pub present_mode: PresentMode,
}

/// Stores information about the capabilities of the surface.
///
/// This information may be used to ensure that the configuration sent to the surface using
/// [`SurfaceConfig`] and [`Surface::configure`] are valid.
#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    /// Returns the maximum size of the surface.
    pub max_size: Option<(u32, u32)>,
    /// Returns the minimum size of the surface, if any.
    pub min_size: (u32, u32),
    /// Returns the present modes supported by the surface.
    pub present_modes: PresentModes,
}

impl SurfaceCapabilities {
    /// Returns whether the provided size is valid for the surface.
    pub fn is_size_valid(&self, width: u32, height: u32) -> bool {
        if width < self.min_size.0 || height < self.min_size.1 {
            return false;
        }

        if let Some((max_width, max_height)) = self.max_size {
            if width > max_width || height > max_height {
                return false;
            }
        }

        true
    }

    /// Returns whether the provided present mode is supported by the surface.
    #[inline(always)]
    pub fn is_present_mode_valid(&self, present_mode: PresentMode) -> bool {
        self.present_modes.contains(present_mode.into())
    }

    /// Returns whether the provided configuration is valid for the surface.
    pub fn is_config_valid(&self, config: &SurfaceConfig) -> bool {
        config.width > 0
            && config.height > 0
            && self.is_size_valid(config.width, config.height)
            && self.is_present_mode_valid(config.present_mode)
    }
}

/// A Vulkan renderer.
pub struct Surface {
    /// The GPU that the renderer is using.
    gpu: Arc<Gpu>,

    /// The surface on which we are rendering.
    surface: vk::SurfaceKHR,

    /// The swapchain that's currently in use.
    ///
    /// # Retired Swapchains
    ///
    /// The swapchain may be in the "retired" state. In that case, this field will have the value
    /// `vk::SwapchainKHR::null()`.
    ///
    /// If the swapchain recreation operation fails, the swapchain will still be destroyed and
    /// the renderer will be in an unusable state. In that case, it is no longer possible
    /// to render to the surface and the swapchain has to be created again.
    swapchain: vk::SwapchainKHR,
    /// The images that were created for the swapchain.
    images: Vec<vk::Image>,
    /// A pool of semaphores to use when acquiring swapchain images.
    semaphore_pool: SemaphorePool,

    /// Information about the swapchain.
    info: SwapchainInfo,
    /// The current configuration of the surface.
    config: SurfaceConfig,

    /// A list of semaphores that the present operation should wait on. Those semaphores are *not*
    /// managed by us and must be destroyed by a [`SurfaceContents`] implementation.
    ///
    /// This vector is kept here to avoid having to re-allocate it every time we present a frame.
    present_wait_semaphores: Vec<vk::Semaphore>,
}

impl Surface {
    /// Creates a new [`Surface`].
    pub fn new(gpu: Arc<Gpu>, target: &dyn SurfaceTarget) -> Result<Self, SurfaceError> {
        let surface = self::window::create_surface(&gpu, target)?;
        let drop_surface = gpu.vk_fns().surface.destroy_surface_khr;
        let i = gpu.vk_instance();
        let surface = ScopeGuard::new(surface, move |s| unsafe { drop_surface(i, s, null()) });

        let info = self::swapchain_info::query(&gpu, *surface)?;

        Ok(Self {
            gpu,
            surface: ScopeGuard::defuse(surface),
            swapchain: vk::SwapchainKHR::null(),
            images: Vec::new(),
            semaphore_pool: SemaphorePool::default(),
            info,
            config: SurfaceConfig {
                width: 0,
                height: 0,
                present_mode: PresentMode::Fifo,
            },

            present_wait_semaphores: Vec::new(),
        })
    }

    /// Returns whether the underlying swapchain is valid.
    ///
    /// If this function returns `true`, the swapchain is currently valid and can be used
    /// to render to the surface.
    ///
    /// If this function returns `false`, the swapchain is retired and the surface cannot be
    /// rendered to anymore.
    ///
    /// More information in the documentation of [`Surface::configure`].
    #[inline(always)]
    pub fn is_swapchain_valid(&self) -> bool {
        self.swapchain != vk::SwapchainKHR::null()
    }

    /// Returns the capabilities of the provided surface.
    ///
    /// # Notes
    ///
    /// Most of the fields of the returned [`SurfaceCapabilities`] instance are guaranteed to
    /// remain constant for the lifetime of the surface.
    ///
    /// However, some may change when the underlying surface target changes.
    ///
    /// - `min_size` and `max_size` may change when the surface is resized.
    pub fn capabilities(&self) -> Result<SurfaceCapabilities, SurfaceError> {
        let caps = get_surface_capabilities(&self.gpu, self.surface)?;

        let max_size = if caps.max_image_extent.width == 0 || caps.max_image_extent.height == 0 {
            None
        } else {
            Some((caps.max_image_extent.width, caps.max_image_extent.height))
        };

        let min_size = (caps.min_image_extent.width, caps.min_image_extent.height);

        Ok(SurfaceCapabilities {
            max_size,
            min_size,
            present_modes: self.info.present_modes,
        })
    }

    /// Re-configures the surface. This function must be called every time the surface is resized.
    ///
    /// # Swapchain Retirement
    ///
    /// In some cases, the swapchain managed by this [`Surface`] instance may fail to be recreated
    /// by this function. In that case, it is destroyed and cannot be used anymore. When that
    /// happens, it is no longer possible to render to the surface and the swapchain has to be
    /// created again.
    ///
    /// You can check whether the swapchain is currently retired by calling
    /// the [`Surface::is_swapchain_valid`] function.
    ///
    /// Hopefully, it's only an edge case that developers should not have to worry too much about.
    /// Specifically, it is probably legitimate to simply `panic!` if surface cannot be
    /// re-configured for some reason.
    pub fn configure(&mut self, config: SurfaceConfig) -> Result<(), SurfaceError> {
        if !self.capabilities()?.is_config_valid(&config) {
            return Err(SurfaceError::InvalidConfig);
        }

        unsafe { self.configure_unchecked(config) }
    }

    /// Re-configures the surface. This function must be called every time the surface is resized.
    ///
    /// More information can be found in [`Surface::configure`].
    ///
    /// # Safety
    ///
    /// The provided configuration must be compatible with the surface.
    ///
    /// Specifically:
    ///
    /// - The `width` and `height` must be greater than zero, and must be within the bounds allowed
    ///   by the surface.
    ///
    /// - The `present_mode` must be supported by the surface.
    ///
    /// You can check the capabilities of the surface by calling [`Surface::capabilities`].
    pub unsafe fn configure_unchecked(
        &mut self,
        config: SurfaceConfig,
    ) -> Result<(), SurfaceError> {
        let old_swapchain = std::mem::replace(&mut self.swapchain, vk::SwapchainKHR::null());
        self.images.clear();

        let new_swapchain = unsafe {
            create_swapchain(&self.gpu, &self.info, &config, self.surface, old_swapchain)?
        };
        let new_swapchain = ScopeGuard::new(new_swapchain, |s| unsafe {
            self.gpu.vk_fns().destroy_swapchain(self.gpu.vk_device(), s)
        });

        unsafe {
            self.gpu
                .vk_fns()
                .get_swapchain_images(self.gpu.vk_device(), *new_swapchain, &mut self.images)
                .map_err(vk_to_surface_err)?;
        }

        self.swapchain = ScopeGuard::defuse(new_swapchain);
        self.config = config;

        Ok(())
    }

    /// Presents a new image to the surface.
    ///
    /// The contents of the frame is dictated by the provided [`SurfaceContents`] implementation.
    ///
    /// # Safety
    ///
    /// The provided [`SurfaceContents`] implementation must be up-to-date. In other words, it must
    /// be kept up-to-date with the current state of the surface and its swapchain using
    /// [`SurfaceContents::notify_destroy_images`] and [`SurfaceContents::new_images`].
    pub unsafe fn present<C>(&mut self, contents: &mut C) -> Result<(), PresentError<C::Error>>
    where
        C: SurfaceContents,
    {
        if !self.is_swapchain_valid() {
            return Err(PresentError::SwapchainRetired);
        }

        unsafe {
            let acquire_semaphore = self
                .semaphore_pool
                .get(&self.gpu)
                .map_err(|_| PresentError::UnexpectedVulkanBehavior)?;

            let (image_index, _suboptimal) = self
                .gpu
                .vk_fns()
                .acquire_next_image(
                    self.gpu.vk_device(),
                    self.swapchain,
                    u64::MAX - 1,
                    *acquire_semaphore,
                    vk::Fence::null(),
                )
                .map_err(vk_to_present_err)?;

            self.present_wait_semaphores.clear();
            let mut context = FrameContext {
                gpu: self.gpu.clone(),
                acquire_semaphore: *acquire_semaphore,
                image_index,
                wait_semaphores: &mut self.present_wait_semaphores,
                image: *self.images.get_unchecked(image_index as usize),
            };

            contents
                .render(&mut context)
                .map_err(PresentError::Contents)?;

            let present_info = vk::PresentInfoKHR {
                p_image_indices: &image_index,
                p_swapchains: &self.swapchain,
                swapchain_count: 1,
                p_wait_semaphores: context.wait_semaphores.as_ptr(),
                wait_semaphore_count: context.wait_semaphores.len() as u32,
                ..Default::default()
            };

            self.gpu
                .vk_fns()
                .queue_present(self.gpu.vk_queue(), &present_info)
                .map_err(vk_to_present_err)?;

            Ok(())
        }
    }

    /// Returns the swapchain that's used by the surface.
    ///
    /// Note that this function might return `vk::SwapchainKHR::null()` if the swapchain is
    /// currently retired.
    #[inline(always)]
    pub fn vk_swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    /// Returns the Vulkan surface handle.
    #[inline(always)]
    pub fn vk_surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    /// Returns the images that were created for the swapchain.
    #[inline(always)]
    pub fn vk_images(&self) -> &[vk::Image] {
        &self.images
    }
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("Surface");

        if let Ok(caps) = self.capabilities() {
            f.field("capabilities", &caps);
        }

        f.field("config", &self.config)
            .field("swapchain", &(self.swapchain != vk::SwapchainKHR::null()))
            .finish_non_exhaustive()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.semaphore_pool.destroy(&self.gpu);

            if self.swapchain != vk::SwapchainKHR::null() {
                self.gpu
                    .vk_fns()
                    .destroy_swapchain(self.gpu.vk_device(), self.swapchain);
            }

            self.gpu
                .vk_fns()
                .destroy_surface(self.gpu.vk_instance(), self.surface);
        }
    }
}

/// Creates a new swapchain.
unsafe fn create_swapchain(
    gpu: &Gpu,
    info: &SwapchainInfo,
    config: &SurfaceConfig,
    surface: vk::SurfaceKHR,
    old_swapchain: vk::SwapchainKHR,
) -> Result<vk::SwapchainKHR, SurfaceError> {
    let info = vk::SwapchainCreateInfoKHR {
        clipped: vk::TRUE,
        composite_alpha: info.composite_alpha,
        image_array_layers: 1,
        image_color_space: info.color_space,
        image_format: info.format,
        image_extent: vk::Extent2D {
            width: config.width,
            height: config.height,
        },
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        min_image_count: info.min_image_count,
        pre_transform: info.pre_transform,
        present_mode: vk::PresentModeKHR::from_raw(config.present_mode as i32),
        surface,
        old_swapchain,
        ..Default::default()
    };

    unsafe {
        gpu.vk_fns()
            .create_swapchain(gpu.vk_device(), &info)
            .map_err(vk_to_surface_err)
    }
}

/// Returns an instance of
fn get_surface_capabilities(
    gpu: &Gpu,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceCapabilitiesKHR, SurfaceError> {
    unsafe {
        gpu.vk_fns()
            .get_physical_device_surface_capabilities(gpu.vk_physical_device(), surface)
            .map_err(vk_to_surface_err)
    }
}

/// Converts a regular Vulkan result into a [`SurfaceError`].
fn vk_to_surface_err(err: vk::Result) -> SurfaceError {
    match err {
        vk::Result::ERROR_SURFACE_LOST_KHR => SurfaceError::Lost,
        _ => SurfaceError::UnexpectedVulkanBehavior,
    }
}

/// Converts a regular Vulkan result into a [`PresentError<C>`].
fn vk_to_present_err<C>(err: vk::Result) -> PresentError<C> {
    match err {
        vk::Result::ERROR_SURFACE_LOST_KHR => PresentError::Lost,
        vk::Result::ERROR_OUT_OF_DATE_KHR => PresentError::OutOfDate,
        vk::Result::TIMEOUT => PresentError::Timeout,
        _ => PresentError::UnexpectedVulkanBehavior,
    }
}
