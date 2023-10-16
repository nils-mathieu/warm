//! Defines [`SurfaceContents`].

use std::sync::Arc;

use ash::vk;

use crate::gpu::Gpu;

use super::{PresentError, Surface, SurfaceConfig, SurfaceError};

/// Contains data that's relative to a frame being rendered.
#[derive(Debug)]
pub struct FrameContext<'a> {
    pub(super) gpu: Arc<Gpu>,
    pub(super) acquire_semaphore: vk::Semaphore,
    pub(super) wait_semaphores: &'a mut Vec<vk::Semaphore>,
    pub(super) image_index: u32,
    pub(super) image: vk::Image,
}

impl<'a> FrameContext<'a> {
    /// Creates a new [`FrameContext`] with the given data.
    #[inline(always)]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Returns the index of the image that was acquired.
    ///
    /// This information may be used to get image-specific data that the [`SurfaceContents`] may
    /// need.
    #[inline(always)]
    pub fn image_index(&self) -> usize {
        self.image_index as usize
    }

    /// The swapchain image that was acquired.
    #[inline(always)]
    pub fn image(&self) -> vk::Image {
        self.image
    }

    /// Returns a semaphore that is signaled when the image is acquired and ready to be rendered
    /// to.
    #[inline(always)]
    pub fn acquire_semaphore(&self) -> vk::Semaphore {
        self.acquire_semaphore
    }

    /// Returns a vector containing a list of semaphores that must be signaled before the image
    /// can be presented to the surface.
    #[inline(always)]
    pub fn wait_semaphores_mut(&mut self) -> &mut Vec<vk::Semaphore> {
        self.wait_semaphores
    }

    /// Returns the list of semaphores that must be signaled before the image can be presented to
    /// the surface.
    ///
    /// More information in the documentation for [`FrameContext::wait_semaphores_mut`].
    #[inline(always)]
    pub fn wait_semaphores(&self) -> &[vk::Semaphore] {
        self.wait_semaphores
    }
}

/// Stores information about the images that were created for a swapchain.
///
/// An instance of this type is passed to [`SurfaceContents::notify_new_images`].
#[derive(Debug, Clone)]
pub struct ImagesInfo<'a> {
    /// The images that were created.
    pub images: &'a [vk::Image],
    /// The width of the images.
    pub width: u32,
    /// The height of the images.
    pub height: u32,
}

/// A trait for types that can be passed to [`Surface::present`].
///
/// # Object Lifetime
///
/// Any object that implements [`SurfaceContents`] must be kept up-to-date with the surface it
/// renders to. This means that the [`SurfaceContents::notify_new_images`] and
/// [`SurfaceContents::notify_destroy_images`] methods must be called whenever the surface is
/// reconfigured.
///
/// This means that such object can be in either of two states:
///
/// - **Valid**: The object is up-to-date with the surface it renders to, and the last method that
///   was called was [`SurfaceContents::notify_new_images`].
///
/// - **Invalid**: The object is not up-to-date with the surface it renders to, and the last method
///   that was called was [`SurfaceContents::notify_destroy_images`].
///
/// The [`SurfaceContents::render`] method can only be called when the object is in the **Valid**
/// state.
///
/// Note that it is valid to drop the object while it is in either of the two states.
///
/// # Safety
///
/// - After calling [`SurfaceContents::render`], the image must be in the
/// **VK_IMAGE_LAYOUT_PRESENT_SRC** layout.
///
/// - The resources passed to the [`FrameContext`] must remain valid until the frame is presented.
///
/// - The `acquired_semaphore` provided by the [`FrameContext`] must be logically signaled before
///   at least one of the semaphores pushed to the `wait_semaphores` vector. If none of the
///   semaphores you push to that vector depend on the `acquired_semaphore`, you can simply push
///   the `acquired_semaphore` object iself to the vector.
pub unsafe trait SurfaceContents {
    /// An error that might occur when using this [`SurfaceContents`] implementation.
    type Error;

    /// Some arguments passed to the [`SurfaceContents::render`] method.
    type Args<'a>;

    /// Notifies the [`SurfaceContents`] implementation that the images it uses will be destroyed.
    ///
    /// It must destroy any resources that depend on those images.
    ///
    /// # Safety
    ///
    /// This function must not be used twice without calling [`SurfaceContents::notify_new_images`]
    /// in between.
    unsafe fn notify_destroy_images(&mut self) -> Result<(), Self::Error>;

    /// Notifies the [`SurfaceContents`] implementation that new images will now be used.
    ///
    /// # Safety
    ///
    /// This function must not be used twice without calling
    /// [`SurfaceContents::notify_destroy_images`] in between.
    ///
    /// If the [`SurfaceContents`] implementation depends on a [`Gpu`] connection, the provided
    /// images must belong to that same [`Gpu`] instance.
    unsafe fn notify_new_images(&mut self, info: ImagesInfo) -> Result<(), Self::Error>;

    /// Renders the contents of the surface to the given image.
    ///
    /// # Safety
    ///
    /// The [`SurfaceContents`] instance must be kept up-to-date using the
    /// [`SurfaceContents::notify_new_images`] and [`SurfaceContents::notify_destroy_images`]
    /// methods.
    ///
    /// If the [`SurfaceContents`] implementation depends on a [`Gpu`] connection, the provided
    /// [`FrameContext`] must belong to that same [`Gpu`] instance.
    unsafe fn render(
        &mut self,
        ctx: &mut FrameContext,
        args: Self::Args<'_>,
    ) -> Result<(), Self::Error>;
}

/// Wraps a [`Surface`] and a [`SurfaceContents`] to keep up-to-date with it.
#[derive(Debug)]
pub struct SurfaceWithContents<C> {
    /// The surface that this [`SurfaceWithContents`] is wrapping.
    surface: Surface,
    /// The contents of the surface.
    contents: C,
    /// Whether the contents have been invalidated without being able to recover. That happens when
    /// the surface fails to be reconfigured.
    contents_valid: bool,
}

impl<C> SurfaceWithContents<C> {
    /// Creates a new [`SurfaceWithContents`] from the given surface and contents.
    ///
    /// # Safety
    ///
    /// The provided [`SurfaceContents`] implementation must be in the **Invalid** state.
    pub unsafe fn new(surface: Surface, contents: C) -> Self {
        Self {
            surface,
            contents,
            contents_valid: false,
        }
    }

    /// Returns a shared reference to the surface.
    #[inline(always)]
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Returns an exclusive reference to the surface.
    ///
    /// # Safety
    ///
    /// The surface must not be re-configured without updating the inner [`SurfaceContents`]
    /// implementation.
    #[inline(always)]
    pub unsafe fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }

    /// Returns a shared reference to the underlying [`SurfaceContents`] implementation.
    #[inline(always)]
    pub fn contents(&self) -> &C {
        &self.contents
    }

    /// Returns an exclusive reference to the underlying [`SurfaceContents`] implementation.
    ///
    /// # Safety
    ///
    /// The contents must not be invalidated.
    #[inline(always)]
    pub unsafe fn contents_mut(&mut self) -> &mut C {
        &mut self.contents
    }

    /// Returns whether the [`SurfaceContents`] implementation is currently valid.
    #[inline(always)]
    pub fn is_contents_valid(&self) -> bool {
        self.contents_valid
    }
}

impl<C: SurfaceContents> SurfaceWithContents<C> {
    /// Re-configures the underlying surface, properly updating the [`SurfaceContents`]
    /// implementation.
    ///
    /// # Safety
    ///
    /// Same requirements as [`Surface::configure_unchecked`].
    pub unsafe fn configure_unchecked(
        &mut self,
        config: SurfaceConfig,
    ) -> Result<(), SurfaceError<C::Error>> {
        unsafe {
            let width = config.width;
            let height = config.height;

            if self.contents_valid {
                self.contents
                    .notify_destroy_images()
                    .map_err(SurfaceError::Contents)?;
                self.contents_valid = false;
            }

            self.surface
                .configure_unchecked(config)
                .map_err(SurfaceError::cast_contents)?;

            self.contents
                .notify_new_images(ImagesInfo {
                    width,
                    height,
                    images: self.surface.vk_images(),
                })
                .map_err(SurfaceError::Contents)?;
            self.contents_valid = true;
        }

        Ok(())
    }

    /// Re-configures the surface with the given configuration, properly updating the
    /// [`SurfaceContents`] implementation.
    ///
    /// More information in the documentation for [`Surface::configure`].
    pub fn configure(&mut self, config: SurfaceConfig) -> Result<(), SurfaceError<C::Error>> {
        let caps = self
            .surface
            .capabilities()
            .map_err(SurfaceError::cast_contents)?;
        if !caps.is_config_valid(&config) {
            return Err(SurfaceError::InvalidConfig);
        }

        unsafe { self.configure_unchecked(config) }
    }

    /// Presents an additional image to the surface, using the managed [`SurfaceContents`]
    /// implementation.
    pub fn present(&mut self, args: C::Args<'_>) -> Result<(), PresentError<C::Error>> {
        if !self.contents_valid {
            return Err(PresentError::OutOfDate);
        }

        unsafe { self.surface.present(&mut self.contents, args) }
    }
}
