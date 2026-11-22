#![feature(const_trait_impl)]
#![feature(const_destruct)]
#![feature(const_array)]

mod iter;
use std::marker::Destruct;

pub use iter::RGBAIntoIter;

#[derive(Debug, Clone, Copy)]
pub struct ColourRGBA([f32; 4]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        pub const fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGBA {
    pub const BLACK: Self = Self::grayscale(0.0, 1.0);
    pub const BLUE: Self = Self::new([0.0, 0.0, 1.0, 1.0]);
    pub const GREEN: Self = Self::new([0.0, 1.0, 0.0, 1.0]);
    pub const RED: Self = Self::new([1.0, 0.0, 0.0, 1.0]);
    pub const WHITE: Self = Self::grayscale(1.0, 1.0);

    get_colour!(r, 0);

    get_colour!(g, 1);

    get_colour!(b, 2);

    get_colour!(a, 3);

    pub const fn new(input: [f32; 4]) -> Self {
        Self(input)
    }

    pub const fn grayscale(input: f32, alpha: f32) -> Self {
        Self::new([input, input, input, alpha])
    }

    pub const fn new_from_arr_alpha(input: [f32; 3], alpha: f32) -> Self {
        Self([input[0], input[1], input[2], alpha])
    }

    pub const fn as_array(self) -> [f32; 4] {
        self.0
    }

    pub const fn as_ref(&self) -> &[f32; 4] {
        &self.0
    }

    #[must_use = "map returns a new colour, so needs to be either used or saved"]
    pub const fn map_colour<F>(self, op: F) -> Self
    where
        F: [const] FnMut(f32) -> f32 + [const] Destruct,
    {
        let [r, g, b, a] = self.as_array();
        let [nr, ng, nb] = [r, g, b].map(op);
        Self::new([nr, ng, nb, a])
    }

    pub const fn to_rgb(self) -> ColourRGB {
        let [r, g, b, _] = self.as_array();
        ColourRGB::new([r, g, b])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColourRGB([f32; 3]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        pub const fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGB {
    pub const BLACK: Self = Self::new([0.0; 3]);
    pub const BLUE: Self = Self::new([0.0, 0.0, 1.0]);
    pub const GREEN: Self = Self::new([0.0, 1.0, 0.0]);
    pub const RED: Self = Self::new([1.0, 0.0, 0.0]);
    pub const WHITE: Self = Self::new([1.0; 3]);

    get_colour!(r, 0);

    get_colour!(g, 1);

    get_colour!(b, 2);

    pub const fn new(input: [f32; 3]) -> Self {
        Self(input)
    }

    pub const fn as_array(self) -> [f32; 3] {
        self.0
    }

    pub const fn as_ref(&self) -> &[f32; 3] {
        &self.0
    }

    pub const fn to_rgba_with(self, alpha: f32) -> ColourRGBA {
        let Self([r, g, b]) = self;
        ColourRGBA::new([r, g, b, alpha])
    }

    #[must_use = "map returns a new colour, so needs to be either used or saved"]
    pub const fn map<F>(&self, f: F) -> Self
    where
        F: [const] Fn(f32) -> f32 + [const] Destruct,
    {
        Self(self.as_array().map(f))
    }
}
