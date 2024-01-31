use ash::vk;
use bitflags::bitflags;

bitflags! {
    /// Flags specifying a collection of [`ImageUsage`]s.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ImageUsages: u32 {
        const TRANSFER_SRC = vk::ImageUsageFlags::TRANSFER_SRC.as_raw();
        const TRANSFER_DST = vk::ImageUsageFlags::TRANSFER_DST.as_raw();
        const SAMPLED = vk::ImageUsageFlags::SAMPLED.as_raw();
        const STORAGE = vk::ImageUsageFlags::STORAGE.as_raw();
        const COLOR_ATTACHMENT = vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw();
        const DEPTH_STENCIL_ATTACHMENT = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT.as_raw();
        const TRANSIENT_ATTACHMENT = vk::ImageUsageFlags::TRANSIENT_ATTACHMENT.as_raw();
        const INPUT_ATTACHMENT = vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw();
    }
}

impl From<ImageUsage> for ImageUsages {
    fn from(value: ImageUsage) -> Self {
        match value {
            ImageUsage::TransferSrc => Self::TRANSFER_SRC,
            ImageUsage::TransferDst => Self::TRANSFER_DST,
            ImageUsage::Sampled => Self::SAMPLED,
            ImageUsage::Storage => Self::STORAGE,
            ImageUsage::ColorAttachment => Self::COLOR_ATTACHMENT,
            ImageUsage::DepthStencilAttachment => Self::DEPTH_STENCIL_ATTACHMENT,
            ImageUsage::TransientAttachment => Self::TRANSIENT_ATTACHMENT,
            ImageUsage::InputAttachment => Self::INPUT_ATTACHMENT,
        }
    }
}

/// A possible usage for an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ImageUsage {
    TransferSrc = vk::ImageUsageFlags::TRANSFER_SRC.as_raw(),
    TransferDst = vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
    Sampled = vk::ImageUsageFlags::SAMPLED.as_raw(),
    Storage = vk::ImageUsageFlags::STORAGE.as_raw(),
    ColorAttachment = vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw(),
    DepthStencilAttachment = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT.as_raw(),
    TransientAttachment = vk::ImageUsageFlags::TRANSIENT_ATTACHMENT.as_raw(),
    InputAttachment = vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw(),
}
