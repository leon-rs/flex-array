#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(dropck_eyepatch))]

use core::{
    ops::{Deref, DerefMut},
    ptr, slice,
};

use error::CapacityExceededError;
use raw_buf::RawBuf;

mod error;
mod raw_buf;

#[cfg(feature = "unstable")]
#[path = "unstable_impls.rs"]
mod impls;
#[cfg(not(feature = "unstable"))]
#[path = "stable_impls.rs"]
mod impls;

#[repr(C)]
pub struct FlexArray<T, const CAP: usize> {
    len: usize,
    buf: RawBuf<T, CAP>,
}

impl<T, const CAP: usize> FlexArray<T, CAP> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            len: 0,
            buf: RawBuf::new(),
        }
    }

    #[inline]
    pub const fn zeroed() -> Self {
        Self {
            len: 0,
            buf: RawBuf::zeroed(),
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.capacity() == self.len()
    }

    #[inline]
    pub const fn has_space(&self) -> bool {
        self.len() < self.capacity()
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.buf.as_ptr()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr()
    }

    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    #[inline]
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
    }

    #[inline]
    pub const fn try_push(&mut self, value: T) -> Result<(), CapacityExceededError<T>> {
        if !self.has_space() {
            return Err(CapacityExceededError::new(value));
        }
        unsafe {
            self.push_unchecked(value);
        }
        Ok(())
    }

    #[inline]
    pub const fn push(&mut self, value: T) {
        if !self.has_space() {
            panic!("attempted to push to a full FlexArray");
        }
        unsafe {
            self.push_unchecked(value);
        }
    }

    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        unsafe { Some(self.pop_unchecked()) }
    }

    #[inline]
    pub fn clear(&mut self) {
        let elems: *mut [T] = self.as_mut_slice();
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }
}

impl<T: Copy, const CAP: usize> FlexArray<T, CAP> {
    #[inline]
    pub const fn const_clear(&mut self) {
        unsafe {
            self.set_len(0);
        }
    }
}

impl<T, const CAP: usize> FlexArray<T, CAP> {
    #[inline]
    const unsafe fn push_unchecked(&mut self, value: T) {
        unsafe {
            self.as_mut_ptr().add(self.len).write(value);
            self.len += 1;
        }
    }

    #[inline]
    const unsafe fn pop_unchecked(&mut self) -> T {
        unsafe {
            self.len -= 1;
            self.as_mut_ptr().add(self.len).read()
        }
    }
}

impl<T, const CAP: usize> Deref for FlexArray<T, CAP> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const CAP: usize> DerefMut for FlexArray<T, CAP> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
