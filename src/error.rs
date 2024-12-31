use core::{
    error::Error,
    fmt::{Debug, Display},
};

macro_rules! error_message {
    () => {
        "attempted to push to a full FlexArray"
    };
}

pub struct CapacityExceededError<T = ()> {
    value: T,
}

impl<T> CapacityExceededError<T> {
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
    pub const fn const_simplify(self) -> CapacityExceededError {
        CapacityExceededError { value: () }
    }
}

impl<T> Display for CapacityExceededError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", error_message!())
    }
}

impl<T> Debug for CapacityExceededError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapacityExceededError: {}", error_message!())
    }
}

impl<T> Error for CapacityExceededError<T> {}
