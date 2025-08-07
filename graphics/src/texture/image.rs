use std::path::Path;

use colour::ColourRGBA;

use crate::Result;
use crate::texture::Error;
use crate::types::{self, TexDim};

#[derive(Debug, Clone)]
pub struct Image {
    size: (TexDim, TexDim),
    data: ImageType,
}

#[derive(Debug, Clone)]
pub enum ImageType {
    Linear(Vec<ColourRGBA>),
    Srgba(Vec<ColourRGBA>),
}

fn new_from_file<P: AsRef<Path>>(path: P) -> Result<((TexDim, TexDim), Vec<ColourRGBA>)> {
    let path_ref = path.as_ref();

    fn new_from_file_path(path: &Path) -> Result<((TexDim, TexDim), Vec<ColourRGBA>)> {
        let image = image::ImageReader::open(&path)
            .map_err(|_| Error::OpeningTexture { path: path.into() })?
            .with_guessed_format()
            .map_err(|_| Error::ParsingTextureImage { path: path.into() })?
            .decode()
            .map_err(|_| Error::ParsingTextureImage { path: path.into() })?
            .flipv() // All OpenGL texture images are flipped (maybe)
            .into_rgba32f();

        let pixels = image
            .enumerate_pixels()
            .map(|(_, _, rgba)| ColourRGBA::new(rgba.0))
            .collect();

        let size_as_u32 = image.dimensions();
        let size = (
            TexDim::new(size_as_u32.0 as i32),
            TexDim::new(size_as_u32.1 as i32),
        );

        Ok((size, pixels))
    }

    new_from_file_path(path_ref)
}

macro_rules! new_from_file {
    ($name:ident $enum_name:ident) => {
        pub fn $name<P: AsRef<Path>>(path: P) -> Result<Self> {
            let (size, buffer) = new_from_file(path)?;
            Ok(Self {
                size,
                data: ImageType::$enum_name(buffer),
            })
        }
    };
}

pub(crate) struct TexInternalFormat<'a> {
    pub(crate) internal_format: types::GLint,
    pub(crate) size: (TexDim, TexDim),
    pub(crate) type_: types::GLenum,
    pub(crate) slice: &'a [ColourRGBA],
}

impl Image {
    new_from_file!(new_from_srgba Srgba);

    new_from_file!(new_from_rgba Linear);

    pub fn new_monochrome(colour: ColourRGBA) -> Self {
        Self {
            size: (TexDim::new(1), TexDim::new(1)),
            data: ImageType::Linear(vec![colour]),
        }
    }

    pub(crate) fn get_opengl_params(&self) -> TexInternalFormat<'_> {
        let internal_format = match &self.data {
            ImageType::Linear(_) => gl::RGBA,
            ImageType::Srgba(_) => gl::SRGB_ALPHA,
        } as types::GLint;

        let size = self.size;

        let type_ = match &self.data {
            ImageType::Linear(_) => gl::FLOAT,
            ImageType::Srgba(_) => gl::FLOAT,
        };

        let slice = match &self.data {
            ImageType::Linear(x) => x.as_slice(),
            ImageType::Srgba(x) => x.as_slice(),
        };

        TexInternalFormat {
            internal_format,
            size,
            type_,
            slice,
        }
    }
}
