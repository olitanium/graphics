pub trait ProceduralVertex {
    fn bind<NEVER>(&self, &mut ActiveShaderProgram) -> EngResult<(), NEVER>;
}
