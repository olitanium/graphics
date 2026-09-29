use alloc::ffi::CString;
use core::ffi::CStr;
use core::hash::{Hash, Hasher};
use std::hash::DefaultHasher;

pub trait UniformName {
    fn as_cstr(&self) -> &CStr;

    fn hash(&self) -> DefaultHasher;
}

impl UniformName for CString {
    fn as_cstr(&self) -> &CStr {
        self
    }

    fn hash(&self) -> DefaultHasher {
        <CStr as UniformName>::hash(self)
    }
}

impl UniformName for CStr {
    fn as_cstr(&self) -> &CStr {
        self
    }

    fn hash(&self) -> DefaultHasher {
        let mut hasher = DefaultHasher::new();
        <Self as Hash>::hash(self, &mut hasher);
        hasher
    }
}

impl UniformName for &CStr {
    fn as_cstr(&self) -> &CStr {
        self
    }

    fn hash(&self) -> DefaultHasher {
        let c_str = *self;
        <CStr as UniformName>::hash(c_str)
    }
}
