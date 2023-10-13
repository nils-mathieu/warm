//! Defines [`Surface`].

use std::fmt;
use std::ptr::null;
use std::sync::Arc;

use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::gpu::Gpu;
use crate::utility::ScopeGuard;

mod error;

pub use self::error::*;

mod swapchain_info;
mod window;

use self::swapchain_info::SwapchainInfo;

/// Some additional configuration for the swapchain.
///
/// Those parameters may change during the lifetime of the program, and the swapchain will have
/// to be recreated if they do.
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    /// The width of swapchain images.
    pub width: u32,
    /// The height of swapchain images.
    pub height: u32,
}

/// A trait for surfaces on which we can render.
pub trait SurfaceTarget: HasWindowHandle + HasDisplayHandle {}
impl<T: HasWindowHandle + HasDisplayHandle> SurfaceTarget for T {}

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

    /// Information about the swapchain.
    info: SwapchainInfo,

    /// The current configuration of the surface.
    config: SurfaceConfig,
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
            info,
            config: SurfaceConfig {
                width: 0,
                height: 0,
            },
        })
    }

    /// The current configuration of the surface.
    pub fn config(&self) -> &SurfaceConfig {
        &self.config
    }

    /// Configures the surface with the provided configuration.
    ///
    /// # Note
    ///
    /// When this function fails, the surface is left in an unusable state. Specifically, it is
    /// no longer possible to render to the surface and it has to be configured again.
    pub fn configure(&mut self, config: SurfaceConfig) -> Result<(), SurfaceConfigureError> {
        let old_swapchain = std::mem::replace(&mut self.swapchain, vk::SwapchainKHR::null());

        self.swapchain =
            create_swapchain(&self.gpu, &self.info, &config, self.surface, old_swapchain)?;

        Ok(())
    }
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Surface")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
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
fn create_swapchain(
    gpu: &Gpu,
    info: &SwapchainInfo,
    config: &SurfaceConfig,
    surface: vk::SurfaceKHR,
    old_swapchain: vk::SwapchainKHR,
) -> Result<vk::SwapchainKHR, SurfaceConfigureError> {
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
        present_mode: info.present_mode,
        surface,
        old_swapchain,
        ..Default::default()
    };

    unsafe {
        gpu.vk_fns()
            .create_swapchain(gpu.vk_device(), &info)
            .map_err(|_| SurfaceConfigureError)
    }
}
