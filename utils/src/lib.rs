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

pub fn partition<
    U,
    T: Clone,
    I: IntoIterator<Item = T>,
    F: FnMut(&T) -> Vec<usize>,
    O: FnMut(T) -> U,
>(
    iter: I,
    mut part_fn: F,
    mut manipulate_elem: O,
) -> Vec<Vec<U>> {
    let mut out = vec![];

    for item in iter {
        let indices = part_fn(&item);
        if let Some(&max) = indices.iter().max() {
            conditionally_resize_vector(&mut out, max + 1, Vec::new);

            for value in indices {
                out[value].push(manipulate_elem(item.clone()))
            }
        }
    }

    out
}

fn conditionally_resize_vector<T, F: FnMut() -> T>(vec: &mut Vec<T>, len: usize, new: F) {
    if vec.len() < len {
        vec.resize_with(len, new);
    }
}
