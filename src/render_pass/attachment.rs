//! Defines the [`Attachment`] trait.

use std::any::TypeId;

use ash::vk;

/// A trait for types that may be used as an attachment in a [`RenderPass`](super::RenderPass).
pub trait Attachment {
    /// A value that's used to clear the attachment.
    ///
    /// If the attachment does not need to be cleared, use `()`.
    type ClearValue;

    /// Creates an [`vk::AttachmentDescription`] for this attachment type.
    fn description(&self) -> vk::AttachmentDescription;

    /// Returns the [`vk::ImageView`] of this attachment.
    fn image_view(&self) -> vk::ImageView;
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

    /// A tuple of references to the attachments in this list.
    type References;

    /// An array-like type that may be turend into a regular Rust slice of [`vk::ImageView`].
    type RawReferences: AsRef<[vk::ImageView]>;

    /// Converts [`Self::References`] into [`Self::RawReferences`].
    fn build_references(references: Self::References) -> Self::RawReferences;

    /// Calls a function for each attachment in this list.
    fn register_attachments(register: impl FnMut(TypeId, vk::AttachmentDescription));
}

/// A trait for types that may be used to clear a render pass attachment.
pub trait ClearValue {
    /// Converts the value into a [`vk::ClearValue`].
    fn into_clear_value(self) -> vk::ClearValue;
}

impl ClearValue for () {
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
