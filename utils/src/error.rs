// pub type Result<T> = core::result::Result<T, Error>;

// #[non_exhaustive]
// #[derive(Debug, Clone)]
// pub enum Error {
// Close,
//
// Environment(crate::environment::Error),
// Texture(crate::texture::Error),
// Buffer(crate::buffers::Error),
// Shader(crate::shader_program::Error),
// Import(crate::modelling::ImportError),
// Window(crate::window::Error),
// }

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
        impl std::error::Error for Error {}

        //  // derive Into for Error
        // impl From<$pth> for $crate::error::Error {
        // fn from(value: $pth) -> Self {
        // Self::$name(value)
        // }
        // }
    };
}
