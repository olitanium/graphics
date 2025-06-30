use std::collections::HashMap;
use std::convert::identity;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use russimp::material::{PropertyTypeInfo, TextureType};
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};

use super::model::{Cubic, Mesh};
use crate::buffers::vertex_array::{IncompleteSimpleVertex, VertexArray};
use crate::linear_algebra::{UnitVector, Vector};
use crate::texture::{FlatTexture, Material, TextureHasBuilder};
use crate::types::{ElementArrayElem, ToPrimitive};
use crate::Result;

mod error {
    use std::path::PathBuf;

    use crate::error_boilerplate;

    #[derive(Debug, Clone)]
    pub enum Error {
        FileInsteadOfDir {
            path: PathBuf,
        },
        PathNotIntoStr {
            path: PathBuf,
        },
        FileCannotBeParsed {
            path: PathBuf,
            as_text: String,
        },
        NoRootNode {
            path: PathBuf,
        },
        NoTexCoords {
            dir: PathBuf,
        },
        DuplicateProperty {
            dir: PathBuf,
        },
        ShouldNotOccur,
        ElementArrayOverflow {
            which: &'static str,
            index_asked: usize,
            actual_len: usize,
        },
    }

    error_boilerplate!(Error: Import);
}

pub use error::Error;

use super::Builder;

#[inline(always)]
pub(super) fn import<P: AsRef<Path>>(path: P, post_process: Vec<PostProcess>) -> Result<Builder> {
    let path = path.as_ref();

    let dir = path
        .parent()
        .ok_or_else(|| Error::FileInsteadOfDir { path: path.into() })?;

    let file_path = path
        .to_str()
        .ok_or_else(|| Error::PathNotIntoStr { path: path.into() })?;

    let scene =
        Scene::from_file(file_path, post_process).map_err(|error| Error::FileCannotBeParsed {
            path: path.into(),
            as_text: error.to_string(),
        })?;

    let out = process_node(
        &scene,
        scene
            .root
            .as_ref()
            .ok_or_else(|| Error::NoRootNode { path: path.into() })?,
        dir,
        &mut HashMap::new(),
    )?;
    Ok(Cubic::builder().meshes(out))
}

fn process_node(
    scene: &Scene,
    node: &Node,
    dir: &Path,
    current_mat: &mut HashMap<u32, Result<Rc<Material>>>,
) -> Result<Vec<Mesh>> {
    // process current node and recursively process each in node.children.
    let mut curr_meshes = node
        .meshes
        .iter()
        .map(|mesh_id| {
            let mesh = &scene.meshes[*mesh_id as usize];

            let positions: Vec<Vector<3>> = mesh
                .vertices
                .iter()
                .map(|vec3d| Vector::new([vec3d.x, vec3d.y, vec3d.z]))
                .collect();

            let textures: Vec<Vector<2>> = mesh
                .texture_coords
                .first()
                .ok_or_else(|| Error::ShouldNotOccur)?
                .as_ref()
                .ok_or_else(|| Error::NoTexCoords { dir: dir.into() })?
                .iter()
                .map(|vec3d| Vector::new([vec3d.x, vec3d.y]))
                .collect();

            let normals: Vec<UnitVector<3>> = mesh
                .normals
                .iter()
                .map(|vec3d| Vector::new([vec3d.x, vec3d.y, vec3d.z]).normalize())
                .collect();

            let opt_normal = (!normals.is_empty()).then_some(normals);

            let tangent: Vec<UnitVector<3>> = mesh
                .tangents
                .iter()
                .map(|vec3d| Vector::new([vec3d.x, vec3d.y, vec3d.z]).normalize())
                .collect();

            let opt_tangent = (!tangent.is_empty()).then_some(tangent);

            let element_buffer: Vec<ElementArrayElem> = mesh
                .faces
                .iter()
                .flat_map(|face| face.0.iter().copied().map(Into::into))
                .collect();

            let mut vertex_array_builder = VertexArray::builder();

            fn is_no_vec_or_element_exists<'a, T>(
                name: &'static str,
                opt_value: &'a Option<Vec<T>>,
                index: usize,
            ) -> Result<Option<&'a T>> {
                match opt_value {
                    None => Ok(None),
                    Some(vector) => match vector.get(index) {
                        Some(value) => Ok(Some(value)),
                        None => Err(Error::ElementArrayOverflow {
                            which: name,
                            index_asked: index,
                            actual_len: vector.len(),
                        }
                        .into()),
                    },
                }
            }

            for triangle in element_buffer.array_chunks() {
                let incomplete_triangle = triangle.map(|index| {
                    let index = index.to_primitive() as usize;
                    let triangle_position =
                        *positions.get(index).ok_or(Error::ElementArrayOverflow {
                            which: "position",
                            index_asked: index,
                            actual_len: positions.len(),
                        })?;
                    let triangle_texture =
                        *textures.get(index).ok_or(Error::ElementArrayOverflow {
                            which: "texture",
                            index_asked: index,
                            actual_len: textures.len(),
                        })?;
                    let triangle_normal =
                        is_no_vec_or_element_exists("normal", &opt_normal, index)?.copied();
                    // opt_normal.and_then(|vec| vec.get(index.to_primitive() as usize).copied() );
                    let triangle_tangent =
                        is_no_vec_or_element_exists("tangent", &opt_tangent, index)?.copied();
                    // opt_tangent.and_then(|vec| vec.get(index.to_primitive() as usize).copied() );

                    Result::Ok(IncompleteSimpleVertex {
                        position: triangle_position,
                        texture: triangle_texture,
                        normal: triangle_normal,
                        tangent: triangle_tangent,
                    })
                });

                let transpose = incomplete_triangle.try_map(identity)?;

                vertex_array_builder.push_incomplete_triangle(transpose);
            }

            let vertex_array = vertex_array_builder.build();

            // let mut vertex_array_builder = VertexArray::cubic_builder()
            // .position(vertices)
            // .tex_coord(texture)
            // .element_array(element_buffer);
            //
            // if let Some(tangent) = opt_tangent {
            // vertex_array_builder = vertex_array_builder.tangent(tangent)
            // }
            // if let Some(normal) = opt_normal {
            // vertex_array_builder = vertex_array_builder.normal(normal)
            // }
            //
            // let vertex_array = Rc::new(vertex_array_builder.build()?);

            let mat = current_mat
                .entry(mesh.material_index)
                .or_insert_with_key(|key| parse_material(&scene.materials[*key as usize], dir))
                .clone()?;

            Result::Ok(Mesh::new(Rc::new(vertex_array), mat, 0)) // TODO: bone
                                                                 // set to 0 for
                                                                 // debug
        })
        .collect::<Result<Vec<_>>>()?;

    let children = node.children.borrow();
    let child_meshes = children
        .iter()
        .map(|node| process_node(scene, node, dir, current_mat))
        .try_collect::<Vec<_>>()?
        .into_iter()
        .flatten();

    curr_meshes.extend(child_meshes);
    Ok(curr_meshes)
}

fn parse_material(material: &russimp::material::Material, dir: &Path) -> Result<Rc<Material>> {
    // get semantic -> key -> data
    let mut material_properties = HashMap::new();

    for property in material.properties.iter().cloned() {
        material_properties
            .entry(property.semantic)
            .or_insert_with(HashMap::new)
            .insert(property.key, property.data)
            .map_or_else(
                || Ok(()),
                |_| Err(Error::DuplicateProperty { dir: dir.into() }),
            )?;
    }

    let mut builder = Material::builder();

    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Diffuse)
        .map(|x| x.get("$tex.file".into()))
    {
        builder = builder.diffuse(
            FlatTexture::builder()
                .srgba_image([dir, filepath.as_ref()].into_iter().collect::<PathBuf>())?
                .build(),
        );
    }
    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Specular)
        .map(|x| x.get("$tex.file".into()))
    {
        builder = builder.specular(
            FlatTexture::builder()
                .srgba_image([dir, filepath.as_ref()].into_iter().collect::<PathBuf>())?
                .build(),
        );
    }
    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Height)
        .map(|x| x.get("$tex.file".into()))
    {
        builder = builder.normal_map(
            FlatTexture::builder()
                .rgba_image([dir, filepath.as_ref()].into_iter().collect::<PathBuf>())?
                .build(),
        );
    }
    if let Some(Some(PropertyTypeInfo::FloatArray(arr))) = material_properties
        .get(&TextureType::None)
        .map(|x| x.get("$mat.shininess".into()))
    {
        if let Some(number) = arr.first() {
            builder = builder.shininess(*number);
        }
    }

    Ok(Rc::new(builder.build()))
}
