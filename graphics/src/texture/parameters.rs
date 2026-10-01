use colour::ColourRGBA;

use crate::types::GLint;

#[derive(Debug, Default)]
pub enum WrapType {
    #[default]
    ClampToEdge,
    ClampToBorder(ColourRGBA),
}

impl WrapType {
    pub(crate) const fn get_enum(&self) -> GLint {
        match self {
            Self::ClampToEdge => gl::CLAMP_TO_EDGE as GLint,
            Self::ClampToBorder(_) => gl::CLAMP_TO_BORDER as GLint,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum Minification {
    #[default]
    Nearest,
    Bilinear,
    Trilinear,
    MipMap {
        sample_type: MipMapInfo,
        mipmap_choice: MipMapInfo,
    },
}

#[derive(Debug, Clone)]
pub enum MipMapInfo {
    Nearest,
    Linear,
}

impl Minification {
    pub(crate) const fn get_enum(&self) -> GLint {
        match self {
            Self::Nearest => gl::NEAREST as GLint,
            Self::Bilinear => gl::LINEAR as GLint,
            Self::Trilinear => gl::LINEAR_MIPMAP_LINEAR as GLint,
            Self::MipMap {
                sample_type,
                mipmap_choice,
            } => get_mipmap_enum(sample_type, mipmap_choice),
        }
    }
}

const fn get_mipmap_enum(sample_type: &MipMapInfo, mipmap_choice: &MipMapInfo) -> GLint {
    use MipMapInfo as M;
    let out = match (sample_type, mipmap_choice) {
        (M::Linear, M::Linear) => gl::LINEAR_MIPMAP_LINEAR,
        (M::Linear, M::Nearest) => gl::LINEAR_MIPMAP_NEAREST,
        (M::Nearest, M::Linear) => gl::NEAREST_MIPMAP_LINEAR,
        (M::Nearest, M::Nearest) => gl::NEAREST_MIPMAP_NEAREST,
    };
    out as GLint
    // GL_{0}_MIPMAP_{1}
    // 0: Determines how each mipmap image is sampled from: be that linear or
    // nearest 1: Determines which mipmap to choose, be that
}

#[derive(Debug, Default, Clone)]
pub enum Magnification {
    #[default]
    Linear,
    Nearest,
}

impl Magnification {
    pub(crate) const fn get_enum(&self) -> GLint {
        match self {
            Self::Nearest => gl::NEAREST as GLint,
            Self::Linear => gl::LINEAR as GLint,
        }
    }
}
