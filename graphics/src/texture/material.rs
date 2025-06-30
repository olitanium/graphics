use super::{FlatTexture, Texture};
use colour::ColourRGBA;
//use crate::modelling::Cubic;
use crate::shader_program::ActiveShaderProgram;
use utils::{builder, new};

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

impl Material {
    #[must_use]
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[must_use]
    #[inline]
    pub fn blank() -> Self {
        Self::builder().build()
    }

    pub fn black() -> Self {
        Self::builder()
            .diffuse(FlatTexture::grayscale(0.0, 1.0))
            .build()
    }

    /// # Errors
    #[inline]
    pub fn register_to<'a, 'b, 'c, const N: usize, L, T: Texture>(
        &'c self,
        shader: &mut ActiveShaderProgram<'a, 'b, 'c, (Cubic, L), T, N>,
        name: &str,
    ) {
        shader.set_uniform(format!("{name}.shininess"), self.shininess);
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
        )
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
    #[inline]
    pub fn build(self) -> Material {
        Material {
            _translucent: self.translucent,
            shininess: self.shininess.unwrap_or(32.0),
            diffuse: self.diffuse.unwrap_or_default(),
            specular_map: self.specular.unwrap_or_default(),
            emission: self.emission.unwrap_or_default(),
            // emission_map: self.emission_map.unwrap_or_else(Texture::zeroes),
            normal_map: self
                .normal_map
                .unwrap_or_else(|| FlatTexture::monochrome(ColourRGBA::new([0.5, 0.5, 1.0, 1.0]))),
            ambient_occlusion: self.ambient_occlusion.unwrap_or_else(FlatTexture::white),
        }
    }
}
