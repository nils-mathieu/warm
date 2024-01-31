use std::mem::MaybeUninit;

use ash::vk;
use smallvec::SmallVec;

/// A trait for types that can be converted to a vector.
pub trait VectorLike {
    /// The item type stored in this vector.
    type Item;

    /// Attempts to reserve additional capacity for this vector.
    fn try_reserve(&mut self, additional: usize) -> Result<(), ()>;

    /// Returns a reference to the spare capacity of this vector.
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::Item>];

    /// Assumes that the next `additional` elements of the vector have been initialized.
    ///
    /// # Safety
    ///
    /// `additional` really must have been initialized.
    unsafe fn assume_init(&mut self, additional: usize);
}

impl<T> VectorLike for Vec<T> {
    type Item = T;

    #[inline]
    fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        Vec::try_reserve(self, additional).map_err(|_| ())
    }

    #[inline]
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        Vec::spare_capacity_mut(self)
    }

    #[inline]
    unsafe fn assume_init(&mut self, additional: usize) {
        Vec::set_len(self, self.len() + additional);
    }
}

impl<A: smallvec::Array> VectorLike for SmallVec<A> {
    type Item = A::Item;

    #[inline]
    fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        SmallVec::try_reserve(self, additional).map_err(|_| ())
    }

    #[inline]
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        unsafe {
            let p = SmallVec::as_mut_ptr(self).add(SmallVec::len(self));
            let len = SmallVec::capacity(self) - SmallVec::len(self);
            std::slice::from_raw_parts_mut(p as *mut MaybeUninit<Self::Item>, len)
        }
    }

    #[inline]
    unsafe fn assume_init(&mut self, additional: usize) {
        SmallVec::set_len(self, self.len() + additional);
    }
}

/// Some Vulkan function allow retrieving a list of values. This function allows reading those
/// values into a vector.
pub unsafe fn read_into_vector<V: VectorLike>(
    v: &mut V,
    mut f: impl FnMut(*mut u32, *mut V::Item) -> vk::Result,
) -> vk::Result {
    let mut count = 0;
    let mut ret;

    loop {
        ret = f(&mut count, core::ptr::null_mut());

        if ret != vk::Result::SUCCESS {
            return ret;
        }

        if count == 0 {
            return vk::Result::SUCCESS;
        }

        match v.try_reserve(count as usize) {
            Ok(()) => {}
            Err(()) => return vk::Result::ERROR_OUT_OF_HOST_MEMORY,
        }

        let spare_capacity = v.spare_capacity_mut();
        count = spare_capacity.len() as u32;
        ret = f(&mut count, spare_capacity.as_mut_ptr() as *mut V::Item);

        match ret {
            vk::Result::SUCCESS => break,
            vk::Result::INCOMPLETE => {}
            _ => return ret,
        }
    }

    v.assume_init(count as usize);

    vk::Result::SUCCESS
}
