#[macro_export]
macro_rules! run_async {
    ($($code:tt)*) => {
        {
            crate::core::runtime::block_on(async move {
                $($code)*
            })
        }
    };
}

#[macro_export]
macro_rules! from_error {
    ($from:ty, $to_variant:ident) => {
        impl From<$from> for crate::utils::error::DbError {
            fn from(err: $from) -> Self {
                crate::utils::error::DbError::$to_variant(err)
            }
        }
    };
}
