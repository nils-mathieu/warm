//! Defines [`SurfaceContents`].

use std::sync::Arc;

use ash::vk;

use crate::gpu::Gpu;

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

/// A trait for types that can be passed to [`Surface::present`].
///
/// # Safety
///
/// - After calling [`SurfaceContents::render`], the image must be in the
/// **VK_IMAGE_LAYOUT_PRESENT_SRC** layout.
///
/// - The resources passed to the [`FrameContext`] must remain valid until the frame is presented.
///
/// - The `acquire_semaphore` must be signaled for at least one of the `wait_semaphores` to be
///   signaled. If `acquire_semaphore` depends on none of the semaphores pushed to
///   `wait_semaphores`, then `acquire_semaphore` itself must be pushed to the list.
pub unsafe trait SurfaceContents {
    /// An error that might occur when using this [`SurfaceContents`] implementation.
    type Error;

    /// Notifies the [`SurfaceContents`] implementation that the images it uses will be destroyed.
    ///
    /// It must destroy any resources that depend on those images.
    ///
    /// # Safety
    ///
    /// This function must not be used twice without calling [`SurfaceContents::new_images`] in
    /// between.
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
    unsafe fn new_images(&mut self, images: &[vk::Image]) -> Result<(), Self::Error>;

    /// Renders the contents of the surface to the given image.
    ///
    /// # Safety
    ///
    /// The [`SurfaceContents`] instance must be kept up-to-date using the
    ///
    /// If the [`SurfaceContents`] implementation depends on a [`Gpu`] connection, the provided
    /// [`FrameContext`] must belong to that same [`Gpu`] instance.
    unsafe fn render(&mut self, ctx: &mut FrameContext) -> Result<(), Self::Error>;
}
