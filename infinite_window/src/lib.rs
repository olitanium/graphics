#![feature(option_array_transpose)]

use std::array;

/// returns an array of references (or ref mut)
#[derive(Debug)]
pub struct InfiniteWindowIter<'a, T, const N: usize> {
    backing: &'a [T],
    current_index: usize,
}

impl<'a, T, const N: usize> InfiniteWindowIter<'a, T, N> {
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            backing: slice,
            current_index: 0,
        }
    }
}

impl<'a, T, const N: usize> Iterator for InfiniteWindowIter<'a, T, N> {
    type Item = [&'a T; N];

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let len = if self.backing.len() == 0 {
            return None;
        } else {
            self.backing.len()
        };

        let output = array::from_fn(|index| &self.backing[(index + self.current_index) % len]);

        self.current_index = (self.current_index + 1) % len;

        Some(output)
    }
}

#[cfg(test)]
mod test {
    use super::InfiniteWindowIter;

    #[test]
    fn longer_backing() {
        let slice = &[1, 2, 3];
        let mut iterator = InfiniteWindowIter::new(slice);

        assert_eq!(iterator.next(), Some([&1, &2]));
        assert_eq!(iterator.next(), Some([&2, &3]));
        assert_eq!(iterator.next(), Some([&3, &1]));
    }

    #[test]
    fn shorter_backing() {
        let slice = &[1, 2];
        let mut iterator = InfiniteWindowIter::new(slice);

        assert_eq!(iterator.next(), Some([&1, &2, &1]));
        assert_eq!(iterator.next(), Some([&2, &1, &2]));
    }

    #[test]
    fn empty_backing() {
        let slice: &[i32; 0] = &[];
        let mut iterator = InfiniteWindowIter::<_, 1>::new(slice);

        assert!(iterator.next().is_none())
    }
}
