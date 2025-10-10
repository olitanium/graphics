#![feature(freeze)]
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Freeze;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

pub struct LocalRc<T: Freeze> {
    data: ManuallyDrop<T>,
    reference_count: Rc<T>,
}

impl<T: Freeze + Debug> Debug for LocalRc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LocalRc").field(&*self.data).finish()
    }
}

impl<T: Freeze + PartialEq> PartialEq for LocalRc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<T: Freeze + Eq> Eq for LocalRc<T> {}

impl<T: Freeze + Hash> Hash for LocalRc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T: Freeze + PartialOrd> PartialOrd for LocalRc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T: Freeze + Default> Default for LocalRc<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Freeze + Ord> Ord for LocalRc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T: Freeze> AsRef<T> for LocalRc<T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T: Freeze> Borrow<T> for LocalRc<T> {
    fn borrow(&self) -> &T {
        &self.data
    }
}

impl<T: Freeze> LocalRc<T> {
    pub fn new(value: T) -> Self {
        // SAFETY:
        // the `ptr::read` is being placed into a ManuallyDrop to prevent dropping. It
        // will never be dropped when in this struct.
        // The `T: Freeze` bound ensures that the value in `data` cannot be modified
        // with a shared reference. this type does not allow the user to obtain
        // a mutable reference to `data`
        let data = unsafe { ManuallyDrop::new(ptr::read(&raw const value)) };
        let reference_count = Rc::new(value);
        Self {
            data,
            reference_count,
        }
    }

    pub fn strong_count(this: &Self) -> usize {
        Rc::strong_count(&this.reference_count)
    }
}

impl<T: Freeze> Deref for LocalRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<T: Freeze> Clone for LocalRc<T> {
    fn clone(&self) -> Self {
        // SAFETY:
        // the `ptr::read` is being placed into a ManuallyDrop to prevent dropping. It
        // will never be dropped when in this struct.
        // The `T: Freeze` bound ensures that the value in `data` cannot be modified
        // with a shared reference which ensures consistency with original.
        // this type does not allow the user to obtain
        // a mutable reference to `data`
        let data = unsafe { ManuallyDrop::new(ptr::read(&raw const *self.data)) };
        Self {
            data,
            reference_count: self.reference_count.clone(),
        }
    }
}
