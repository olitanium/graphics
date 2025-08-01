use super::shadow::ShadowListLights;
use super::simple::ListLights;

pub trait ListLightCompatible<const MAX: usize> {}
pub trait ShadowLightCompatible<const MAX: usize>: ListLightCompatible<MAX> {}

impl<const MAX: usize> ListLightCompatible<MAX> for ListLights<MAX> {}
impl<const MAX: usize> ListLightCompatible<MAX> for ShadowListLights<MAX> {}
impl<const MAX: usize> ShadowLightCompatible<MAX> for ShadowListLights<MAX> {}
