use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::sync::Arc;

use ash::vk;
use smallvec::SmallVec;

use crate::{
    ColorSpace, CompositeAlphas, Format, ImageUsages, Instance, PresentModes, Result, Surface,
    SurfaceCaps, SurfaceTransform, SurfaceTransforms,
};

/// A physical device.
#[derive(Debug, Clone)]
pub struct PhysicalDevice {
    /// The instance that owns this physical device.
    instance: Arc<Instance>,

    /// The handle to the physical device.
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    /// Creates a new [`PhysicalDevice`] instance.
    ///
    /// # Safety
    ///
    /// The provided handle must be valid and belong to the provided instance.
    #[inline]
    pub unsafe fn from_handle(instance: Arc<Instance>, handle: vk::PhysicalDevice) -> Self {
        Self { instance, handle }
    }

    /// Returns information about this [`PhysicalDevice`].
    #[doc(alias = "vkGetPhysicalDeviceProperties")]
    pub fn properties(&self) -> PhysicalDeviceInfo {
        let mut properties = MaybeUninit::<vk::PhysicalDeviceProperties>::uninit();

        unsafe {
            (self.instance.fns().get_physical_device_properties)(
                self.handle,
                properties.as_mut_ptr(),
            );
        }

        let properties = unsafe { properties.assume_init_ref() };

        let name_cstr = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
        let name = name_cstr.to_str().unwrap_or_default().into();

        PhysicalDeviceInfo {
            api_version: properties.api_version,
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: match properties.device_type {
                vk::PhysicalDeviceType::OTHER => DeviceType::Other,
                vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGpu,
                vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGpu,
                vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGpu,
                vk::PhysicalDeviceType::CPU => DeviceType::Cpu,
                unknown => unreachable!("unknown device type: {}", unknown.as_raw()),
            },
            name,
        }
    }

    /// Returns the list of present modes that the provided surface supports with this physical
    /// device.
    pub fn surface_present_modes(&self, surface: &Surface) -> Result<PresentModes> {
        let mut list = SmallVec::<[vk::PresentModeKHR; 8]>::default();

        let ret = unsafe {
            crate::utility::read_into_vector(&mut list, |count, data| {
                (self
                    .instance
                    .fns()
                    .get_physical_device_surface_present_modes)(
                    self.handle,
                    surface.handle(),
                    count,
                    data,
                )
            })
        };

        if ret != vk::Result::SUCCESS {
            Err(ret.into())
        } else {
            let modes = list
                .iter()
                .copied()
                .map(|x| match x {
                    vk::PresentModeKHR::IMMEDIATE => PresentModes::IMMEDIATE,
                    vk::PresentModeKHR::MAILBOX => PresentModes::MAILBOX,
                    vk::PresentModeKHR::FIFO => PresentModes::FIFO,
                    vk::PresentModeKHR::FIFO_RELAXED => PresentModes::FIFO_RELAXED,
                    unknown => unreachable!("unknown present mode: {}", unknown.as_raw()),
                })
                .collect();

            Ok(modes)
        }
    }

    /// Returns an iterator over the list of supported surface formats for the provided surface.
    #[doc(alias = "vkGetPhysicalDeviceSurfaceFormats")]
    pub fn surface_supported_formats(
        &self,
        surface: &Surface,
    ) -> Result<impl Iterator<Item = (Format, ColorSpace)>> {
        assert!(Arc::ptr_eq(self.instance(), surface.instance()));

        let mut vec = Vec::new();

        let ret = unsafe {
            crate::utility::read_into_vector(&mut vec, |count, data| {
                (self.instance.fns().get_physical_device_surface_formats)(
                    self.handle,
                    surface.handle(),
                    count,
                    data,
                )
            })
        };

        if ret != vk::Result::SUCCESS {
            Err(ret.into())
        } else {
            let iter = vec.into_iter().map(|surface_format| {
                (
                    Format::from_raw(surface_format.format),
                    ColorSpace::from_raw(surface_format.color_space),
                )
            });

            Ok(iter)
        }
    }

    /// Returns the capabilities of a surface, when rendered to by this physical device.
    pub fn surface_capabilities(&self, surface: &Surface) -> Result<SurfaceCaps> {
        assert!(Arc::ptr_eq(self.instance(), surface.instance()));

        let mut caps = MaybeUninit::<vk::SurfaceCapabilitiesKHR>::uninit();

        let ret = unsafe {
            (self.instance.fns().get_physical_device_surface_capabilities)(
                self.handle,
                surface.handle(),
                caps.as_mut_ptr(),
            )
        };

        if ret != vk::Result::SUCCESS {
            Err(ret.into())
        } else {
            let caps = unsafe { caps.assume_init_ref() };

            Ok(SurfaceCaps {
                min_image_count: caps.min_image_count,
                max_image_count: match caps.max_image_count {
                    0 => None,
                    x => Some(x),
                },
                current_extent: match (caps.current_extent.width, caps.current_extent.height) {
                    (0xFFFFFFFF, _) | (_, 0xFFFFFFFF) => None,
                    _ => Some([caps.current_extent.width, caps.current_extent.height]),
                },
                min_image_extent: [caps.min_image_extent.width, caps.min_image_extent.height],
                max_image_extent: [caps.max_image_extent.width, caps.max_image_extent.height],
                max_image_array_layers: caps.max_image_array_layers,
                supported_transforms: SurfaceTransforms::from_bits_retain(
                    caps.supported_transforms.as_raw(),
                ),
                current_transform: match caps.current_transform {
                    vk::SurfaceTransformFlagsKHR::IDENTITY => SurfaceTransform::Identity,
                    vk::SurfaceTransformFlagsKHR::ROTATE_90 => SurfaceTransform::Rotate90,
                    vk::SurfaceTransformFlagsKHR::ROTATE_180 => SurfaceTransform::Rotate180,
                    vk::SurfaceTransformFlagsKHR::ROTATE_270 => SurfaceTransform::Rotate270,
                    vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR => {
                        SurfaceTransform::HorizontalMirror
                    }
                    vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_90 => {
                        SurfaceTransform::HorizontalMirrorRotate90
                    }
                    vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_180 => {
                        SurfaceTransform::HorizontalMirrorRotate180
                    }
                    vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_270 => {
                        SurfaceTransform::HorizontalMirrorRotate270
                    }
                    vk::SurfaceTransformFlagsKHR::INHERIT => SurfaceTransform::Inherit,
                    unknown => unreachable!("unknown surface transform: {}", unknown.as_raw()),
                },
                supported_composite_alpha: CompositeAlphas::from_bits_retain(
                    caps.supported_composite_alpha.as_raw(),
                ),
                supported_usage: ImageUsages::from_bits_retain(caps.supported_usage_flags.as_raw()),
            })
        }
    }

    /// Returns the instance that owns this physical device.
    #[inline(always)]
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    /// Returns the Vulkan handle for this physical device.
    #[inline(always)]
    pub fn handle(&self) -> vk::PhysicalDevice {
        self.handle
    }
}

/// Stores information about a physical device.
#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    /// The name of the device.
    pub name: Box<str>,
    /// The version of the Vulkan API that the device implements.
    pub api_version: u32,
    /// The version of the driver that the device uses.
    pub driver_version: u32,
    /// The ID of the vendor that created the device.
    pub vendor_id: u32,
    /// The ID of the device.
    pub device_id: u32,
    /// The type of the device.
    pub device_type: DeviceType,
}

/// The type of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// The type of the device is unknown.
    Other,
    /// The GPU is integrated or tightly coupled with the host CPU.
    IntegratedGpu,
    /// The GPU is typically a separate processor connected to the host via an interlink.
    DiscreteGpu,
    /// The GPU is a virtual node in a virtualization environment.
    VirtualGpu,
    /// The device is running on the same processor as the host.
    Cpu,
}
