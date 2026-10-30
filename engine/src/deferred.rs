pub const GBUFFER_ELEMENTS: usize = 4;

pub type GBuffer = Framebuffer<GBUFFER_ELEMENTS, WithStencil>;

mod simple;
use graphics::framebuffer::Framebuffer;
use graphics::framebuffer::attachments::WithStencil;
pub use simple::{SIMPLE_DEFERRED_GROUP_MAX_LIGHTS, SimpleDeferredCompleteGroup};

mod shadow;
pub use shadow::{SHADOW_DEFERRED_GROUP_MAX_LIGHTS, ShadowDeferredCompleteGroup};
