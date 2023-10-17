//! Defines [`SemaphorePool`].

use std::ops::Deref;

use ash::prelude::VkResult;
use ash::vk;

use crate::gpu::Gpu;

/// A pool of [`vk::Semaphore`]s that can be used to cheaply synchronize operations.
///
/// Because this type is exclusively used internally, it does not include a [`Gpu`] instance to
/// properly implement [`Drop`]. Instead, one must manually call [`destroy`] to free the
/// [`vk::Semaphore`]s.
#[derive(Default)]
pub struct SemaphorePool(Vec<vk::Semaphore>);

impl SemaphorePool {
    /// Gets a [`vk::Semaphore`] from the pool.
    ///
    /// # Safety
    ///
    /// - The returned semaphore must be either returned back to the pool or properly destroyed
    /// before the [`Gpu`] instance is destroyed.
    ///
    /// - The same [`Gpu`] instance must be used for all calls to this function.
    pub unsafe fn get(&mut self, gpu: &Gpu) -> VkResult<SemaphoreInPool> {
        let semaphore = match self.0.pop() {
            Some(s) => s,
            None => unsafe {
                gpu.vk_fns()
                    .create_semaphore(gpu.vk_device(), &Default::default())?
            },
        };

        Ok(SemaphoreInPool {
            semaphore,
            pool: self,
        })
    }

    /// Gives back a semaphore to the pool.
    ///
    /// # Safety
    ///
    /// - The given semaphore must have been created using the same [`Gpu`] instance that was used
    /// to create the semaphores in the pool.
    pub unsafe fn give_back(&mut self, semaphore: vk::Semaphore) {
        self.0.push(semaphore);
    }

    /// Destroys the semaphores in the pool using the given [`Gpu`] instance.
    ///
    /// # Safety
    ///
    /// - The [`Gpu`] instance must be the same one that was used to create the semaphores.
    pub unsafe fn destroy(&mut self, gpu: &Gpu) {
        for semaphore in self.0.drain(..) {
            unsafe { gpu.vk_fns().destroy_semaphore(gpu.vk_device(), semaphore) };
        }
    }
}

/// A [`vk::Semaphore`] that's part of a [`SemaphorePool`].
pub struct SemaphoreInPool<'a> {
    semaphore: vk::Semaphore,
    pool: &'a mut SemaphorePool,
}

impl<'a> Deref for SemaphoreInPool<'a> {
    type Target = vk::Semaphore;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.semaphore
    }
}

impl<'a> Drop for SemaphoreInPool<'a> {
    fn drop(&mut self) {
        unsafe { self.pool.give_back(self.semaphore) };
    }
}
