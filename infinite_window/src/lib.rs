use std::array;
use std::ops::Rem;

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
        let len = match self.backing.len() {
            0 => return None,
            x => x,
        };

        /// valid when value is 0 <= value < 2*divisor
        fn short_modulo(value: usize, divisor: usize) -> usize {
            if value >= divisor {
                value - divisor
            } else {
                value
            }
        }

        let output =
            array::from_fn(|index| &self.backing[short_modulo(index + self.current_index, len)]);

        self.current_index = short_modulo(self.current_index + 1, len);

        Some(output)
    }

    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let len = self.backing.len();

        self.take(len).find(predicate)
    }
}

#[cfg(test)]
mod test {
    use std::iter;

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

    #[test]
    fn find_at_end() {
        let slice = &[1, 2, 3, 4, 5, 6, 7];
        let mut iterator = InfiniteWindowIter::new(slice);

        assert_eq!(
            iterator.find(|&[val, val2]| val == &7 && val2 == &1),
            Some([&7, &1])
        );
    }
}
