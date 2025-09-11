use std::rc::Rc;

use graphics::shader_program::CullFace;
use utils::{builder, new};

use super::material::Material;
use super::{Cubic, Skeleton};
use crate::modelling::cubic::Mesh;
use crate::modelling::cubic::mesh::MeshBuilder;
use crate::modelling::test_models;

#[derive(Debug, Clone)]
pub struct Builder {
    //meshes: Vec<Mesh>,
    mesh_builders: Vec<MeshBuilder>,
    cull_face: CullFace,
    skeleton: Option<Skeleton>,
    scale: f32,
    relative: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            mesh_builders: Default::default(),
            cull_face: Default::default(),
            skeleton: Default::default(),
            scale: 1.0,
            relative: false,
        }
    }
}

impl Builder {
    new!();

    builder!(cull_face: CullFace);
    builder!(relative: bool);
    builder!(skeleton: Option<Skeleton>);
    builder!(scale: f32);

    pub fn push_mesh(&mut self, mesh: MeshBuilder) {
        self.mesh_builders.push(mesh);
    }

    pub fn extend_meshes<I: IntoIterator<Item = MeshBuilder>>(&mut self, meshes: I) {
        self.mesh_builders.extend(meshes);
    }

    pub fn push_cube(&mut self, material: Rc<Material>, side_length: f32, bone: usize) {
        let mut mesh_builder = Mesh::builder().material(material).bone(bone);
        mesh_builder.extend_triangle(test_models::origin_cube(side_length));
        self.mesh_builders.push(mesh_builder);
    }

    pub fn material(mut self, material: Rc<Material>) -> Self {
        for mesh in &mut self.mesh_builders {
            mesh.material = material.clone();
        }
        self
    }

    pub fn build(self) -> Cubic {
        let meshes = self
            .mesh_builders
            .into_iter()
            .map(|builder| {
                builder.scale(self.scale).build()
            })
            .collect();
        
        Cubic {
            meshes,
            cull_face: self.cull_face,
            skeleton: self.skeleton.unwrap_or_default(),
            //scale: self.scale,
            relative: self.relative,
        }
    }
}
