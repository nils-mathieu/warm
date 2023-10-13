use ash::vk;

use super::{PresentModes, SurfaceError};
use crate::gpu::Gpu;

/// Stores information about the swapchain.
///
/// This information is queryed once and is used when creating or re-creating the swapchain.
pub struct SwapchainInfo {
    /// The minimum number of images that the swapchain must have.
    pub min_image_count: u32,
    /// The alpha compositing mode selected for the swapchain.
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
    /// A transformation to apply to swapchain images before they are presented to the surface.
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
    /// The format of the swapchain images.
    pub format: vk::Format,
    /// The color space of the swapchain images.
    pub color_space: vk::ColorSpaceKHR,
    /// The present mode that should be used for the swapchain.
    pub present_modes: PresentModes,
}

/// Queries an instance of [`SwapchainInfo`].
pub fn query(gpu: &Gpu, surface: vk::SurfaceKHR) -> Result<SwapchainInfo, SurfaceError> {
    let caps = surface_caps(gpu, surface)?;

    sanitize_assumed_capabilities(&caps)?;
    let min_image_count = get_min_image_count(&caps);
    let composite_alpha = get_composite_alpha(&caps)?;
    let pre_transform = get_pre_transform(&caps);
    let surface_format = get_surface_format(gpu, surface)?;
    let present_modes = get_present_modes(gpu, surface)?;

    Ok(SwapchainInfo {
        min_image_count,
        composite_alpha,
        pre_transform,
        format: surface_format.format,
        color_space: surface_format.color_space,
        present_modes,
    })
}

/// Returns the capabilities of the surface provided by the physical device.
fn surface_caps(
    gpu: &Gpu,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceCapabilitiesKHR, SurfaceError> {
    unsafe {
        gpu.vk_fns()
            .get_physical_device_surface_capabilities(gpu.vk_physical_device(), surface)
            .map_err(|_| SurfaceError::UnexpectedVulkanBehavior)
    }
}

/// Returns the minimum number of images that the swapchain must have.
fn get_min_image_count(caps: &vk::SurfaceCapabilitiesKHR) -> u32 {
    if caps.min_image_count > 3 {
        caps.min_image_count
    } else if caps.max_image_count != 0 && caps.max_image_count < 3 {
        caps.max_image_count
    } else {
        3
    }
}

/// Check some of the values of the capabilities to make sure that they are supported.
fn sanitize_assumed_capabilities(caps: &vk::SurfaceCapabilitiesKHR) -> Result<(), SurfaceError> {
    if caps.max_image_array_layers < 1 {
        return Err(SurfaceError::NotSupported);
    }

    if !caps
        .supported_usage_flags
        .contains(vk::ImageUsageFlags::COLOR_ATTACHMENT)
    {
        return Err(SurfaceError::NotSupported);
    }

    Ok(())
}

/// Returns the alpha compositing mode that should be used for the swapchain.
fn get_composite_alpha(
    caps: &vk::SurfaceCapabilitiesKHR,
) -> Result<vk::CompositeAlphaFlagsKHR, SurfaceError> {
    const ORDER_OF_PREFERENCE: &[vk::CompositeAlphaFlagsKHR] = &[
        vk::CompositeAlphaFlagsKHR::OPAQUE,
        vk::CompositeAlphaFlagsKHR::INHERIT,
        vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
    ];

    ORDER_OF_PREFERENCE
        .iter()
        .copied()
        .find(|&mode| caps.supported_composite_alpha.contains(mode))
        .ok_or(SurfaceError::NotSupported)
}

/// Returns the surface pre-transform to apply to swapchain images.
fn get_pre_transform(caps: &vk::SurfaceCapabilitiesKHR) -> vk::SurfaceTransformFlagsKHR {
    if caps
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        caps.current_transform
    }
}

/// Returns the prefered format for the swapchain images.
fn get_surface_format(
    gpu: &Gpu,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, SurfaceError> {
    unsafe {
        let mut formats = Vec::new();
        gpu.vk_fns()
            .get_physical_device_surface_formats(gpu.vk_physical_device(), surface, &mut formats)
            .map_err(|_| SurfaceError::UnexpectedVulkanBehavior)?;

        // NOTE:
        //  We're reversing the iterator to get the first format that we prefer (if multiple
        //  formats have the same score).
        formats
            .into_iter()
            .rev()
            .max_by_key(|sf| {
                let mut score = 0;

                if sf.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                    score += 100;
                }

                match sf.format {
                    vk::Format::R8G8B8A8_UNORM => score += 10,
                    vk::Format::B8G8R8A8_UNORM => score += 10,
                    _ => (),
                }

                score
            })
            .ok_or(SurfaceError::NotSupported)
    }
}

/// Returns the prefered present mode for the swapchain.
fn get_present_modes(gpu: &Gpu, surface: vk::SurfaceKHR) -> Result<PresentModes, SurfaceError> {
    let mut modes = Vec::new();
    unsafe {
        gpu.vk_fns()
            .get_physical_device_surface_present_modes(
                gpu.vk_physical_device(),
                surface,
                &mut modes,
            )
            .map_err(|_| SurfaceError::UnexpectedVulkanBehavior)?;
    }

    let ret = modes
        .into_iter()
        .filter_map(|mode| match mode {
            vk::PresentModeKHR::IMMEDIATE => Some(PresentModes::IMMEDIATE),
            vk::PresentModeKHR::MAILBOX => Some(PresentModes::MAILBOX),
            vk::PresentModeKHR::FIFO => Some(PresentModes::FIFO),
            vk::PresentModeKHR::FIFO_RELAXED => Some(PresentModes::FIFO_RELAXED),
            _ => None,
        })
        .collect();

    Ok(ret)
}
