use core::ptr;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;

use colour::ColourRGB;
use image::DynamicImage;
use utils::{builder, new};

use super::CubeMap;
use crate::error::Result;
use crate::framebuffer::attachments::CubeWithDepth;
use crate::framebuffer::traits::Attachment;
use crate::texture::{Error, Magnification, Minification, TexBuilder, TexBuilderCanBuild};
use crate::types::{TexDim, TexId};
use crate::{gl_call, types};

#[derive(Debug, Default)]
pub struct MissingData;

#[derive(Debug)]
pub struct Dimensions(TexDim);

#[derive(Debug)]
pub struct FramebufferAttachment<T: Attachment> {
    size: TexDim,
    _att_type: PhantomData<T>,
}

#[derive(Debug, Default)]
pub struct HasSomeImage {
    positive_x: DynamicImage,
    negative_x: DynamicImage,
    positive_y: DynamicImage,
    negative_y: DynamicImage,
    positive_z: DynamicImage,
    negative_z: DynamicImage,
}

#[derive(Debug, Clone, Default)]
pub struct Builder<T> {
    data: T,
    min_filter: Minification,
    mag_filter: Magnification,
}

impl TexBuilder for Builder<MissingData> {
    type ExpectedFinal = CubeMap;
    type ReadyToBuild = Builder<Dimensions>;

    fn size(self, size: (TexDim, TexDim)) -> Self::ReadyToBuild {
        Self::ReadyToBuild {
            data: Dimensions(size.0),
            ..self
        }
    }
}

fn load_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
    let image = image::ImageReader::open(&path)
        .map_err(|_| Error::OpeningTexture {
            path: path.as_ref().into(),
        })?
        .decode()
        .map_err(|_| Error::ParsingTextureImage {
            path: path.as_ref().into(),
        })?;

    Ok(image)
}

fn make_colour(colour: ColourRGB) -> DynamicImage {
    let mut image = image::Rgb32FImage::new(1, 1);
    image.get_pixel_mut(0, 0).0 = colour.as_array();
    DynamicImage::ImageRgb32F(image)
}

impl<T> Builder<T> {
    builder!(min_filter: Minification);

    builder!(mag_filter: Magnification);

    // TODO: WithoutExtra
    pub fn depth_attachment(self, size: TexDim) -> Builder<FramebufferAttachment<CubeWithDepth>> {
        Builder {
            data: FramebufferAttachment {
                size,
                _att_type: PhantomData,
            },
            ..self
        }
    }
}

impl Builder<MissingData> {
    new!();

    pub fn image<P: AsRef<Path>>(
        self,
        positive_x: P,
        negative_x: P,
        positive_y: P,
        negative_y: P,
        positive_z: P,
        negative_z: P,
    ) -> Result<Builder<HasSomeImage>> {
        let data = HasSomeImage {
            positive_x: load_image(positive_x)?,
            negative_x: load_image(negative_x)?,
            positive_y: load_image(positive_y)?,
            negative_y: load_image(negative_y)?,
            positive_z: load_image(positive_z)?,
            negative_z: load_image(negative_z)?,
        };

        Ok(Builder { data, ..self })
    }

    pub fn monochrome(self, colour: ColourRGB) -> Builder<HasSomeImage> {
        let data = HasSomeImage {
            positive_x: make_colour(colour),
            negative_x: make_colour(colour),
            positive_y: make_colour(colour),
            negative_y: make_colour(colour),
            positive_z: make_colour(colour),
            negative_z: make_colour(colour),
        };

        Builder { data, ..self }
    }

    pub fn grayscale(self, brightness: f32) -> Builder<HasSomeImage> {
        self.monochrome(ColourRGB::new([brightness; 3]))
    }
}

fn make_tex_set_parameters<T>(build: &Builder<T>) -> TexId {
    let id = {
        let mut id = 0;
        gl_call! { gl::GenTextures(1, &raw mut id); }
        gl_call! { gl::BindTexture(gl::TEXTURE_CUBE_MAP, id); }
        TexId::new(id)
    };

    gl_call! {
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as types::GLint,
        );
    }
    gl_call! {
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as types::GLint,
        );
    }
    gl_call! {
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as types::GLint,
        );
    }
    gl_call! {
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            build.min_filter.get_enum()
        );
    }
    gl_call! {
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            build.mag_filter.get_enum()
        );
    }

    id
}

impl TexBuilderCanBuild for Builder<Dimensions> {
    type Output = CubeMap;

    fn build(self) -> Self::Output {
        let id = make_tex_set_parameters(&self);

        let size = self.data.0;

        for target in [
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        ] {
            gl_call! {
                gl::TexImage2D(
                    target,
                    0,
                    gl::RGBA16F as types::GLint,
                    size.to_primitive(),
                    size.to_primitive(),
                    0,
                    gl::RGBA,
                    gl::FLOAT,
                    ptr::null(),
                );
            }
        }

        Self::Output { id, size }
    }
}

impl Builder<HasSomeImage> {
    pub fn build(self) -> CubeMap {
        let id = make_tex_set_parameters(&self);

        let size = TexDim::new(self.data.positive_x.width() as i32);

        for (image_data, target) in [
            (self.data.positive_x, gl::TEXTURE_CUBE_MAP_POSITIVE_X),
            (self.data.negative_x, gl::TEXTURE_CUBE_MAP_NEGATIVE_X),
            (self.data.positive_y, gl::TEXTURE_CUBE_MAP_POSITIVE_Y),
            (self.data.negative_y, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y),
            (self.data.positive_z, gl::TEXTURE_CUBE_MAP_POSITIVE_Z),
            (self.data.negative_z, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z),
        ] {
            let image = image_data.into_rgb8().into_flat_samples();
            gl_call! {
                gl::TexImage2D(
                    target,
                    0,
                    gl::SRGB as types::GLint,
                    image.layout.width as types::GLsizei,
                    image.layout.height as types::GLsizei,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    image.samples.as_ptr().cast(),
                );
            }
        }

        CubeMap { id, size }
    }
}

impl<T: Attachment> Builder<FramebufferAttachment<T>> {
    pub fn build(self) -> Option<CubeMap> {
        T::components().map(|tex_info| {
            let id = make_tex_set_parameters(&self);

            for target in [
                gl::TEXTURE_CUBE_MAP_POSITIVE_X,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
                gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            ] {
                gl_call! {
                    gl::TexImage2D(
                        target,
                        0,
                        tex_info.internal_format as types::GLint,
                        self.data.size.to_primitive(),
                        self.data.size.to_primitive(),
                        0,
                        tex_info.format,
                        tex_info.data_type,
                        ptr::null(),
                    );
                }
            }

            gl_call! {
                gl::FramebufferTexture(
                    gl::FRAMEBUFFER,
                    tex_info.attachment,
                    id.to_primitive(),
                    0,
                );
            }

            CubeMap {
                id,
                size: self.data.size,
            }
        })
    }
}

impl Drop for CubeMap {
    fn drop(&mut self) {
        let primitive = self.id.to_primitive();
        gl_call! {
            gl::DeleteTextures(1, &raw const primitive);
        }
    }
}
