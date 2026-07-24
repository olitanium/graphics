mod error;
mod ref_or_owned;

pub use ref_or_owned::RefOwned;
mod zip_with;
pub use zip_with::ArrayZipWith;

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
    'a,
    U,
    T: Clone + 'a,
    I: IntoIterator<Item = &'a T>,
    Fo: IntoIterator<Item = usize>,
    F: FnMut(&'a T) -> Fo,
    O: FnMut(T) -> U,
>(
    iter: I,
    mut part_fn: F,
    mut manipulate_elem: O,
) -> Vec<Vec<U>> {
    // let mut out = vec![];
    //
    // for item in iter {
    // let indices = part_fn(&item);
    // if let Some(&max) = indices.iter().max() {
    // conditionally_resize_vector(&mut out, max + 1, Vec::new);
    //
    // for value in indices {
    // out[value].push(manipulate_elem(item.clone()))
    // }
    // }
    // }
    //
    // out

    iter.into_iter().fold(Vec::new(), |mut accum, item| {
        for index in part_fn(item) {
            let at_index = accum.get_at_or_extend(index, Vec::new);
            at_index.push(manipulate_elem(item.clone()));
        }

        accum
    })
}

// fn get_at_index_or_extend_with<T, F: FnMut() -> T>(
// vec: &mut Vec<T>,
// index: usize,
// new: F,
// ) -> &mut T {
// let Some(elem) = vec.get_mut(index) else {
// vec.resize_with(index + 1, new);
// return vec
// .last_mut()
// .expect("resize was to at least 1, because index + 1 >= 1");
// };
// elem
// }

fn get_at_index_or_extend_with<T, F: FnMut() -> T>(
    vec: &mut Vec<T>,
    index: usize,
    new: F,
) -> &mut T {
    if vec.len() <= index {
        // we need to extend
        // index + 1 > index >= len, so we don't worry about shrinking
        vec.resize_with(index + 1, new);
    }
    // we just made the vector index + 1 len, so index is the last element
    &mut vec[index]
}

pub trait GetAtOrExtend {
    type Out;

    fn get_at_or_extend<F: FnMut() -> Self::Out>(&mut self, index: usize, new: F)
    -> &mut Self::Out;
}

impl<T> GetAtOrExtend for Vec<T> {
    type Out = T;

    fn get_at_or_extend<F: FnMut() -> Self::Out>(
        &mut self,
        index: usize,
        new: F,
    ) -> &mut Self::Out {
        get_at_index_or_extend_with(self, index, new)
    }
}

mod test1 {
    use super::*;

    #[test]
    fn get_at_index() {
        let mut test = vec![0, 1, 2, 3, 4, 5];
        let ref_five = get_at_index_or_extend_with(&mut test, 5, || 0);
        assert_eq!(ref_five, &5);

        let ref_seven = get_at_index_or_extend_with(&mut test, 7, || 6);
        assert_eq!(ref_seven, &6);
        *ref_seven = 7;

        assert_eq!(test, [0, 1, 2, 3, 4, 5, 6, 7])
    }
}
