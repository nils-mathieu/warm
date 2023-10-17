//! Defines the [`RenderPass`] type, which implements the [`SurfaceContents`] trait.

use std::any::TypeId;
use std::fmt;
use std::sync::Arc;

use ash::vk;

use crate::gpu::Gpu;
use crate::surface::{FrameContext, ImagesInfo, SurfaceContents};
use crate::utility::ScopeGuard;
use crate::VulkanError;

pub mod attachment;
pub mod subpass;

use self::attachment::{Attachment, AttachmentList};
use self::subpass::{Subpass, SubpassList};

mod error;

pub use self::error::*;

/// Stores information about the output image of the [`RenderPass`].
struct OutputInfo {
    /// The width of the output image.
    pub width: u32,
    /// The height of the output image.
    pub height: u32,
    /// The format of the output image.
    pub format: vk::Format,
}

/// Contains data that's duplicated for each frame of the swapchain.
#[derive(Debug)]
struct PerFrame {
    /// A framebuffer that includes all the attachments of the render pass.
    ///
    /// This framebuffer may be removed. In that case, it will be set to `vk::Framebuffer::null()`.
    framebuffer: vk::Framebuffer,
    /// The command buffer responsible for recording the commands for this frame.
    command_buffer: vk::CommandBuffer,
    /// The fence that is signaled when the command buffer is finished executing.
    fence: vk::Fence,
    /// The semaphore that is signaled when the command buffer is finished executing.
    semaphore: vk::Semaphore,
}

impl PerFrame {
    /// Creates a new [`PerFrame`] instance.
    fn new(gpu: &Gpu, pool: vk::CommandPool) -> Result<Self, VulkanError> {
        let fence = create_fence(gpu, true)?;
        let fence = ScopeGuard::new(fence, |f| unsafe {
            gpu.vk_fns().destroy_fence(gpu.vk_device(), f)
        });
        let semaphore = create_semaphore(gpu)?;
        let semaphore = ScopeGuard::new(semaphore, |s| unsafe {
            gpu.vk_fns().destroy_semaphore(gpu.vk_device(), s)
        });
        let command_buffer = create_command_buffer(gpu, pool)?;
        let command_buffer = ScopeGuard::new(command_buffer, |cb| unsafe {
            gpu.vk_fns()
                .free_command_buffers(gpu.vk_device(), pool, &[cb])
        });

        Ok(Self {
            command_buffer: ScopeGuard::defuse(command_buffer),
            fence: ScopeGuard::defuse(fence),
            framebuffer: vk::Framebuffer::null(),
            semaphore: ScopeGuard::defuse(semaphore),
        })
    }

    /// Removes the framebuffer object of this instance.
    ///
    /// # Safety
    ///
    /// - The [`Gpu`] instance must be the same one that was used to create the framebuffer.
    unsafe fn remove_framebuffer(&mut self, gpu: &Gpu) {
        unsafe {
            gpu.vk_fns()
                .destroy_framebuffer(gpu.vk_device(), self.framebuffer);
            self.framebuffer = vk::Framebuffer::null();
        }
    }

    /// Creates a framebuffer for the render pass.
    ///
    /// # Safety
    ///
    /// - The [`Gpu`] instance must be the same one that was used to create the framebuffer.
    unsafe fn place_framebuffer(
        &mut self,
        gpu: &Gpu,
        render_pass: vk::RenderPass,
        info: &OutputInfo,
        views: &[vk::ImageView],
    ) -> Result<(), VulkanError> {
        self.framebuffer = create_framebuffer(gpu, views, render_pass, info)?;
        Ok(())
    }

    /// Destroys the resources used by this frame.
    ///
    /// # Safety
    ///
    /// - The [`Gpu`] instance must be the same one that was used to create the resources.
    unsafe fn destroy(&mut self, gpu: &Gpu, pool: vk::CommandPool) {
        unsafe {
            if self.framebuffer != vk::Framebuffer::null() {
                gpu.vk_fns()
                    .destroy_framebuffer(gpu.vk_device(), self.framebuffer);
            }

            gpu.vk_fns().destroy_fence(gpu.vk_device(), self.fence);
            gpu.vk_fns()
                .destroy_semaphore(gpu.vk_device(), self.semaphore);
            gpu.vk_fns()
                .free_command_buffers(gpu.vk_device(), pool, &[self.command_buffer]);
        }
    }
}

/// Creates a semaphore.
fn create_semaphore(gpu: &Gpu) -> Result<vk::Semaphore, VulkanError> {
    let info = vk::SemaphoreCreateInfo::default();

    unsafe { gpu.vk_fns().create_semaphore(gpu.vk_device(), &info) }
}

/// Creates a fence.
fn create_fence(gpu: &Gpu, signaled: bool) -> Result<vk::Fence, VulkanError> {
    let info = vk::FenceCreateInfo {
        flags: if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        },
        ..Default::default()
    };

    unsafe { gpu.vk_fns().create_fence(gpu.vk_device(), &info) }
}

/// Creates a new command buffer.
fn create_command_buffer(
    gpu: &Gpu,
    pool: vk::CommandPool,
) -> Result<vk::CommandBuffer, VulkanError> {
    let info = vk::CommandBufferAllocateInfo {
        command_buffer_count: 1,
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        ..Default::default()
    };

    let mut result = vk::CommandBuffer::null();

    unsafe {
        gpu.vk_fns()
            .allocate_command_buffers(gpu.vk_device(), &info, &mut result)?;
    }

    Ok(result)
}

/// Creates a new framebuffer for the render pass.
fn create_framebuffer(
    gpu: &Gpu,
    views: &[vk::ImageView],
    render_pass: vk::RenderPass,
    info: &OutputInfo,
) -> Result<vk::Framebuffer, VulkanError> {
    let info = vk::FramebufferCreateInfo {
        attachment_count: views.len() as u32,
        p_attachments: views.as_ptr(),
        flags: vk::FramebufferCreateFlags::empty(),
        height: info.height,
        width: info.width,
        layers: 1,
        render_pass,
        ..Default::default()
    };

    unsafe { gpu.vk_fns().create_framebuffer(gpu.vk_device(), &info) }
}

/// An implementation of [`SurfaceContents`] that uses a render pass to render the frames to
/// present to a surface.
pub struct RenderPass<Attachments, Subpasses> {
    /// The GPU that the render pass is associated with.
    gpu: Arc<Gpu>,

    /// The state required to keep the render pass attachments alive.
    attachments: Attachments,
    /// The state required to run the subpasses of the render pass.
    subpasses: Subpasses,
    /// The per-frame data.
    per_frame: Vec<PerFrame>,

    /// A command pool responsible for allocating the command buffers used to record the commands
    /// for each frame.
    command_pool: vk::CommandPool,
    /// The render pass used to render the frames.
    render_pass: vk::RenderPass,

    /// Information about the output image of the render pass.
    output_info: OutputInfo,
}

impl<Attachments, Subpasses> RenderPass<Attachments, Subpasses>
where
    Attachments: AttachmentList,
    Subpasses: SubpassList,
{
    /// Creates a new [`RenderPass`] instance.
    pub fn new(
        gpu: Arc<Gpu>,
        attachments: Attachments,
        subpasses: Subpasses,
    ) -> Result<Self, RenderPassError> {
        let mut builder = RenderPassBuilder::default();

        attachments.register(&mut builder)?;
        subpasses.register(&mut builder)?;

        let info = builder.build();

        let render_pass = unsafe { gpu.vk_fns().create_render_pass(gpu.vk_device(), &info)? };
        let render_pass = ScopeGuard::new(render_pass, |r| unsafe {
            gpu.vk_fns().destroy_render_pass(gpu.vk_device(), r)
        });

        let command_pool = create_command_pool(&gpu)?;
        let command_pool = ScopeGuard::new(command_pool, |cp| unsafe {
            gpu.vk_fns().destroy_command_pool(gpu.vk_device(), cp);
        });

        Ok(Self {
            attachments,
            subpasses,
            per_frame: Vec::new(),
            command_pool: ScopeGuard::defuse(command_pool),
            render_pass: ScopeGuard::defuse(render_pass),
            output_info: OutputInfo {
                width: 0,
                height: 0,
                format: vk::Format::UNDEFINED,
            },
            gpu,
        })
    }
}

unsafe impl<Attachments, Subpasses> SurfaceContents for RenderPass<Attachments, Subpasses>
where
    Attachments: AttachmentList,
    Subpasses: SubpassList,
{
    type Args<'a> = RenderPassArgs<'a, Attachments, Subpasses>;

    unsafe fn notify_destroy_images(&mut self) {
        for per_frame in &self.per_frame {
            unsafe {
                let _ = self.gpu.vk_fns().wait_for_fences(
                    self.gpu.vk_device(),
                    &[per_frame.fence],
                    true,
                    u64::MAX,
                );
            }
        }

        self.attachments.notify_destroying_output();

        for per_frame in &mut self.per_frame {
            unsafe {
                per_frame.remove_framebuffer(&self.gpu);
            }
        }
    }

    unsafe fn notify_new_images(&mut self, info: ImagesInfo) -> Result<(), VulkanError> {
        use std::cmp::Ordering::*;

        self.output_info.width = info.width;
        self.output_info.height = info.height;
        self.output_info.format = info.format;

        match info.images.len().cmp(&self.per_frame.len()) {
            Less => {
                // Remove the per-frame data that's no longer needed.

                for mut per_frame in self.per_frame.drain(info.images.len()..) {
                    unsafe { per_frame.destroy(&self.gpu, self.command_pool) };
                }
            }
            Equal => (),
            Greater => {
                // Add new per-frame data for the new images.

                for _ in self.per_frame.len()..info.images.len() {
                    let per_frame = PerFrame::new(&self.gpu, self.command_pool)?;
                    self.per_frame.push(per_frame);
                }
            }
        }

        self.attachments.notify_output_changed(&info)?;

        // Restore the framebuffers.

        for (index, per_frame) in self.per_frame.iter_mut().enumerate() {
            let views = unsafe { self.attachments.image_views(index) };

            unsafe {
                per_frame.place_framebuffer(
                    &self.gpu,
                    self.render_pass,
                    &self.output_info,
                    views.as_ref(),
                )?;
            }
        }

        Ok(())
    }

    unsafe fn render(
        &mut self,
        ctx: &mut FrameContext,
        args: Self::Args<'_>,
    ) -> Result<(), VulkanError> {
        let per_frame = unsafe { self.per_frame.get_unchecked_mut(ctx.image_index()) };

        //
        // 1. Acquire and begin recording commands in the command buffer of the frame.
        //
        unsafe {
            self.gpu.vk_fns().wait_for_fences(
                self.gpu.vk_device(),
                &[per_frame.fence],
                true,
                u64::MAX,
            )?;
            self.gpu.vk_fns().reset_command_buffer(
                per_frame.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            )?;

            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };

            self.gpu
                .vk_fns()
                .begin_command_buffer(per_frame.command_buffer, &begin_info)?;
        }

        //
        // 2. Start the render pass and record all subpasses.
        //
        let clear_values = Attachments::build_clear_values(args.clear_values);
        let clear_values: &[vk::ClearValue] = clear_values.as_ref();

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            framebuffer: per_frame.framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: self.output_info.width,
                    height: self.output_info.height,
                },
            },
            render_pass: self.render_pass,
            ..Default::default()
        };

        unsafe {
            self.gpu.vk_fns().cmd_begin_render_pass(
                per_frame.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }

        self.subpasses.record(args.args, || unsafe {
            self.gpu
                .vk_fns()
                .cmd_next_subpass(per_frame.command_buffer, vk::SubpassContents::INLINE);
        });

        unsafe {
            self.gpu
                .vk_fns()
                .cmd_end_render_pass(per_frame.command_buffer);
        }

        //
        // 3. Reset the fence and submit the command buffer.
        //
        unsafe {
            self.gpu
                .vk_fns()
                .end_command_buffer(per_frame.command_buffer)?;
            self.gpu
                .vk_fns()
                .reset_fences(self.gpu.vk_device(), &[per_frame.fence])?;
        }

        let wait_semaphores = [ctx.acquire_semaphore()];
        let wait_dst_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = [vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &per_frame.command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &per_frame.semaphore,
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_dst_stages.as_ptr(),
            ..Default::default()
        }];

        unsafe {
            self.gpu
                .vk_fns()
                .queue_submit(self.gpu.vk_queue(), &submit_info, per_frame.fence)?;
        }

        ctx.wait_semaphores_mut().push(per_frame.semaphore);

        Ok(())
    }
}

impl<A: fmt::Debug, S: fmt::Debug> fmt::Debug for RenderPass<A, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderPass")
            .field("attachments", &self.attachments)
            .field("subpasses", &self.subpasses)
            .finish_non_exhaustive()
    }
}

impl<A, S> Drop for RenderPass<A, S> {
    fn drop(&mut self) {
        unsafe {
            for mut per_frame in self.per_frame.drain(..) {
                per_frame.destroy(&self.gpu, self.command_pool);
            }

            self.gpu
                .vk_fns()
                .destroy_render_pass(self.gpu.vk_device(), self.render_pass);
            self.gpu
                .vk_fns()
                .destroy_command_pool(self.gpu.vk_device(), self.command_pool);
        }
    }
}

/// The arguments passed to the [`render`](SurfaceContents::render) method of a [`RenderPass`].
#[derive(Debug)]
pub struct RenderPassArgs<'a, Attachments, Subpasses>
where
    Attachments: AttachmentList,
    Subpasses: SubpassList,
{
    /// A tuple of clear values for each attachment of the render pass.
    pub clear_values: Attachments::ClearValues,
    /// The arguments passed to the subpasses.
    pub args: Subpasses::Args<'a>,
}

/// Contains the state required to create a [`vk::RenderPassCreateInfo`] instance from an
/// [`AttachmentList`] and [`SubpassList`] implementations.
///
/// An instance of this type is passed to [`Subpass`] implementations.
#[derive(Debug, Default)]
pub struct RenderPassBuilder {
    /// The list of all requested attachments.
    attachment_descs: Vec<vk::AttachmentDescription>,
    /// The list of all requested attachment references.
    attachment_ids: Vec<TypeId>,

    /// The list of all requested attachment references.
    attachment_refs: Vec<vk::AttachmentReference>,
    /// The lsit of all requested attachment indices.
    attachments: Vec<u32>,
    /// The list of all requested subpasses.
    subpass_descs: Vec<vk::SubpassDescription>,

    /// The dependencies between subpasses.
    dependencies: Vec<vk::SubpassDependency>,
}

impl RenderPassBuilder {
    /// Registers an attachment with the provided builder.
    pub fn register_attachment<A: Attachment>(
        &mut self,
        attachment: &A,
    ) -> Result<(), RenderPassError> {
        let desc = attachment.description()?;

        self.attachment_descs.push(desc);
        self.attachment_ids.push(TypeId::of::<A>());

        Ok(())
    }

    /// Requests an attachment, adding an attachment reference in the list of attachment
    /// references.
    ///
    /// # Errors
    ///
    /// This function returns `None` if the attachment is not found.
    pub fn request_attachment_ref<A: Attachment>(
        &mut self,
        layout: vk::ImageLayout,
    ) -> Option<usize> {
        self._request_attachment_ref(TypeId::of::<A>(), layout)
    }

    fn _request_attachment_ref(&mut self, id: TypeId, layout: vk::ImageLayout) -> Option<usize> {
        let attachment_ref = vk::AttachmentReference {
            attachment: self.attachment_ids.iter().position(|i| i == &id)? as u32,
            layout,
        };

        let ret = self.attachment_refs.len();
        self.attachment_refs.push(attachment_ref);

        Some(ret)
    }

    /// Requests an attachment by its [`TypeId`].
    pub fn request_attachment<A: Attachment>(&mut self) -> Option<usize> {
        self._request_attachment(TypeId::of::<A>())
    }

    #[inline]
    fn _request_attachment(&mut self, id: TypeId) -> Option<usize> {
        let index = self.attachment_ids.iter().position(|i| i == &id)?;

        let ret = self.attachments.len();
        self.attachments.push(index as u32);
        Some(ret)
    }

    /// Registers a subpass.
    #[rustfmt::skip]
    pub fn register_subpass<S: Subpass>(&mut self, subpass: &S) -> Result<(), RenderPassError> {
        let desc = subpass.register(self)?;

        self.subpass_descs.push(vk::SubpassDescription {
            color_attachment_count: desc.color_attachment_count as u32,
            p_color_attachments: desc.first_color_attachment as *const _,
            p_resolve_attachments: std::ptr::null(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            flags: vk::SubpassDescriptionFlags::empty(),
            input_attachment_count: desc.input_attachment_count as u32,
            p_input_attachments: desc.first_input_attachment as *const _,
            p_depth_stencil_attachment: desc.depth_stencil_attachment.unwrap_or(usize::MAX) as *const _,
            preserve_attachment_count: desc.preserve_attachment_count as u32,
            p_preserve_attachments: desc.first_preserve_attachment as *const _,
        });

        Ok(())
    }

    /// Builds a [`vk::RenderPassCreateInfo`] instance from the registered attachments and
    /// subpasses.
    ///
    /// Note that the returned instance is only valid as long as the builder is not modified
    /// or dropped.
    pub fn build(&mut self) -> vk::RenderPassCreateInfo {
        // Fix the subpass descriptions.

        for desc in &mut self.subpass_descs {
            if desc.color_attachment_count > 0 {
                desc.p_color_attachments = self
                    .attachment_refs
                    .as_ptr()
                    .wrapping_add(desc.p_color_attachments as usize);
            } else {
                desc.p_color_attachments = std::ptr::null();
            }

            if desc.input_attachment_count > 0 {
                desc.p_input_attachments = self
                    .attachment_refs
                    .as_ptr()
                    .wrapping_add(desc.p_input_attachments as usize);
            } else {
                desc.p_input_attachments = std::ptr::null();
            }

            if desc.p_depth_stencil_attachment as usize != usize::MAX {
                desc.p_depth_stencil_attachment = self
                    .attachment_refs
                    .as_ptr()
                    .wrapping_add(desc.p_depth_stencil_attachment as usize);
            } else {
                desc.p_depth_stencil_attachment = std::ptr::null();
            }

            if desc.preserve_attachment_count > 0 {
                desc.p_preserve_attachments = self
                    .attachments
                    .as_ptr()
                    .wrapping_add(desc.p_preserve_attachments as usize);
            } else {
                desc.p_preserve_attachments = std::ptr::null();
            }
        }

        vk::RenderPassCreateInfo {
            attachment_count: self.attachment_descs.len() as u32,
            p_attachments: self.attachment_descs.as_ptr(),
            subpass_count: self.subpass_descs.len() as u32,
            p_subpasses: self.subpass_descs.as_ptr(),
            dependency_count: self.dependencies.len() as u32,
            p_dependencies: self.dependencies.as_ptr(),
            flags: vk::RenderPassCreateFlags::empty(),
            ..Default::default()
        }
    }
}

/// Creates a new command pool.
fn create_command_pool(gpu: &Gpu) -> Result<vk::CommandPool, VulkanError> {
    let info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
            | vk::CommandPoolCreateFlags::TRANSIENT,
        queue_family_index: gpu.vk_queue_family(),
        ..Default::default()
    };

    unsafe { gpu.vk_fns().create_command_pool(gpu.vk_device(), &info) }
}
