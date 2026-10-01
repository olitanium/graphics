use core::iter::Fuse;
use core::mem;

pub struct IterWithCloned<I: Iterator, T: Clone> {
    into_iter: Fuse<I>,
    next_item: Option<(I::Item, T)>,
}

impl<I, T> Iterator for IterWithCloned<I, T>
where
    I: Iterator,
    T: Clone,
{
    type Item = (I::Item, T);

    fn next(&mut self) -> Option<Self::Item> {
        // if there is an item in next_next_item, then we need to take a copy of
        // the T in next_item, for replacing later.

        let next_next_item = self.into_iter.next();
        match next_next_item {
            Some(next_next_value) => {
                // we need to take a clone of the T in next_item and bundle it
                // with this.
                let clone = {
                    let (_, original) = self.next_item.as_ref()?;
                    original.clone()
                };

                let next_next_option = Some((next_next_value, clone));

                mem::replace(&mut self.next_item, next_next_option)
            }
            None => self.next_item.take(),
        }
    }
}

impl<I, T> IterWithCloned<I, T>
where
    I: Iterator,
    T: Clone,
{
    pub fn new<Z>(iter: Z, cloneable: T) -> Self
    where
        Z: IntoIterator<IntoIter = I>,
    {
        let mut into_iter = iter.into_iter().fuse();
        let next_item = into_iter.next().map(|elem| (elem, cloneable));

        Self {
            into_iter,
            next_item,
        }
    }
}

pub trait HasIterWithCloned: Iterator + Sized {
    fn with_cloned<T>(self, cloneable: T) -> IterWithCloned<Self, T>
    where
        T: Clone,
    {
        IterWithCloned::new(self, cloneable)
    }
}

impl<T> HasIterWithCloned for T where T: Iterator {}

#[cfg(test)]
mod tests {
    use crate::HasIterWithCloned as _;

    #[test]
    fn counting() {
        struct CloneCounter(usize);

        impl Clone for CloneCounter {
            fn clone(&self) -> Self {
                let number = self.0;
                let incr = number + 1;

                Self(incr)
            }
        }

        let initial_counter = CloneCounter(0);

        for (value, counter) in (0..100).with_cloned(initial_counter) {
            assert_eq!(value, counter.0);
        }
    }
}
