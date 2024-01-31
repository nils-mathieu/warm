use ash::vk;
use bitflags::bitflags;

use crate::ImageUsages;

bitflags! {
    /// Flags specifying a collection of [`PresentMode`]s.
    ///
    /// The description of the variants is written for the variants of [`PresentMode`].
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PresentModes: u32 {
        const IMMEDIATE = 1 << 0;
        const MAILBOX = 1 << 1;
        const FIFO = 1 << 2;
        const FIFO_RELAXED = 1 << 3;
    }
}

impl From<PresentMode> for PresentModes {
    fn from(value: PresentMode) -> Self {
        match value {
            PresentMode::Immediate => Self::IMMEDIATE,
            PresentMode::Mailbox => Self::MAILBOX,
            PresentMode::Fifo => Self::FIFO,
            PresentMode::FifoRelaxed => Self::FIFO_RELAXED,
        }
    }
}

// TODO: add documentation on the variants.
/// A mode that the presentation engine can operate in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

/// Represents the capabilities of a surface.
#[derive(Debug, Clone)]
#[doc(alias = "vkSurfaceCapabilitiesKHR")]
pub struct SurfaceCaps {
    /// The minimum number of images that the presentation engine requires.
    pub min_image_count: u32,
    /// The maximum (if any) of images that the presentation engine supports.
    pub max_image_count: Option<u32>,
    /// The current extent of the surface.
    ///
    /// Note that this might not be available if the surface has no fixed extent (or if it won't
    /// have a defined size until the swapchain is created).
    pub current_extent: Option<[u32; 2]>,
    /// The minimum size of the swapchain's images.
    pub min_image_extent: [u32; 2],
    /// The maximum size of the swapchain's images.
    pub max_image_extent: [u32; 2],
    /// The maximum number of layers that the swapchain's images can have.
    pub max_image_array_layers: u32,
    /// A bitmask of [`SurfaceTransform`]s that are supported by the presentation engine.
    pub supported_transforms: SurfaceTransforms,
    /// The current transform of the surface.
    pub current_transform: SurfaceTransform,
    /// A bitmask of [`CompositeAlpha`]s that are supported by the presentation engine.
    pub supported_composite_alpha: CompositeAlphas,
    /// A bitmask of [`ImageUsage`]s that are supported by the presentation engine.
    pub supported_usage: ImageUsages,
}

bitflags! {
    /// A set of [`SurfaceTransform`]s.
    ///
    /// More information can be found in the documentation for [`SurfaceTransform`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SurfaceTransforms: u32 {
        const IDENTITY = vk::SurfaceTransformFlagsKHR::IDENTITY.as_raw();
        const ROTATE_90 = vk::SurfaceTransformFlagsKHR::ROTATE_90.as_raw();
        const ROTATE_180 = vk::SurfaceTransformFlagsKHR::ROTATE_180.as_raw();
        const ROTATE_270 = vk::SurfaceTransformFlagsKHR::ROTATE_270.as_raw();
        const HORIZONTAL_MIRROR = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR.as_raw();
        const HORIZONTAL_MIRROR_ROTATE_90 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_90.as_raw();
        const HORIZONTAL_MIRROR_ROTATE_180 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_180.as_raw();
        const HORIZONTAL_MIRROR_ROTATE_270 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_270.as_raw();
        const INHERIT = vk::SurfaceTransformFlagsKHR::INHERIT.as_raw();
    }
}

/// A transformation that can be applied to a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SurfaceTransform {
    Identity = vk::SurfaceTransformFlagsKHR::IDENTITY.as_raw(),
    Rotate90 = vk::SurfaceTransformFlagsKHR::ROTATE_90.as_raw(),
    Rotate180 = vk::SurfaceTransformFlagsKHR::ROTATE_180.as_raw(),
    Rotate270 = vk::SurfaceTransformFlagsKHR::ROTATE_270.as_raw(),
    HorizontalMirror = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR.as_raw(),
    HorizontalMirrorRotate90 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_90.as_raw(),
    HorizontalMirrorRotate180 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_180.as_raw(),
    HorizontalMirrorRotate270 = vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_270.as_raw(),
    Inherit = vk::SurfaceTransformFlagsKHR::INHERIT.as_raw(),
}

impl From<SurfaceTransform> for SurfaceTransforms {
    fn from(value: SurfaceTransform) -> Self {
        match value {
            SurfaceTransform::Identity => Self::IDENTITY,
            SurfaceTransform::Rotate90 => Self::ROTATE_90,
            SurfaceTransform::Rotate180 => Self::ROTATE_180,
            SurfaceTransform::Rotate270 => Self::ROTATE_270,
            SurfaceTransform::HorizontalMirror => Self::HORIZONTAL_MIRROR,
            SurfaceTransform::HorizontalMirrorRotate90 => Self::HORIZONTAL_MIRROR_ROTATE_90,
            SurfaceTransform::HorizontalMirrorRotate180 => Self::HORIZONTAL_MIRROR_ROTATE_180,
            SurfaceTransform::HorizontalMirrorRotate270 => Self::HORIZONTAL_MIRROR_ROTATE_270,
            SurfaceTransform::Inherit => Self::INHERIT,
        }
    }
}

bitflags! {
    /// A set of [`CompositeAlpha`] modes.
    ///
    /// More information can be found in the documentation for [`CompositeAlpha`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CompositeAlphas: u32 {
        const OPAQUE = vk::CompositeAlphaFlagsKHR::OPAQUE.as_raw();
        const PRE_MULTIPLIED = vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED.as_raw();
        const POST_MULTIPLIED = vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED.as_raw();
        const INHERIT = vk::CompositeAlphaFlagsKHR::INHERIT.as_raw();
    }
}

impl From<CompositeAlpha> for CompositeAlphas {
    fn from(value: CompositeAlpha) -> Self {
        match value {
            CompositeAlpha::Opaque => Self::OPAQUE,
            CompositeAlpha::PreMultiplied => Self::PRE_MULTIPLIED,
            CompositeAlpha::PostMultiplied => Self::POST_MULTIPLIED,
            CompositeAlpha::Inherit => Self::INHERIT,
        }
    }
}

/// A mode that the presentation engine can operate in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum CompositeAlpha {
    Opaque = vk::CompositeAlphaFlagsKHR::OPAQUE.as_raw(),
    PreMultiplied = vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED.as_raw(),
    PostMultiplied = vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED.as_raw(),
    Inherit = vk::CompositeAlphaFlagsKHR::INHERIT.as_raw(),
}
