pub const GBUFFER_ELEMENTS: usize = 4;

pub type GBuffer = Framebuffer<GBUFFER_ELEMENTS, WithDepth>;
pub type GBufferFramebufferQuad<'a> = FramebufferQuad<'a, GBUFFER_ELEMENTS, WithDepth>;

mod simple;
use graphics::framebuffer::Framebuffer;
use graphics::framebuffer::attachments::WithDepth;
pub use simple::{SIMPLE_DEFERRED_GROUP_MAX_LIGHTS, SimpleDeferredCompleteGroup};

mod shadow;
pub use shadow::{SHADOW_DEFERRED_GROUP_MAX_LIGHTS, ShadowDeferredCompleteGroup};

mod ambient_occlusion;
pub use ambient_occlusion::MakeAmbientOcclusionGroup;

use crate::modelling::FramebufferQuad;
