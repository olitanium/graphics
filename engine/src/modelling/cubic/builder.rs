use std::rc::Rc;

use super::model::Mesh;
use super::{Cubic, Skeleton};
use graphics::buffers::VertexArray;
use crate::modelling::test_models::vertex_array_cube;
use crate::modelling::SimpleVertex;
use graphics::shader_program::CullFace;
use super::material::Material;
use utils::{builder, new};

#[derive(Debug, Default, Clone)]
pub struct Builder {
    meshes: Vec<Mesh>,
    cull_face: CullFace,
    skeleton: Option<Skeleton>,
    scale: Option<f32>,
    relative: bool,
}

impl Builder {
    new!();

    builder!(meshes: Vec<Mesh>);

    builder!(cull_face: CullFace);

    builder!(scale: Option<f32>);

    builder!(relative: bool);

    builder!(skeleton: Option<Skeleton>);

    pub fn push_mesh(mut self, mesh: Mesh) -> Self {
        self.meshes.push(mesh);
        self
    }

    pub fn push_cube(self, material: Rc<Material>, side_length: f32, bone: usize) -> Self {
        let cube = Rc::new(vertex_array_cube(side_length));
        self.push_mesh_from(cube, material, bone)
    }

    pub fn push_mesh_from(
        self,
        vertex_array: Rc<VertexArray<SimpleVertex>>,
        material: Rc<Material>,
        bone: usize,
    ) -> Self {
        self.push_mesh(Mesh::new(vertex_array, material, bone))
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for mesh in &mut self.meshes {
            mesh.material = material.clone();
        }
        self
    }
}

impl Builder {
    pub fn build(self) -> Cubic {
        Cubic {
            meshes: self.meshes,
            cull_face: self.cull_face,
            skeleton: self.skeleton.unwrap_or_default(),
            scale: self.scale.unwrap_or(1.0),
            realtive: self.relative,
        }
    }
}
