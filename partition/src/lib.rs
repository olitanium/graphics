//! Partition an iterator `iter` into bins.

use get_or_extend::GetAtOrExtend as _;
/// Partition an iterator `iter` into bins.
///
/// Bin chosen determined by the `usize` iterator returned from `part_fn`.
/// Elements `&T` are manipulated by the closure `manipulate`. That might be
/// used to `Clone` the value, or transform the type before collection into the
/// `Vec`.
#[inline]
pub fn partition<U, T, I, Fo, F, O>(iter: I, mut part_fn: F, mut manipulate: O) -> Vec<Vec<U>>
where
    I: IntoIterator<Item = T>,
    Fo: IntoIterator<Item = usize>,
    F: FnMut(&T) -> Fo,
    O: FnMut(&T) -> U,
{
    iter.into_iter().fold(Vec::new(), |mut accum, item| {
        for index in part_fn(&item) {
            let at_index = accum.get_at_or_extend(index, Vec::new);
            at_index.push(manipulate(&item));
        }

        accum
    })
}
