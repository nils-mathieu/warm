//! Defines the [`RenderPass`] type, which implements the [`SurfaceContents`] trait.

use std::sync::Arc;

use crate::gpu::Gpu;
use crate::surface::{FrameContext, ImagesInfo, SurfaceContents};

pub mod attachment;
pub mod subpass;

use self::attachment::AttachmentList;
use self::subpass::SubpassList;

mod error;

pub use self::error::*;

/// An implementation of [`SurfaceContents`] that uses a render pass to render the frames to
/// present to a surface.
#[derive(Debug, Clone, Copy)]
pub struct RenderPass<Attachments, Subpasses> {
    a: Attachments,
    s: Subpasses,
}

impl<Attachments, Subpasses> RenderPass<Attachments, Subpasses> {
    /// Creates a new [`RenderPass`] instance.
    pub fn new(gpu: Arc<Gpu>) -> Self {
        unimplemented!();
    }
}

unsafe impl<Attachments, Subpasses> SurfaceContents for RenderPass<Attachments, Subpasses>
where
    Attachments: AttachmentList,
    Subpasses: SubpassList,
{
    type Error = RenderPassError;
    type Args<'a> = RenderPassArgs<'a, Attachments, Subpasses>;

    unsafe fn notify_destroy_images(&mut self) {
        unimplemented!();
    }

    unsafe fn notify_new_images(&mut self, info: ImagesInfo) -> Result<(), Self::Error> {
        unimplemented!();
    }

    unsafe fn render(
        &mut self,
        ctx: &mut FrameContext,
        args: Self::Args<'_>,
    ) -> Result<(), Self::Error> {
        unimplemented!();
    }
}

/// The arguments passed to the [`render`](SurfaceContents::render) method of a [`RenderPass`].
#[derive(Debug)]
pub struct RenderPassArgs<'a, Attachments, Subpasses>
where
    Attachments: AttachmentList,
    Subpasses: SubpassList,
{
    /// A tuple of clear values for each attachment of the render pass.
    pub clear_values: Attachments::ClearValues,
    /// The arguments passed to the subpasses.
    pub args: Subpasses::Args<'a>,
}
