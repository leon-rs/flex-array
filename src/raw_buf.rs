use core::{marker::PhantomData, mem::MaybeUninit};

#[repr(transparent)]
pub(crate) struct RawBuf<T, const CAP: usize> {
    buf: [MaybeUninit<T>; CAP],
    _marker: PhantomData<T>,
}

impl<T, const CAP: usize> RawBuf<T, CAP> {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; CAP],
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn zeroed() -> Self {
        Self {
            buf: [const { MaybeUninit::zeroed() }; CAP],
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn as_ptr(&self) -> *const T {
        self.buf.as_ptr() as *const _ as *const T
    }

    #[inline]
    pub(crate) const fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr() as *mut _ as *mut T
    }

    #[inline]
    pub(crate) const fn capacity(&self) -> usize {
        CAP
    }
}
