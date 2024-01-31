use ash::vk;

/// The encoding format of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Rgba8Unorm,
    Bgra8Unorm,
    Srgb8Srgb,
    Bgra8Srgb,
}

impl Format {
    /// Converts the provided raw Vulkan format and turns it into a [`Format`].
    pub(crate) fn from_raw(raw: vk::Format) -> Self {
        match raw {
            vk::Format::R8G8B8A8_UNORM => Self::Rgba8Unorm,
            vk::Format::B8G8R8A8_UNORM => Self::Bgra8Unorm,
            vk::Format::R8G8B8A8_SRGB => Self::Srgb8Srgb,
            vk::Format::B8G8R8A8_SRGB => Self::Bgra8Srgb,
            _ => unreachable!("unsupported format: {}", raw.as_raw()),
        }
    }
}

/// The color-space associated with a color encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    Srgb,
}

impl ColorSpace {
    /// Converts the provided raw Vulkan color space and turns it into a [`ColorSpace`].
    pub(crate) fn from_raw(raw: vk::ColorSpaceKHR) -> Self {
        match raw {
            vk::ColorSpaceKHR::SRGB_NONLINEAR => Self::Srgb,
            _ => unreachable!("unsupported color space: {}", raw.as_raw()),
        }
    }
}
