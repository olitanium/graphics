use super::shadow::ShadowListLights;
use super::simple::ListLights;

pub trait ListLightCompatible<const MAX: usize> {}
pub trait ShadowLightCompatible<const MAX: usize>: ListLightCompatible<MAX> {}

impl<'a, const MAX: usize> ListLightCompatible<MAX> for ListLights<'a, MAX> {}
impl<'a, const MAX: usize> ListLightCompatible<MAX> for ShadowListLights<'a, MAX> {}
impl<'a, const MAX: usize> ShadowLightCompatible<MAX> for ShadowListLights<'a, MAX> {}
