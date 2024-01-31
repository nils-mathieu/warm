use std::mem::ManuallyDrop;
use std::sync::Arc;

use ash::vk;
use bitflags::bitflags;

use crate::{ColorSpace, Device, Format, ImageUsages, Instance, Result, SharingMode, Surface};

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

/// Describes how to create a [`Swapchain`].
#[derive(Debug, Clone)]
#[doc(alias = "vkSwapchainCreateInfoKHR")]
pub struct SwapchainDesc<'a> {
    /// Whether the swapchain should allow occluded parts of the output surface to avoid being
    /// rendered to.
    ///
    /// This is useful when the application that no side effects are required to be exected
    /// for pixels that are not visible.
    pub clipped: bool,

    /// The composite alpha mode that the swapchain should use.
    pub composite_alpha: CompositeAlpha,

    /// The present mode that the swapchain should use.
    pub present_mode: PresentMode,

    /// The minimum number of images that the swapchain should use.
    ///
    /// Note that it's possible for the swapchain to use more images than requested.
    pub min_image_count: u32,

    /// The format that the swapchain should use for the images.
    pub format: Format,

    /// The color space that the swapchain should use for the images.
    pub color_space: ColorSpace,

    /// The size of the swapchain's images.
    pub extent: [u32; 2],

    /// The number of layers that the swapchain's images should have.
    pub array_layers: u32,

    /// The usage that the swapchain's images should have.
    pub usage: ImageUsages,

    /// The sharing mode of the swapchain images.
    pub sharing_mode: SharingMode<&'a [u32]>,

    /// A pre-transform to apply to the output image before it is presented to the surface.
    pub pre_transform: SurfaceTransform,
}

/// A swapchain that can be used to present images to a surface.
pub struct Swapchain {
    /// The surface that the swapchain presents images to.
    surface: Arc<Surface>,
    /// The device that owns this swapchain.
    device: Arc<Device>,

    /// The Vulkan handle for the swapchain.
    handle: vk::SwapchainKHR,
}

impl Swapchain {
    /// Creates a new [`Swapchain`] instance.
    pub fn new(device: Arc<Device>, surface: Arc<Surface>, desc: SwapchainDesc) -> Result<Self> {
        create_swapchain(&desc, device, surface, vk::SwapchainKHR::null())
    }

    /// Re-creates this swapchain with the provided description.
    pub fn recreate(self, desc: SwapchainDesc) -> Result<Self> {
        let this = ManuallyDrop::new(self);

        create_swapchain(
            &desc,
            this.device.clone(),
            this.surface.clone(),
            this.handle,
        )
    }

    /// Returns the surface that this swapchain presents images to.
    #[inline(always)]
    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }

    /// Returns the instance that owns this swapchain.
    #[inline(always)]
    pub fn instance(&self) -> &Arc<Instance> {
        self.surface.instance()
    }

    /// Returns the Vulkan handle for this swapchain.
    #[inline(always)]
    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }
}

/// Creates a new swapchain from the provided description.
fn create_swapchain(
    desc: &SwapchainDesc,
    device: Arc<Device>,
    surface: Arc<Surface>,
    old_swapchain: vk::SwapchainKHR,
) -> Result<Swapchain> {
    let mut handle = vk::SwapchainKHR::null();

    let mut create_info = vk::SwapchainCreateInfoKHR {
        clipped: desc.clipped as vk::Bool32,
        surface: surface.handle(),
        composite_alpha: vk::CompositeAlphaFlagsKHR::from_raw(desc.composite_alpha as u32),
        image_array_layers: desc.array_layers,
        image_color_space: vk::ColorSpaceKHR::from_raw(desc.color_space as i32),
        image_extent: vk::Extent2D {
            width: desc.extent[0],
            height: desc.extent[1],
        },
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        image_format: vk::Format::from_raw(desc.format as i32),
        image_usage: vk::ImageUsageFlags::from_raw(desc.usage.bits()),
        image_sharing_mode: vk::SharingMode::default(),
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
        min_image_count: desc.min_image_count,
        pre_transform: vk::SurfaceTransformFlagsKHR::from_raw(desc.pre_transform as u32),
        old_swapchain,
        p_next: std::ptr::null(),
        present_mode: vk::PresentModeKHR::from_raw(desc.present_mode as i32),
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
    };

    match desc.sharing_mode {
        SharingMode::Exclusive => {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }
        SharingMode::Concurrent(indices) => {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = indices.len() as u32;
            create_info.p_queue_family_indices = indices.as_ptr();
        }
    };

    let ret = unsafe {
        (device.fns().create_swapchain)(
            device.handle(),
            &create_info,
            std::ptr::null(),
            &mut handle,
        )
    };

    if ret != vk::Result::SUCCESS {
        return Err(ret.into());
    }

    Ok(Swapchain {
        handle,
        device,
        surface,
    })
}
