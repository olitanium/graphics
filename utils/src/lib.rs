#![feature(never_type)]
mod error;
mod ref_or_owned;

pub use ref_or_owned::RefOwned;

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
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = Some($name);
            self
        }

        pub fn $optname(mut self, $optname: Option<$typ>) -> Self {
            self.$name = $optname;
            self
        }
    };

    ($name:ident : Option < $typ:ty >) => {
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = Some($name);
            self
        }
    };

    ($name:ident : $typ:ty) => {
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = $name;
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
