#[macro_export]
macro_rules! error_boilerplate {
    ($pth:path) => {
        // Display IS debug
        impl core::fmt::Display for $pth {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                core::fmt::Debug::fmt(self, f)
            }
        }

        // Derive Error for Error
        impl std::error::Error for $pth {}

        impl From<!> for $pth {
            fn from(value: !) -> Self {
                value
            }
        }
    };

    (<$($lt:lifetime),*> $pth: path) => {
        impl<$($lt,)*> core::fmt::Display for $pth {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                core::fmt::Debug::fmt(self, f)
            }
        }

        // Derive Error for Error
        impl<$($lt,)*> std::error::Error for $pth {}

        impl<$($lt,)*> From<!> for $pth {
            fn from(value: !) -> Self {
                value
            }
        }

    };

    ({$($typ_req:tt)*} $(<$($lt:lifetime),*>)? $pth: path) => {
        impl<$($($lt,)*)? $($typ_req)*> core::fmt::Display for $pth {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                core::fmt::Debug::fmt(self, f)
            }
        }

        // Derive Error for Error
        impl<$($($lt,)*)? $($typ_req)*> std::error::Error for $pth {}

        impl<$($($lt,)*)? $($typ_req)*> From<!> for $pth {
            fn from(value: !) -> Self {
                value
            }
        }

    };
}
