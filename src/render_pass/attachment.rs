//! Defines the [`Attachment`] trait.

use std::any::TypeId;

use ash::vk;

use crate::surface::ImagesInfo;
use crate::VulkanError;

/// A trait for types that may be used as an attachment in a [`RenderPass`](super::RenderPass).
pub trait Attachment: 'static {
    /// A value that's used to clear the attachment.
    ///
    /// If the attachment does not need to be cleared, use `()`.
    type ClearValue: ClearValue;

    /// Creates an [`vk::AttachmentDescription`] for this attachment type.
    fn description(&self) -> vk::AttachmentDescription;

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
    fn register(&self, register: impl FnMut(TypeId, vk::AttachmentDescription));
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

            fn register(&self, mut register: impl FnMut(TypeId, vk::AttachmentDescription)) {
                let ( $($T,)* ) = self;

                $(
                    register(TypeId::of::<$T>(), $T.description());
                )*
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
