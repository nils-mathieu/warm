use ash::vk;

/// The encoding format of a color.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {}

impl Format {
    /// Converts the provided raw Vulkan format and turns it into a [`Format`].
    pub(crate) fn from_raw(_raw: vk::Format) -> Self {
        todo!();
    }
}

/// The color-space associated with a color encoding format.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {}

impl ColorSpace {
    /// Converts the provided raw Vulkan color space and turns it into a [`ColorSpace`].
    pub(crate) fn from_raw(_raw: vk::ColorSpaceKHR) -> Self {
        todo!();
    }
}

/// A [`Format`] associated with a [`ColorSpace`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceFormat {
    /// The format of the color.
    pub format: Format,
    /// The color space of the color.
    pub color_space: ColorSpace,
}
