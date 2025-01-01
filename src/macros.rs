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
