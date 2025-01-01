use core::{
    error::Error,
    fmt::{Debug, Display},
};

pub struct CapacityExceededError<T = ()> {
    value: T,
}

impl<T> CapacityExceededError<T> {
    const ERROR_MESSAGE: &str = "capacity exceeded";

    #[inline]
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn simplify(self) -> CapacityExceededError {
        CapacityExceededError { value: () }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Copy> CapacityExceededError<T> {
    pub const fn const_into_inner(self) -> T {
        self.value
    }

    pub const fn const_simplify(self) -> CapacityExceededError {
        CapacityExceededError { value: () }
    }
}

impl<T> Display for CapacityExceededError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", Self::ERROR_MESSAGE)
    }
}

impl<T> Debug for CapacityExceededError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapacityExceededError: {}", Self::ERROR_MESSAGE)
    }
}

impl<T> Error for CapacityExceededError<T> {}
