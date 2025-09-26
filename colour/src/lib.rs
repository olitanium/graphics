mod iter;
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
    pub const WHITE: Self = Self::grayscale(1.0, 1.0);

    pub const RED: Self = Self::new([1.0, 0.0, 0.0, 1.0]);
    pub const GREEN: Self = Self::new([0.0, 1.0, 0.0, 1.0]);
    pub const BLUE: Self = Self::new([0.0, 0.0, 1.0, 1.0]);

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

    pub fn as_ref(&self) -> &[f32; 4] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColourRGB([f32; 3]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        pub fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGB {
    pub const BLACK: Self = ColourRGB::new([0.0, 0.0, 0.0]);
    pub const WHITE: Self = ColourRGB::new([1.0, 1.0, 1.0]);

    get_colour!(r, 0);

    get_colour!(g, 1);

    get_colour!(b, 2);

    pub const fn new(input: [f32; 3]) -> Self {
        Self(input)
    }

    pub const fn as_array(self) -> [f32; 3] {
        self.0
    }

    pub fn as_ref(&self) -> &[f32; 3] {
        &self.0
    }

    pub const fn to_rgba_with(&self, alpha: f32) -> ColourRGBA {
        ColourRGBA::new([self.0[0], self.0[1], self.0[2], alpha])
    }

    pub fn map<F: Fn(f32) -> f32>(&self, f: F) -> ColourRGB {
        ColourRGB(self.as_array().map(f))
    }
}
