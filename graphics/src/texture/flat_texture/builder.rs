use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use colour::ColourRGBA;
use image::Rgba32FImage;
use utils::{builder, new};

use super::FlatTexture;
use crate::error::Result;
use crate::framebuffer::attachments::{WithDepth, WithStencil, WithoutExtra};
use crate::framebuffer::traits::Attachment;
use crate::gl_call;
use crate::texture::{Magnification, Minification, TexBuilder, TexBuilderCanBuild, WrapType};
use crate::types::{self, GLint, GLsizei, TexDim, TexId, };

#[derive(Default, Debug)]
pub struct Builder<T> {
    image: T,
    wrap_s_t: WrapType,
    mag_filter: Magnification,
    min_filter: Minification,
}

#[derive(Debug, Default)]
pub struct MissingData;

#[derive(Debug)]
pub struct Dimensions((TexDim, TexDim));

#[derive(Debug)]
pub enum ImageType {
    Srgba(image::FlatSamples<Vec<u8>>),
    Rgba(image::FlatSamples<Vec<u8>>),
    RgbaFloat(image::FlatSamples<Vec<f32>>),
}

#[derive(Debug)]
pub struct FramebufferAttachment<X: Attachment> {
    size: (TexDim, TexDim),
    _att_type: PhantomData<X>,
}

macro_rules! add_image {
    ($image_type:ident => $fn_name:ident, $into_func:ident) => {
        pub fn $fn_name<P: AsRef<Path> + Debug>(self, path: P) -> Result<Builder<ImageType>> {
            let image = ImageType::$image_type(
                image::ImageReader::open(&path)
                    .map_err(|_| super::super::error::Error::OpeningTexture {
                        path: path.as_ref().into(),
                    })?
                    .decode()
                    .map_err(|_| super::super::error::Error::ParsingTextureImage {
                        path: path.as_ref().into(),
                    })?
                    .flipv()
                    .$into_func()
                    .into_flat_samples(),
            );

            Ok(Builder { image, ..self })
        }
    };
}

impl Builder<MissingData> {
    new!();
}

impl TexBuilder for Builder<MissingData> {
    type ExpectedFinal = FlatTexture;
    type ReadyToBuild = Builder<Dimensions>;

    fn size(self, dims: (TexDim, TexDim)) -> Self::ReadyToBuild {
        let image = Dimensions(dims);

        Builder { image, ..self }
    }
}

impl<T> Builder<T> {
    builder!(wrap_s_t: WrapType);

    builder!(min_filter: Minification);

    builder!(mag_filter: Magnification);

    add_image!(Srgba => srgba_image, into_rgba8);

    add_image!(Rgba => rgba_image, into_rgba8);

    add_image!(RgbaFloat => rgba_float_image, into_rgba32f);

    pub fn withoutextra_attachment(
        self,
        size: (TexDim, TexDim),
    ) -> Builder<FramebufferAttachment<WithoutExtra>> {
        let image = FramebufferAttachment {
            size,
            _att_type: PhantomData,
        };

        Builder { image, ..self }
    }

    pub fn depth_attachment(
        self,
        size: (TexDim, TexDim),
    ) -> Builder<FramebufferAttachment<WithDepth>> {
        let image = FramebufferAttachment {
            size,
            _att_type: PhantomData,
        };

        Builder { image, ..self }
    }

    pub fn stencil_attachment(
        self,
        size: (TexDim, TexDim),
    ) -> Builder<FramebufferAttachment<WithStencil>> {
        let image = FramebufferAttachment {
            size,
            _att_type: PhantomData,
        };

        Builder { image, ..self }
    }

    pub fn monochrome(self, colour: ColourRGBA) -> Builder<ImageType> {
        let mut image = Rgba32FImage::new(1, 1);
        image.get_pixel_mut(0, 0).0 = colour.as_array();
        let image = ImageType::RgbaFloat(image.into_flat_samples());

        Builder { image, ..self }
    }
}

fn gen_tex_set_parameters<T>(builder: &Builder<T>) -> TexId {
    // let mut output = Internal { id: 0 };
    let id = {
        let mut id = 0;
        gl_call! {gl::GenTextures(1, &raw mut id);}
        TexId::new(id)
    };

    gl_call! { gl::BindTexture(gl::TEXTURE_2D, id.to_primitive()); }

    let wrap_s_t = builder.wrap_s_t.get_enum();

    gl_call! { gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S,
        wrap_s_t,
    );}
    gl_call! {gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T,
        wrap_s_t,
    );}
    gl_call! {gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER,
        builder.min_filter.get_enum(),
    );}
    gl_call! {gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER,
        builder.mag_filter.get_enum(),
    );}

    if let WrapType::ClampToBorder(colour) = builder.wrap_s_t {
        gl_call! { gl::TexParameterfv(
            gl::TEXTURE_2D,
            gl::TEXTURE_BORDER_COLOR,
            colour.as_array().as_ptr()
        ); }
    }

    id
}

impl Builder<ImageType> {
    pub fn build(self) -> FlatTexture {
        let id = gen_tex_set_parameters(&self);

        let (internalformat, width, height, type_, pixels) = match &self.image {
            ImageType::Rgba(samples) => (
                gl::RGBA as GLint,
                TexDim::new(samples.layout.width as GLsizei),
                TexDim::new(samples.layout.height as GLsizei),
                gl::UNSIGNED_BYTE,
                samples.samples.as_ptr().cast(),
            ),
            ImageType::Srgba(samples) => (
                gl::SRGB_ALPHA as GLint,
                TexDim::new(samples.layout.width as GLsizei),
                TexDim::new(samples.layout.height as GLsizei),
                gl::UNSIGNED_BYTE,
                samples.samples.as_ptr().cast(),
            ),
            ImageType::RgbaFloat(samples) => (
                gl::RGBA16F as GLint,
                TexDim::new(samples.layout.width as GLsizei),
                TexDim::new(samples.layout.height as GLsizei),
                gl::FLOAT,
                samples.samples.as_ptr().cast(),
            ),
        };

        gl_call! {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internalformat,
                width.to_primitive(),
                height.to_primitive(),
                0,
                gl::RGBA,
                type_,
                pixels,
            );
        }

        if let Minification::MipMap { .. } = self.min_filter {
            gl_call! {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
        }

        FlatTexture {
            id,
            size: (width, height),
        }
    }
}

impl TexBuilderCanBuild for Builder<Dimensions> {
    type Output = FlatTexture;

    fn build(self) -> Self::Output {
        let id = gen_tex_set_parameters(&self);

        gl_call! {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as GLint,
                self.image.0.0.to_primitive(),
                self.image.0.1.to_primitive(),
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
        }

        FlatTexture {
            id,
            size: self.image.0,
        }
    }
}

// TODO: X: WithDepth and bugfix
impl<X: Attachment> Builder<FramebufferAttachment<X>> {
    pub fn build(self) -> Option<FlatTexture> {
        X::components().map(|tex_info| {
            let id = gen_tex_set_parameters(&self);

            gl_call! { gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                tex_info.internal_format as types::GLint,
                self.image.size.0.to_primitive(),
                self.image.size.1.to_primitive(),
                0,
                tex_info.format,
                tex_info.data_type,
                ptr::null(),
            ); }

            gl_call! {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    tex_info.attachment,
                    gl::TEXTURE_2D,
                    id.to_primitive(),
                    0,
                );
            }

            FlatTexture {
                id,
                size: self.image.size,
            }
        })
    }
}
