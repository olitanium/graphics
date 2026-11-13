use core::fmt::Debug;
use core::marker::PhantomData;
use core::{cmp, ptr};
use std::cmp::Ordering;

use colour::ColourRGBA;
use utils::{builder, new};

use super::FlatTexture;
use crate::framebuffer::attachments::{WithDepth, WithStencil};
use crate::framebuffer::traits::{AttachmentWithDepth, FramebufferAttachment};
use crate::texture::image::TexInternalFormat;
use crate::texture::{
    Image,
    Magnification,
    Minification,
    TexBuilder,
    TexBuilderCanBuild,
    WrapType,
};
use crate::types::{self, GLint, TexDim, TexId};
use crate::{gl_call, texture};

#[derive(Default, Debug)]
#[must_use = "`FlatTexture` must be built with the `build` method"]
pub struct Builder<T> {
    image: T,
    wrap_s_t: WrapType,
}

#[derive(Debug, Default)]
pub struct MissingData;

#[derive(Debug)]
pub struct Dimensions<const N: usize> {
    size: (TexDim, TexDim),
    mag_filter: Magnification,
    min_filter: Minification,
}

#[derive(Debug)]
pub struct FlatFramebufferAttachment<X: FramebufferAttachment> {
    size: (TexDim, TexDim),
    depth_cmp: Option<cmp::Ordering>,
    _att_type: PhantomData<X>,
}

impl Builder<MissingData> {
    new!();
}

impl TexBuilder for Builder<MissingData> {
    type ExpectedFinal = FlatTexture;
    type ReadyToBuild<const N: usize> = Builder<Dimensions<N>>;

    fn size<const N: usize>(self, size: (TexDim, TexDim)) -> <Self as TexBuilder>::ReadyToBuild<N> {
        let image = Dimensions {
            size,
            mag_filter: Magnification::default(),
            min_filter: Minification::default(),
        };

        Builder { image, ..self }
    }
}

impl<T> Builder<T> {
    builder!(wrap_s_t: WrapType);

    pub fn image(self, image: Image) -> Builder<Image> {
        Builder { image, ..self }
    }

    // pub fn withoutextra_attachment(
    // self,
    // size: (TexDim, TexDim),
    // ) -> Builder<FlatFramebufferAttachment<WithoutExtra>> {
    // let image = FlatFramebufferAttachment {
    // size,
    // _att_type: PhantomData,
    // };
    //
    // Builder { image, ..self }
    // }

    pub fn depth_attachment(
        self,
        size: (TexDim, TexDim),
    ) -> Builder<FlatFramebufferAttachment<WithDepth>> {
        let image = FlatFramebufferAttachment {
            size,
            depth_cmp: None,
            _att_type: PhantomData,
        };

        Builder { image, ..self }
    }

    pub fn stencil_attachment(
        self,
        size: (TexDim, TexDim),
    ) -> Builder<FlatFramebufferAttachment<WithStencil>> {
        let image = FlatFramebufferAttachment {
            size,
            depth_cmp: None,
            _att_type: PhantomData,
        };

        Builder { image, ..self }
    }

    pub fn monochrome_clamped(self, colour: ColourRGBA) -> Builder<Image> {
        // let mut image = Rgba32FImage::new(1, 1);
        // image.get_pixel_mut(0, 0).0 = colour.as_array();
        // let image = ImageType::RgbaFloat(image.into_flat_samples());

        Builder {
            image: Image::new_monochrome_clamped(colour),
            ..self
        }
    }

    pub fn monochrome(self, colour: ColourRGBA) -> Builder<Image> {
        // let mut image = Rgba32FImage::new(1, 1);
        // image.get_pixel_mut(0, 0).0 = colour.as_array();
        // let image = ImageType::RgbaFloat(image.into_flat_samples());

        Builder {
            image: Image::new_monochrome(colour),
            ..self
        }
    }
}

impl<X: AttachmentWithDepth> Builder<FlatFramebufferAttachment<X>> {
    /// In the glsl shader you compare the given value `r` against
    /// the sampled depth `D`.
    ///
    /// `r depth_cmp C`.
    ///
    /// for example if `depth_cmp` is `Ordering::Greater` then the
    /// ```glsl
    /// uniform sampler2DShadow this_depth;
    /// return texture(this_depth, vec3(texture_coords, r));
    /// ```
    ///
    /// is like
    ///
    /// ```glsl
    /// uniform sampler2D this_depth;
    /// return r > texture(this_depth, texture_coords) ? 1.0 : 0.0;
    /// ```
    pub const fn set_depth_cmp(mut self, depth_cmp: cmp::Ordering) -> Self {
        self.image.depth_cmp = Some(depth_cmp);
        self
    }
}

fn gen_tex_set_parameters<T>(builder: &Builder<T>) -> TexId {
    // let mut output = Internal { id: 0 };
    let id = {
        let mut id = 0;
        gl_call! {gl::GenTextures(1, &raw mut id);}
        TexId::new(id)
    };

    gl_call! { gl::BindTexture(gl::TEXTURE_2D, id.as_primitive()); }

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

    if let WrapType::ClampToBorder(colour) = builder.wrap_s_t {
        gl_call! { gl::TexParameterfv(
            gl::TEXTURE_2D,
            gl::TEXTURE_BORDER_COLOR,
            colour.as_array().as_ptr()
        ); }
    }

    id
}

impl Builder<Image> {
    pub fn build(self) -> FlatTexture {
        let id = gen_tex_set_parameters(&self);

        gl_call! {gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            self.image.min_filter.get_enum(),
        );}

        gl_call! {gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            self.image.mag_filter.get_enum(),
        );}

        let TexInternalFormat {
            internal_format,
            size: (width, height),
            type_,
            slice,
        } = self.image.get_opengl_params();

        gl_call! {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                width.as_primitive(),
                height.as_primitive(),
                0,
                gl::RGBA,
                type_,
                slice.as_ptr().cast(),
            );
        }

        if let Minification::MipMap { .. } = self.image.min_filter {
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

impl<const N: usize> TexBuilderCanBuild<N> for Builder<Dimensions<N>> {
    type Output = FlatTexture;

    fn min_filter(mut self, min_filter: Minification) -> Self {
        self.image.min_filter = min_filter;
        self
    }

    fn mag_filter(mut self, mag_filter: Magnification) -> Self {
        self.image.mag_filter = mag_filter;
        self
    }

    fn build(self) -> <Self as TexBuilderCanBuild<N>>::Output {
        let id = gen_tex_set_parameters(&self);

        gl_call! {gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            self.image.min_filter.get_enum(),
        );}

        gl_call! {gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            self.image.mag_filter.get_enum(),
        );}

        let (internal_format, format) = const { texture::get_format::<N>() };

        gl_call! {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                self.image.size.0.as_primitive(),
                self.image.size.1.as_primitive(),
                0,
                format,
                gl::FLOAT,
                ptr::null(),
            );
        }

        FlatTexture {
            id,
            size: self.image.size,
        }
    }
}

impl<X: AttachmentWithDepth> Builder<FlatFramebufferAttachment<X>> {
    pub fn build(self) -> FlatTexture {
        let tex_info = X::components();
        let id = gen_tex_set_parameters(&self);

        // NO MIN/MAG FILTER, instead texture compare func
        match self.image.depth_cmp {
            None => {
                gl_call! {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, Minification::default().get_enum());
                }
                gl_call! {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, Magnification::default().get_enum());
                }
            }
            Some(ordering) => {
                gl_call! {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE as types::GLint);
                }

                let mode = match ordering {
                    Ordering::Less => gl::LESS,
                    Ordering::Equal => gl::EQUAL,
                    Ordering::Greater => gl::GREATER,
                };

                gl_call! {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC, mode as types::GLint);
                }
            }
        }

        gl_call! { gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            tex_info.internal_format as types::GLint,
            self.image.size.0.as_primitive(),
            self.image.size.1.as_primitive(),
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
                id.as_primitive(),
                0,
            );
        }

        FlatTexture {
            id,
            size: self.image.size,
        }
    }
}
