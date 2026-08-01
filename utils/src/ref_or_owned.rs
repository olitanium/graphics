use std::ops::Deref;

#[derive(Debug, Clone)]
pub enum RefOwned<'a, T> {
    Owned(T),
    Ref(&'a T),
}

impl<'a, T> RefOwned<'a, T> {
    pub fn get_owned(self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Owned(t) => t,
            Self::Ref(reft) => reft.clone(),
        }
    }

    pub fn get_ref(&self) -> RefOwned<'_, T> {
        RefOwned::Ref(self)
    }
}

impl<'a, T> Deref for RefOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(x) => x,
            Self::Ref(x) => x,
        }
    }
}

impl<'a, T> Default for RefOwned<'a, T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Owned(T::default())
    }
}
