#![feature(phantom_variance_markers)]

use std::marker::{PhantomData, PhantomInvariantLifetime};

#[derive(Debug)]
pub struct Heap<'a, T> {
    data: Vec<T>,
    data_index: Vec<usize>,
    id: Vec<usize>,
    _invariant: PhantomInvariantLifetime<'a>,
}

impl<'a, T> Default for Heap<'a, T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            data_index: Default::default(),
            id: Default::default(),
            _invariant: PhantomInvariantLifetime::new(),
        }
    }
}

#[must_use = "handle is the only way to access this element in the array. In the long term, \
              dropping this will also drop the corresponding element in the heap"]
#[derive(Debug)]
pub struct HeapHandle<'a, T> {
    index: usize,
    _invariant: PhantomInvariantLifetime<'a>,
    _type: PhantomData<T>,
}

impl<'a, T> Heap<'a, T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<F: for<'b> FnOnce(Heap<'b, T>)>(func: F) {
        let value = Self::default();
        func(value)
    }

    pub fn push(&mut self, item: T) -> HeapHandle<'a, T> {
        let new_position = self.data.len();
        self.data.push(item);
        let index = match self.id.get(new_position) {
            Some(&index) => index,
            None => {
                self.id.push(new_position);
                self.data_index.push(new_position);
                new_position
            }
        };
        HeapHandle {
            index,
            _invariant: PhantomInvariantLifetime::new(),
            _type: PhantomData,
        }
    }

    // TODO: check lifetimes
    pub fn get<'b>(&'b self, handle: &HeapHandle<'a, T>) -> &'b T {
        let index = handle.index;

        let actual_index = *self
            .data_index
            .get(index)
            .expect("handle exists, so heap holds item");

        (self
            .data
            .get(actual_index)
            .expect("actual_index is in data_index, so this area must exist")) as _
    }

    // TODO: check lifetimes
    pub fn get_mut<'b>(&'b mut self, handle: &HeapHandle<'a, T>) -> &'b mut T {
        let index = handle.index;
        let actual_index = *self
            .data_index
            .get(index)
            .expect("handle exists, so heap holds item");

        (self
            .data
            .get_mut(actual_index)
            .expect("actual_index is in data_index, so this area must exist")) as _
    }

    pub fn pop(&mut self, handle: HeapHandle<'a, T>) -> T {
        let HeapHandle { index, .. } = handle;

        let actual_index = *self.data_index.get(index).unwrap();

        let out = self.data.swap_remove(actual_index);
        let new_len = self.data.len();
        self.id.swap(actual_index, new_len);

        *self.data_index.get_mut(index).unwrap() = self.data.len();

        let actual_index_id = *self.id.get(actual_index).unwrap();
        *self.data_index.get_mut(actual_index_id).unwrap() = actual_index;

        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_remove() {
        let mut test = Heap {
            data_index: vec![4, 5, 6, 0, 2, 3, 1],
            id: vec![3, 6, 4, 5, 0, 1, 2],
            data: vec!['D', 'G', 'E', 'F', 'A', 'B', 'C'],
            _invariant: PhantomInvariantLifetime::new(),
        };

        let handle = HeapHandle {
            index: 0,
            _invariant: PhantomInvariantLifetime::new(),
            _type: PhantomData,
        };

        let char = test.pop(handle);
        assert_eq!(char, 'A');
        assert_eq!(test.data_index, [6, 5, 4, 0, 2, 3, 1]);
        assert_eq!(test.id, [3, 6, 4, 5, 2, 1, 0]);
        assert_eq!(test.data, ['D', 'G', 'E', 'F', 'C', 'B']);

        let handle2 = HeapHandle {
            index: 6,
            _invariant: PhantomInvariantLifetime::new(),
            _type: PhantomData,
        };

        let char2 = test.pop(handle2);

        assert_eq!(char2, 'G');
        assert_eq!(test.id, [3, 1, 4, 5, 2, 6, 0]);
        assert_eq!(test.data, ['D', 'B', 'E', 'F', 'C']);
        assert_eq!(test.data_index, [6, 1, 4, 0, 2, 3, 5]);
    }

    #[test]
    fn push() {
        let mut test = Heap {
            id: vec![3, 1, 4, 5, 2, 6, 0],
            data: vec!['D', 'B', 'E', 'F', 'C'],
            data_index: vec![6, 1, 4, 0, 2, 3, 5],
            _invariant: PhantomInvariantLifetime::new(),
        };

        let handle_h = test.push('H');
        assert_eq!(handle_h.index, 6);

        assert_eq!(test.id, [3, 1, 4, 5, 2, 6, 0]);
        assert_eq!(test.data, ['D', 'B', 'E', 'F', 'C', 'H']);
        assert_eq!(test.data_index, [6, 1, 4, 0, 2, 3, 5]);

        let handle_i = test.push('I');
        assert_eq!(handle_i.index, 0);

        assert_eq!(test.id, [3, 1, 4, 5, 2, 6, 0]);
        assert_eq!(test.data, ['D', 'B', 'E', 'F', 'C', 'H', 'I']);
        assert_eq!(test.data_index, [6, 1, 4, 0, 2, 3, 5]);

        let handle_j = test.push('J');
        assert_eq!(handle_j.index, 7);

        assert_eq!(test.id, [3, 1, 4, 5, 2, 6, 0, 7]);
        assert_eq!(test.data, ['D', 'B', 'E', 'F', 'C', 'H', 'I', 'J']);
        assert_eq!(test.data_index, [6, 1, 4, 0, 2, 3, 5, 7]);
    }

    #[test]
    fn compile_error() {
        Heap::new(|mut test1| {
            Heap::new(|mut test2| {
                let handle0 = test1.push('A');
                let _handle = test2.push('T');
                assert_eq!(test1.pop(handle0), 'A');
                // test1.pop(_handle); // compile fail
            });
        });
    }
}
