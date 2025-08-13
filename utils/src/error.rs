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
}
