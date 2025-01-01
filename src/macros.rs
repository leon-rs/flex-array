macro_rules! __maybe_specialization {
    ($(#[$attrs:meta])* fn $name:ident($($args:tt)*) $(->$ret:ty)? $body:block) => {
        #[cfg(feature = "unstable")]
        $(#[$attrs])*
        default fn $name($($args)*) $(->$ret)? $body

        #[cfg(not(feature = "unstable"))]
        $(#[$attrs])*
        fn $name($($args)*) $(->$ret)? $body
    };
}

macro_rules! __impl_partial_eq {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
        where
            T: PartialEq<U>,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    };
}
