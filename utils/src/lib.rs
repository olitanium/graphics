mod error;
// pub use error::Error;
// pub use error::Result;

#[macro_export]
macro_rules! getter {
    ($value:ident : $type:ty) => {
        pub fn $value(&self) -> &$type {
            &self.$value
        }
    };
}

#[macro_export]
macro_rules! builder {
    ($name:ident, $optname:ident : Option < $typ:ty >) => {
        pub fn $name<X: Into<$typ>>(mut self, $name: X) -> Self {
            self.$name = Some($name.into());
            self
        }

        pub fn $optname<X: Into<$typ>>(mut self, $optname: Option<X>) -> Self {
            self.$name = $optname.map(Into::into);
            self
        }
    };

    ($name:ident : Option < $typ:ty >) => {
        pub fn $name<X: Into<$typ>>(mut self, $name: X) -> Self {
            self.$name = Some($name.into());
            self
        }
    };

    ($name:ident : $typ:ty) => {
        pub fn $name<X: Into<$typ>>(mut self, $name: X) -> Self {
            self.$name = $name.into();
            self
        }
    };
}

#[macro_export]
macro_rules! new {
    () => {
        pub fn new() -> Self {
            Self::default()
        }
    };
}
