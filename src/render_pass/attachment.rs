//! Defines the [`Attachment`] trait.

use std::sync::Arc;

use ash::vk;

use crate::gpu::Gpu;
use crate::surface::ImagesInfo;
use crate::VulkanError;

use super::{RenderPassBuilder, RenderPassError};

/// A trait for types that may be used as an attachment in a [`RenderPass`](super::RenderPass).
pub trait Attachment: 'static {
    /// A value that's used to clear the attachment.
    ///
    /// If the attachment does not need to be cleared, use `()`.
    type ClearValue: ClearValue;

    /// Creates an [`vk::AttachmentDescription`] for this attachment type.
    fn description(&self) -> Result<vk::AttachmentDescription, RenderPassError>;

    /// Returns the [`vk::ImageView`] of this attachment.
    ///
    /// # Safety
    ///
    /// - `index` must be less than the number of images notified by [`notify_output_changed`].
    ///
    /// [`notify_output_changed`]: Attachment::notify_output_changed
    unsafe fn image_view(&self, index: usize) -> vk::ImageView;

    /// Notifies the attachment that the output images are being destroyed.
    fn notify_destroying_output(&mut self);

    /// Notifies the attachment that the output image has changed.
    fn notify_output_changed(&mut self, info: &ImagesInfo) -> Result<(), VulkanError>;
}

/// An implementation of [`Attachment`] that represents the output of a render pass.
#[derive(Debug)]
pub struct OutputAttachment {
    /// The GPU to which this attachment is bound to.
    gpu: Arc<Gpu>,

    /// The output format of the attachment.
    format: vk::Format,

    /// The image views of the output images.
    views: Vec<vk::ImageView>,
}

impl OutputAttachment {
    /// Creates a new [`OutputAttachment`].
    pub fn new(gpu: Arc<Gpu>, format: vk::Format) -> Self {
        Self {
            gpu,
            format,
            views: Vec::new(),
        }
    }
}

impl Attachment for OutputAttachment {
    type ClearValue = ClearColorValue<f32>;

    fn description(&self) -> Result<vk::AttachmentDescription, RenderPassError> {
        Ok(vk::AttachmentDescription {
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: self.format,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            samples: vk::SampleCountFlags::TYPE_1,
        })
    }

    #[inline(always)]
    unsafe fn image_view(&self, index: usize) -> vk::ImageView {
        unsafe { *self.views.get_unchecked(index) }
    }

    fn notify_destroying_output(&mut self) {
        for view in self.views.drain(..) {
            unsafe {
                self.gpu
                    .vk_fns()
                    .destroy_image_view(self.gpu.vk_device(), view)
            };
        }
    }

    fn notify_output_changed(&mut self, info: &ImagesInfo) -> Result<(), VulkanError> {
        if info.format != self.format {
            return Err(VulkanError::ERROR_FORMAT_NOT_SUPPORTED);
        }

        for &image in info.images {
            let info = vk::ImageViewCreateInfo {
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                flags: vk::ImageViewCreateFlags::empty(),
                format: self.format,
                image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_array_layer: 0,
                    base_mip_level: 0,
                    layer_count: 1,
                    level_count: 1,
                },
                view_type: vk::ImageViewType::TYPE_2D,
                ..Default::default()
            };

            let view = unsafe {
                self.gpu
                    .vk_fns()
                    .create_image_view(self.gpu.vk_device(), &info)?
            };

            self.views.push(view);
        }

        Ok(())
    }
}

impl Drop for OutputAttachment {
    fn drop(&mut self) {
        for view in self.views.drain(..) {
            unsafe {
                self.gpu
                    .vk_fns()
                    .destroy_image_view(self.gpu.vk_device(), view)
            };
        }
    }
}

/// A list of render pass [`Attachment`]s.
pub trait AttachmentList {
    /// A type that can be turned into a list of clear values for the attachments of this list.
    type ClearValues;

    /// An array-like type that may be turned into a regular Rust slice of [`vk::ClearValue`]. This
    /// will usually be a constant-size array of [`vk::ClearValue`].
    type RawClearValues: AsRef<[vk::ClearValue]>;

    /// Converts [`Self::ClearValues`] into [`Self::RawClearValues`].
    fn build_clear_values(clear_values: Self::ClearValues) -> Self::RawClearValues;

    /// An array-like type that may be turend into a regular Rust slice of [`vk::ImageView`].
    type RawImageViews: AsRef<[vk::ImageView]>;

    /// Returns the [`vk::ImageView`]s of the attachments in this list.
    ///
    /// The images view are associated with the swapchain image `index`.
    ///
    /// # Safety
    ///
    /// `index` must be less than the number of images notified by [`notify_output_changed`].
    unsafe fn image_views(&self, index: usize) -> Self::RawImageViews;

    /// Notifies the attachments that the output images are being destroyed.
    fn notify_destroying_output(&mut self);

    /// Notifies the attachments that the output image has changed.
    fn notify_output_changed(&mut self, info: &ImagesInfo) -> Result<(), VulkanError>;

    /// Calls a function for each attachment in this list.
    fn register(&self, builder: &mut RenderPassBuilder) -> Result<(), RenderPassError>;
}

macro_rules! impl_AttachmentList {
    ( $($T:ident),* ) => {
        #[allow(unused_variables, unused_mut, non_snake_case)]
        impl< $($T,)* > AttachmentList for ( $($T,)* )
        where
            $( $T: Attachment, )*
        {
            type ClearValues = ( $($T::ClearValue,)* );

            type RawClearValues = [vk::ClearValue; impl_AttachmentList!( @ $($T,)* )];

            fn build_clear_values(clear_values: Self::ClearValues) -> Self::RawClearValues {
                let ( $($T,)* ) = clear_values;

                [
                    $( $T.into_clear_value(), )*
                ]
            }

            type RawImageViews = [vk::ImageView; impl_AttachmentList!( @ $($T,)* )];

            unsafe fn image_views(&self, index: usize) -> Self::RawImageViews {
                let ( $($T,)* ) = self;

                [
                    $( unsafe { $T.image_view(index) }, )*
                ]
            }

            fn notify_destroying_output(&mut self) {
                let ( $($T,)* ) = self;

                $( $T.notify_destroying_output(); )*
            }

            fn notify_output_changed(&mut self, info: &ImagesInfo) -> Result<(), VulkanError> {
                let ( $($T,)* ) = self;

                $(
                    $T.notify_output_changed(info)?;
                )*

                Ok(())
            }

            fn register(&self, builder: &mut RenderPassBuilder) -> Result<(), RenderPassError> {
                let ( $($T,)* ) = self;

                $(
                    builder.register_attachment($T)?;
                )*

                Ok(())
            }
        }
    };

    ( @ $first:ident, $($rest:ident,)* ) => {
        1 + impl_AttachmentList!(@ $($rest,)*)
    };

    ( @ ) => {
        0
    };
}

impl_AttachmentList!();
impl_AttachmentList!(A);
impl_AttachmentList!(A, B);
impl_AttachmentList!(A, B, C);
impl_AttachmentList!(A, B, C, D);
impl_AttachmentList!(A, B, C, D, E);
impl_AttachmentList!(A, B, C, D, E, F);

/// A trait for types that may be used to clear a render pass attachment.
pub trait ClearValue {
    /// Converts the value into a [`vk::ClearValue`].
    fn into_clear_value(self) -> vk::ClearValue;
}

impl ClearValue for () {
    #[inline(always)]
    fn into_clear_value(self) -> vk::ClearValue {
        unsafe { std::mem::zeroed() }
    }
}

/// An implementation of [`ClearValue`] that can be used to clear a render pass attachment with a
/// color.
#[derive(Debug, Clone, Copy)]
pub struct ClearColorValue<T> {
    /// The red component of the color.
    pub red: T,
    /// The green component of the color.
    pub green: T,
    /// The blue component of the color.
    pub blue: T,
    /// The alpha component of the color.
    pub alpha: T,
}

impl<T> From<ClearColorValue<T>> for [T; 4] {
    #[inline]
    fn from(value: ClearColorValue<T>) -> Self {
        [value.red, value.green, value.blue, value.alpha]
    }
}

impl ClearValue for ClearColorValue<f32> {
    #[inline]
    fn into_clear_value(self) -> vk::ClearValue {
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: self.into(),
            },
        }
    }
}

impl ClearValue for ClearColorValue<u32> {
    #[inline]
    fn into_clear_value(self) -> vk::ClearValue {
        vk::ClearValue {
            color: vk::ClearColorValue {
                uint32: self.into(),
            },
        }
    }
}

impl ClearValue for ClearColorValue<i32> {
    #[inline]
    fn into_clear_value(self) -> vk::ClearValue {
        vk::ClearValue {
            color: vk::ClearColorValue { int32: self.into() },
        }
    }
}

/// An implementation of [`ClearValue`] that can be used to clear a render pass attachment with a
/// depth/stencil value.
#[derive(Debug, Clone, Copy)]
pub struct ClearDepthStencil {
    /// The depth component of the clear value.
    pub depth: f32,
    /// The stencil component of the clear value.
    pub stencil: u32,
}

impl ClearValue for ClearDepthStencil {
    #[inline]
    fn into_clear_value(self) -> vk::ClearValue {
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: self.depth,
                stencil: self.stencil,
            },
        }
    }
}
