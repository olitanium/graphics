use std::io;
use std::path::Path;

use colour::ColourRGBA;
use image::ImageError;
use utils::error_boilerplate;

use crate::texture::{Magnification, Minification};
use crate::types::{self, TexDim};

#[derive(Debug, Clone)]
pub struct Image {
    size: (TexDim, TexDim),
    data: ImageType,
    pub(crate) mag_filter: Magnification,
    pub(crate) min_filter: Minification,
}

#[derive(Debug, Clone)]
pub enum ImageType {
    Linear(Vec<ColourRGBA>),
    LinearUnclamped(Vec<ColourRGBA>),
    Srgba(Vec<ColourRGBA>),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum NewImageError {
    Io(io::Error),
    Parse(ImageError),
}
error_boilerplate!(NewImageError);

fn new_from_file<P>(path: P) -> Result<((TexDim, TexDim), Vec<ColourRGBA>), NewImageError>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    fn new_from_file_path(
        path: &Path,
    ) -> Result<((TexDim, TexDim), Vec<ColourRGBA>), NewImageError> {
        let image = image::ImageReader::open(path)
            .map_err(NewImageError::Io)?
            .with_guessed_format()
            .map_err(NewImageError::Io)?
            .decode()
            .map_err(NewImageError::Parse)?
            .flipv() // All OpenGL texture images are flipped (maybe)
            .into_rgba32f();

        let pixels = image
            .enumerate_pixels()
            .map(|(_, _, rgba)| ColourRGBA::new(rgba.0))
            .collect();

        let size_as_u32 = image.dimensions();
        let size = (
            TexDim::new(size_as_u32.0 as TexDim::Primitive),
            TexDim::new(size_as_u32.1 as TexDim::Primitive),
        );

        Ok((size, pixels))
    }

    new_from_file_path(path_ref)
}

macro_rules! new_from_file {
    ($name:ident $enum_name:ident) => {
        pub fn $name<P>(path: P) -> Result<Self, NewImageError>
        where
            P: AsRef<Path>,
        {
            let (size, buffer) = new_from_file(path)?;
            Ok(Self {
                size,
                data: ImageType::$enum_name(buffer),
                min_filter: Minification::default(),
                mag_filter: Magnification::default(),
            })
        }
    };
}

pub struct TexInternalFormat<'a> {
    pub(crate) internal_format: types::GLint,
    pub(crate) size: (TexDim, TexDim),
    pub(crate) type_: types::GLenum,
    pub(crate) slice: &'a [ColourRGBA],
}

#[derive(Debug)]
pub enum NewFromStreamError {
    WrongSize,
}
error_boilerplate!(NewFromStreamError);

impl Image {
    new_from_file!(new_from_srgba Srgba);

    new_from_file!(new_from_rgba Linear);

    pub fn new_from_stream(
        size: (TexDim, TexDim),
        pixels: Vec<ColourRGBA>,
    ) -> Result<Self, NewFromStreamError> {
        if pixels.len() == (size.0.as_primitive() * size.1.as_primitive()) as usize {
            Ok(Self {
                size,
                data: ImageType::LinearUnclamped(pixels),
                mag_filter: Magnification::default(),
                min_filter: Minification::default(),
            })
        } else {
            Err(NewFromStreamError::WrongSize)
        }
    }

    pub fn new_monochrome_clamped(colour: ColourRGBA) -> Self {
        Self {
            size: (TexDim::new(1), TexDim::new(1)),
            data: ImageType::Linear(vec![colour]),
            mag_filter: Magnification::Nearest,
            min_filter: Minification::Nearest,
        }
    }

    pub fn new_monochrome(colour: ColourRGBA) -> Self {
        Self {
            size: (TexDim::new(1), TexDim::new(1)),
            data: ImageType::LinearUnclamped(vec![colour]),
            mag_filter: Magnification::Nearest,
            min_filter: Minification::Nearest,
        }
    }

    pub(crate) const fn get_opengl_params(&self) -> TexInternalFormat<'_> {
        let internal_format = match &self.data {
            ImageType::Linear(_) => gl::RGBA,
            ImageType::Srgba(_) => gl::SRGB_ALPHA,
            ImageType::LinearUnclamped(_) => gl::RGBA16F,
        } as types::GLint;

        let size = self.size;

        let type_ = match &self.data {
            ImageType::Linear(_) => gl::FLOAT,
            ImageType::Srgba(_) => gl::FLOAT,
            ImageType::LinearUnclamped(_) => gl::FLOAT,
        };

        let slice = match &self.data {
            ImageType::Linear(x) | ImageType::Srgba(x) | ImageType::LinearUnclamped(x) => {
                x.as_slice()
            }
        };

        TexInternalFormat {
            internal_format,
            size,
            type_,
            slice,
        }
    }
}
