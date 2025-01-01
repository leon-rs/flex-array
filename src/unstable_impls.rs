use core::ptr;

use crate::FlexArray;

unsafe impl<#[may_dangle] T, const CAP: usize> Drop for FlexArray<T, CAP> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len));
        }
    }
}
