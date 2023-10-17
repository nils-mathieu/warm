//! Defines the [`Subpass`] trait.

use std::any::TypeId;

use ash::vk;

/// Represents a list of types.
///
/// This trait is used to gather the list of [`TypeId`]s required by a [`Subpass`].
pub trait TypeList {}

/// Describes a subpass. An instance of this type is returned by [`RawSubpass::register`].
#[derive(Debug, Clone)]
pub struct SubpassDescription {
    /// The index of the first color attachment in the subpass. Only relevant if
    /// `color_attachment_count` is non-zero.
    pub first_color_attachment: usize,
    /// The number of color attachments in the subpass.
    pub color_attachment_count: usize,
}

/// A subpass that can be registered with a [`RenderPass`](super::RenderPass).
pub trait Subpass {
    /// Returns the [`SubpassDescription`] of this subpass.
    ///
    /// The `request_attachment` closure is called for each type of attachment that the subpass
    /// needs. The closure is passed the [`TypeId`] of the attachment type, and it returns an
    /// index into the list of attachment references.
    fn description(
        &self,
        request_attachment: impl FnMut(TypeId, vk::ImageLayout) -> usize,
    ) -> SubpassDescription;

    /// Some arguments passed to the [`record`](RawSubpass::record) method.
    type Args<'a>;

    /// Records the commands for this subpass.
    fn record(&mut self, args: Self::Args<'_>);
}

/// A [`Subpass`] that does nothing.
#[derive(Debug)]
pub struct EmptySubpass;

impl Subpass for EmptySubpass {
    fn description(
        &self,
        _request_attachment: impl FnMut(TypeId, vk::ImageLayout) -> usize,
    ) -> SubpassDescription {
        SubpassDescription {
            first_color_attachment: 0,
            color_attachment_count: 0,
        }
    }

    type Args<'a> = ();

    fn record(&mut self, (): Self::Args<'_>) {}
}

/// A list of [`Subpass`]es.
pub trait SubpassList {
    /// Registers a number of [`SubpassDescription`]s with the given closure.
    fn register(
        &self,
        request_attachment: impl FnMut(TypeId, vk::ImageLayout) -> usize,
        register: impl FnMut(SubpassDescription),
    );

    /// A type that includes all of the [`Subpass`]es in this list.
    type Args<'a>;

    /// Records the commands for all of the subpasses in this list.
    ///
    /// The `next_subpass` closure is called between each subpass of the list. Note that if there
    /// is only one subpass in the list, `next_subpass` must never be called.
    fn record(&mut self, args: Self::Args<'_>, next_subpass: impl FnMut());
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
                mut request_attachment: impl FnMut(TypeId, vk::ImageLayout) -> usize,
                mut register: impl FnMut(SubpassDescription),
            ) {
                let ( $($T,)* ) = self;

                $(
                    register($T.description(&mut request_attachment));
                )*
            }

            type Args<'a> = ( $( $T::Args<'a>, )* );

            fn record(&mut self, args: Self::Args<'_>, mut next_subpass: impl FnMut()) {
                let ( $($T,)* ) = self;
                let ( $($t,)* ) = args;

                impl_SubpassList!( alternate, next_subpass(), $( $T.record($t), )* );
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
