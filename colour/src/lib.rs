#[derive(Debug, Clone, Copy)]
pub struct ColourRGBA([f32; 4]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        #[inline]
        pub const fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGBA {
    get_colour!(r, 0);

    get_colour!(g, 1);

    get_colour!(b, 2);

    get_colour!(a, 3);

    #[inline]
    pub const fn new(input: [f32; 4]) -> Self {
        Self(input)
    }

    #[inline]
    pub const fn new_from_arr_alpha(input: [f32; 3], alpha: f32) -> Self {
        Self([input[0], input[1], input[2], alpha])
    }

    #[inline]
    pub const fn as_array(self) -> [f32; 4] {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColourRGB([f32; 3]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        #[inline]
        pub fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGB {
    get_colour!(r, 0);

    get_colour!(g, 1);

    get_colour!(b, 2);

    #[inline]
    pub const fn new(input: [f32; 3]) -> Self {
        Self(input)
    }

    #[inline]
    pub const fn as_array(&self) -> [f32; 3] {
        self.0
    }

    pub const fn to_rgba_with(&self, alpha: f32) -> ColourRGBA {
        ColourRGBA::new([self.0[0], self.0[1], self.0[2], alpha])
    }

    pub fn map<F: Fn(f32) -> f32>(&self, f: F) -> ColourRGB {
        ColourRGB(self.as_array().map(f))
    }
}
