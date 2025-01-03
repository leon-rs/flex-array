use core::ptr;

use crate::FlexArray;

unsafe impl<#[may_dangle] T, const CAP: usize> Drop for FlexArray<T, CAP> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len));
        }
    }
}

impl<T: Copy, const CAP: usize> Clone for FlexArray<T, CAP> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { ptr::read(self) }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        unsafe {
            ptr::copy_nonoverlapping(source, self, 1);
        }
    }
}
