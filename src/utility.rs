//! Some utility functions.

use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use ash::vk;

/// Containers that are like a vector and that can be filled with data.
pub trait VectorLike {
    /// The item type of the container.
    type Item;

    /// Reserves space for `additional` items and returns a pointer to the first item of the
    /// reserved space.
    fn reserve(&mut self, additional: usize) -> *mut Self::Item;

    /// Assumes that the container has been filled with `additional` items.
    ///
    /// # Safety
    ///
    /// Those items must be properly initialized.
    unsafe fn assume_init(&mut self, additional: usize);
}

impl<T> VectorLike for Vec<T> {
    type Item = T;

    fn reserve(&mut self, additional: usize) -> *mut Self::Item {
        self.reserve(additional);
        unsafe { self.as_mut_ptr().add(self.len()) }
    }

    unsafe fn assume_init(&mut self, additional: usize) {
        let len = self.len();
        unsafe { self.set_len(len + additional) };
    }
}

/// Reads the items provided by `read` into `container`.
///
/// # Safety
///
/// The `read` function must work in the following way.
///
/// ```rust
/// fn read(count: &mut u32, item: *mut u32) -> vk::Result
/// ```
///
/// - If `item` is null, then the number of items that will be read is written to `count`.
///
/// - If `item` is not null, then at most items are written to it. The maximum number of items
///   that can be written is the value that `count` points to.
///
/// - If enough items could be written to `item`, the function returns `vk::Result::SUCCESS`. If
///   the function returns `vk::Result::INCOMPLETE`, then the procedure is repeated. Otherwise,
///   the function returns the error.
pub unsafe fn read_into_vector<C, F>(container: &mut C, mut read: F) -> vk::Result
where
    C: VectorLike,
    F: FnMut(*mut u32, *mut C::Item) -> vk::Result,
{
    let mut count: u32;
    let mut result: vk::Result;

    loop {
        count = 0;
        result = read(&mut count, std::ptr::null_mut());
        if result != vk::Result::SUCCESS {
            return result;
        }

        let data = container.reserve(count as usize);
        result = read(&mut count, data);
        match result {
            vk::Result::SUCCESS => break,
            vk::Result::INCOMPLETE => continue,
            _ => return result,
        }
    }

    unsafe { container.assume_init(count as usize) };
    vk::Result::SUCCESS
}

/// A guard that runs a function when it is dropped.
#[derive(Debug)]
pub struct ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    subject: ManuallyDrop<T>,
    drop_fn: ManuallyDrop<F>,
}

impl<T, F> ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    /// Creates a new [`ScopeGuard`] that runs `drop_fn` when it is dropped.
    pub fn new(subject: T, drop_fn: F) -> Self {
        Self {
            subject: ManuallyDrop::new(subject),
            drop_fn: ManuallyDrop::new(drop_fn),
        }
    }

    /// Defuses the [`ScopeGuard`], returning the subject without running the destructor.
    pub fn defuse(this: Self) -> T {
        unsafe {
            let mut this = ManuallyDrop::new(this);

            let subject = ManuallyDrop::take(&mut this.subject);
            ManuallyDrop::drop(&mut this.drop_fn);

            subject
        }
    }
}

impl<T, F> Drop for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    fn drop(&mut self) {
        unsafe {
            let subject = ManuallyDrop::take(&mut self.subject);
            let drop_fn = ManuallyDrop::take(&mut self.drop_fn);

            drop_fn(subject);
        }
    }
}

impl<T, F> Deref for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.subject
    }
}

impl<T, F> DerefMut for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subject
    }
}
