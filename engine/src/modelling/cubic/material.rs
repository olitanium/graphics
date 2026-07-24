use graphics::Model;
use graphics::colour::{ColourRGB, ColourRGBA};
use graphics::shader_program::{ActiveShaderProgram, ShaderSetUniformError};
use graphics::texture::{FlatTexture, Texture};
use utils::{builder, error_boilerplate, new};

use crate::modelling::CubicVertex;

#[derive(Debug)]
pub struct Material {
    pub _translucent: bool,
    pub shininess: f32,
    pub diffuse: FlatTexture,
    pub specular_map: FlatTexture,
    pub emission: FlatTexture,
    pub normal_map: FlatTexture,
    pub ambient_occlusion: FlatTexture,
}

impl Default for Material {
    fn default() -> Self {
        Self::white()
    }
}

impl Material {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn blank() -> Self {
        Self::builder().build()
    }

    pub fn from_colour(colour: ColourRGBA) -> Self {
        Self::builder()
            .diffuse(FlatTexture::monochrome(colour))
            .build()
    }

    pub fn from_colour_alpha(colour: ColourRGB, alpha: f32) -> Self {
        let rgba_colour = ColourRGBA::new_from_arr_alpha(colour.as_array(), alpha);
        Self::from_colour(rgba_colour)
    }

    pub fn black() -> Self {
        Self::from_colour(ColourRGBA::BLACK)
    }

    pub fn white() -> Self {
        Self::from_colour(ColourRGBA::WHITE)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RegisterToError<'a> {
    SetUniformError(ShaderSetUniformError<'a>),
}
error_boilerplate!(<'a> RegisterToError<'a>);

impl Material {
    pub fn register_to<'a, 'b, const N: usize, T: Texture, M: Model, O>(
        &'a self,
        shader: &mut ActiveShaderProgram<'b, '_, 'a, M, O, T, N>,
        name: &str,
    ) -> Result<(), RegisterToError<'b>>
    where
        M::Vertex: CubicVertex,
    {
        shader
            .set_uniform(format!("{name}.shininess"), &self.shininess)
            .map_err(|err| RegisterToError::SetUniformError(err))?;
        shader.register_texture(
            vec![
                (format!("{name}.diffuse"), &self.diffuse),
                (format!("{name}.specular_map"), &self.specular_map),
                (format!("{name}.emission"), &self.emission),
                (format!("{name}.normal_map"), &self.normal_map),
                (format!("{name}.ambient_occlusion"), &self.ambient_occlusion),
            ]
            .into_iter()
            .map(|(string, tex)| (string, tex as &dyn Texture)),
        );

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Builder {
    translucent: bool,
    shininess: Option<f32>,
    diffuse: Option<FlatTexture>,
    specular: Option<FlatTexture>,
    emission: Option<FlatTexture>,
    normal_map: Option<FlatTexture>,
    ambient_occlusion: Option<FlatTexture>,
}

impl Builder {
    new!();

    builder!(diffuse: Option<FlatTexture>);

    builder!(specular: Option<FlatTexture>);

    builder!(emission: Option<FlatTexture>);

    builder!(normal_map: Option<FlatTexture>);

    builder!(ambient_occlusion: Option<FlatTexture>);

    builder!(shininess: Option<f32>);

    builder!(translucent: bool);
}

impl Builder {
    pub fn build(self) -> Material {
        Material {
            _translucent: self.translucent,
            shininess: self.shininess.unwrap_or(32.0),
            diffuse: self.diffuse.unwrap_or_default(),
            specular_map: self.specular.unwrap_or_default(),
            emission: self.emission.unwrap_or_default(),
            normal_map: self
                .normal_map
                .unwrap_or_else(|| FlatTexture::monochrome(ColourRGBA::new([0.5, 0.5, 1.0, 1.0]))),
            ambient_occlusion: self.ambient_occlusion.unwrap_or_else(FlatTexture::white),
        }
    }
}
