#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    #[default]
    DoNotCull,
    FrontFace,
    BackFace,
}
