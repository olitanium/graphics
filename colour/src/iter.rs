use core::array;

use crate::{ColourRGB, ColourRGBA};

impl IntoIterator for ColourRGBA {
    type IntoIter = RGBAIntoIter;
    type Item = f32;

    fn into_iter(self) -> Self::IntoIter {
        RGBAIntoIter {
            inner: self.0.into_iter(),
        }
    }
}

pub struct RGBAIntoIter {
    inner: array::IntoIter<f32, 4>,
}

impl Iterator for RGBAIntoIter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl IntoIterator for ColourRGB {
    type IntoIter = RGBIntoIter;
    type Item = f32;

    fn into_iter(self) -> Self::IntoIter {
        RGBIntoIter {
            inner: self.0.into_iter(),
        }
    }
}

pub struct RGBIntoIter {
    inner: array::IntoIter<f32, 3>,
}

impl Iterator for RGBIntoIter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
