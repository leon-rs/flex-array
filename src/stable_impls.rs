use core::ptr;

use crate::FlexArray;

impl<T, const CAP: usize> Drop for FlexArray<T, CAP> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len));
        }
    }
}

impl<T: Clone, const CAP: usize> Clone for FlexArray<T, CAP> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_slice(self)
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clone_from_slice(source);
    }
}
