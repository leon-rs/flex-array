#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "unstable", feature(dropck_eyepatch))]
#![cfg_attr(feature = "unstable", feature(min_specialization))]

use core::{
    fmt,
    hash::Hash,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
    slice::{self, SliceIndex},
};

use error::CapacityExceededError;
use raw_buf::RawBuf;

mod error;
#[macro_use]
mod macros;
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

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len) as *mut MaybeUninit<T>,
                self.buf.capacity() - self.len,
            )
        }
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            if self.len < len {
                return;
            }
            let remaining_len = self.len - len;
            let s = ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            self.len = len;
            ptr::drop_in_place(s);
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

impl<T, const CAP: usize> FlexArray<T, CAP> {
    #[inline]
    fn from_array<const N: usize>(array: [T; N]) -> Self {
        let mut f = Self::new();
        unsafe {
            for b in array.into_iter().take(CAP) {
                f.push_unchecked(b);
            }
        }
        f
    }
}

impl<T: Clone, const CAP: usize> FlexArray<T, CAP> {
    fn from_slice(s: &[T]) -> Self {
        struct DropGuard<'a, T, const CAP: usize> {
            array: &'a mut FlexArray<T, CAP>,
            num_init: usize,
        }
        impl<'a, T, const CAP: usize> Drop for DropGuard<'a, T, CAP> {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    self.array.set_len(self.num_init);
                }
            }
        }

        let mut array = Self::new();
        let mut guard = DropGuard {
            array: &mut array,
            num_init: 0,
        };
        let slots = guard.array.spare_capacity_mut();
        for (i, b) in s.iter().enumerate().take(slots.len()) {
            guard.num_init = i;
            slots[i].write(b.clone());
        }
        let num_init = guard.num_init;
        core::mem::forget(guard);
        unsafe {
            array.set_len(num_init);
        }
        array
    }

    fn clone_from_slice(&mut self, source: &[T]) {
        let len = source.len().min(self.capacity());
        self.truncate(len);
        let (init, tail) = source.split_at(self.len());
        self.deref_mut().clone_from_slice(init);
        unsafe {
            for b in tail.iter().take(self.capacity() - self.len()) {
                self.push_unchecked(b.clone());
            }
        }
    }
}

impl<T: Clone, const CAP: usize> Clone for FlexArray<T, CAP> {
    __maybe_specialization!(
        #[inline]
        fn clone(&self) -> Self {
            Self::from_slice(self)
        }
    );

    __maybe_specialization!(
        #[inline]
        fn clone_from(&mut self, source: &Self) {
            self.clone_from_slice(source);
        }
    );
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

impl<T, I: SliceIndex<[T]>, const CAP: usize> Index<I> for FlexArray<T, CAP> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, const CAP: usize> IndexMut<I> for FlexArray<T, CAP> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T: Hash, const CAP: usize> Hash for FlexArray<T, CAP> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state);
    }
}

impl<T, const CAP: usize> Default for FlexArray<T, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug, const CAP: usize> fmt::Debug for FlexArray<T, CAP> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, const CAP: usize> AsRef<FlexArray<T, CAP>> for FlexArray<T, CAP> {
    #[inline]
    fn as_ref(&self) -> &FlexArray<T, CAP> {
        self
    }
}

impl<T, const CAP: usize> AsMut<FlexArray<T, CAP>> for FlexArray<T, CAP> {
    #[inline]
    fn as_mut(&mut self) -> &mut FlexArray<T, CAP> {
        self
    }
}

impl<T, const CAP: usize> AsRef<[T]> for FlexArray<T, CAP> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const CAP: usize> AsMut<[T]> for FlexArray<T, CAP> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone, const CAP: usize> From<&[T]> for FlexArray<T, CAP> {
    #[inline]
    fn from(value: &[T]) -> Self {
        Self::from_slice(value)
    }
}

impl<T: Clone, const CAP: usize> From<&mut [T]> for FlexArray<T, CAP> {
    #[inline]
    fn from(value: &mut [T]) -> Self {
        Self::from_slice(value)
    }
}

impl<T: Clone, const N: usize, const CAP: usize> From<&[T; N]> for FlexArray<T, CAP> {
    #[inline]
    fn from(value: &[T; N]) -> Self {
        Self::from_slice(value.as_slice())
    }
}

impl<T, const N: usize, const CAP: usize> From<[T; N]> for FlexArray<T, CAP> {
    #[inline]
    fn from(value: [T; N]) -> Self {
        Self::from_array(value)
    }
}
