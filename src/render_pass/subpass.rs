//! Defines the [`Subpass`] trait.

use ash::vk;

use super::attachment::OutputAttachment;
use super::{RenderPassBuilder, RenderPassError};

/// Represents a list of types.
///
/// This trait is used to gather the list of [`TypeId`]s required by a [`Subpass`].
pub trait TypeList {}

/// Describes a subpass. An instance of this type is returned by [`RawSubpass::register`].
#[derive(Debug, Clone)]
pub struct SubpassDescription {
    /// The index of the first input attachment in the subpass. Only relevant if
    /// `input_attachment_count` is non-zero.
    ///
    /// Must be a *attachment reference* index.
    pub first_input_attachment: usize,
    /// The number of input attachments in the subpass.
    pub input_attachment_count: usize,
    /// The index of the first color attachment in the subpass. Only relevant if
    /// `color_attachment_count` is non-zero.
    ///
    /// Must be a *attachment reference* index.
    pub first_color_attachment: usize,
    /// The number of color attachments in the subpass.
    pub color_attachment_count: usize,
    /// The depth/stencil attachment, if any.
    ///
    /// Must be a *attachment reference* index.
    pub depth_stencil_attachment: Option<usize>,
    /// The index of the first input attachment in the subpass. Only relevant if
    /// `input_attachment_count` is non-zero.
    ///
    /// Must be a *attachment* index.
    pub first_preserve_attachment: usize,
    /// The number of input attachments in the subpass.
    ///
    /// Must be a *attachment* index.
    pub preserve_attachment_count: usize,
}

/// A subpass that can be registered with a [`RenderPass`](super::RenderPass).
pub trait Subpass {
    /// Returns the [`SubpassDescription`] of this subpass.
    ///
    /// The `request_attachment` closure is called for each type of attachment that the subpass
    /// needs. The closure is passed the [`TypeId`] of the attachment type, and it returns an
    /// index into the list of attachment references.
    fn register(
        &self,
        builder: &mut RenderPassBuilder,
    ) -> Result<SubpassDescription, RenderPassError>;

    /// Some arguments passed to the [`record`](RawSubpass::record) method.
    type Args<'a>;

    /// Records the commands for this subpass.
    fn record(&mut self, cmd: vk::CommandBuffer, args: Self::Args<'_>);
}

/// A [`Subpass`] that does nothing.
#[derive(Debug)]
pub struct EmptySubpass;

impl Subpass for EmptySubpass {
    fn register(
        &self,
        builder: &mut RenderPassBuilder,
    ) -> Result<SubpassDescription, RenderPassError> {
        let output = builder
            .request_attachment_ref::<OutputAttachment>(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .ok_or(RenderPassError::MissingAttachment)?;

        Ok(SubpassDescription {
            depth_stencil_attachment: None,
            first_input_attachment: 0,
            input_attachment_count: 0,
            first_color_attachment: output,
            color_attachment_count: 1,
            first_preserve_attachment: 0,
            preserve_attachment_count: 0,
        })
    }

    type Args<'a> = ();

    fn record(&mut self, _cmd: vk::CommandBuffer, (): Self::Args<'_>) {}
}

/// A list of [`Subpass`]es.
pub trait SubpassList {
    /// Registers a number of [`SubpassDescription`]s with the given closure.
    fn register(&self, builder: &mut RenderPassBuilder) -> Result<(), RenderPassError>;

    /// A type that includes all of the [`Subpass`]es in this list.
    type Args<'a>;

    /// Records the commands for all of the subpasses in this list.
    ///
    /// The `next_subpass` closure is called between each subpass of the list. Note that if there
    /// is only one subpass in the list, `next_subpass` must never be called.
    fn record(&mut self, cmd: vk::CommandBuffer, args: Self::Args<'_>, next_subpass: impl FnMut());
}

macro_rules! impl_SubpassList {
    ( $( $T:ident $t:ident ),* ) => {
        #[allow(unused_variables, unused_mut, non_snake_case)]
        impl< $($T,)* > SubpassList for ( $($T,)* )
        where
            $( $T: Subpass, )*
        {
            fn register(
                &self,
                builder: &mut RenderPassBuilder,
            ) -> Result<(), RenderPassError> {
                let ( $($T,)* ) = self;

                $(
                    builder.register_subpass($T)?;
                )*

                Ok(())
            }

            type Args<'a> = ( $( $T::Args<'a>, )* );

            fn record(&mut self, cmd: vk::CommandBuffer, args: Self::Args<'_>, mut next_subpass: impl FnMut()) {
                let ( $($T,)* ) = self;
                let ( $($t,)* ) = args;

                impl_SubpassList!( alternate, next_subpass(), $( $T.record(cmd, $t), )* );
            }
        }
    };

    ( alternate, $e:expr, $first:expr, $( $rest:expr, )* ) => {
        $first;
        $(
            $e;
            $rest;
        )*
    };

    ( alternate, $e:expr, ) => {};
}

impl_SubpassList!();
impl_SubpassList!(A a);
impl_SubpassList!(A a, B b);
impl_SubpassList!(A a, B b, C c);
impl_SubpassList!(A a, B b, C c, D d);
impl_SubpassList!(A a, B b, C c, D d, E e);
impl_SubpassList!(A a, B b, C c, D d, E e, F f);
