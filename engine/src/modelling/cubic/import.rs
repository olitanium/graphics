use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use geometry::{Pose, Transform, TransformBy};
use graphics::linear_algebra::{Matrix, UnitVector, Vector};
use graphics::texture::{FlatTexture, Image, NewImageError, TextureHasBuilder};
use graphics::types::ElementArrayElem;
use russimp::Matrix4x4;
use russimp::material::{PropertyTypeInfo, TextureType};
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use utils::error_boilerplate;

use super::material::Material;
use crate::modelling::Bone;
use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::cubic::mesh::MeshBuilder;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImportError {
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

error_boilerplate!(ImportError);

pub(crate) struct ImportOutput {
    pub meshes: Vec<MeshBuilder>,
}

pub(super) fn import<P: AsRef<Path>>(
    path: P,
    post_process: Vec<PostProcess>,
    load_tangent_space: bool,
) -> Result<ImportOutput, ImportError> {
    let path = path.as_ref();

    let dir = path
        .parent()
        .ok_or_else(|| ImportError::FileInsteadOfDir { path: path.into() })?;

    let file_path = path
        .to_str()
        .ok_or_else(|| ImportError::PathNotIntoStr { path: path.into() })?;

    let scene = Scene::from_file(file_path, post_process).map_err(|error| {
        ImportError::FileCannotBeParsed {
            path: path.into(),
            as_text: error.to_string(),
        }
    })?;

    let mut bones = HashMap::new();
    let meshes = process_node(
        &scene,
        scene
            .root
            .as_ref()
            .ok_or_else(|| ImportError::NoRootNode { path: path.into() })?,
        dir,
        &mut HashMap::new(),
        &mut bones,
        Pose::IDENTITY,
        load_tangent_space,
    )?;
    // let mut builder = SimpleCubic::builder();
    // builder.extend_meshes(out);
    // Ok(builder)
    Ok(ImportOutput { meshes })
}

fn assimp_matrix_to_pose(matrix: Matrix4x4) -> Pose {
    let matrix = Matrix::from_col_major([
        [matrix.a1, matrix.a2, matrix.a3, matrix.a4],
        [matrix.b1, matrix.b2, matrix.b3, matrix.b4],
        [matrix.c1, matrix.c2, matrix.c3, matrix.c4],
        [matrix.d1, matrix.d2, matrix.d3, matrix.d4],
    ]);

    Pose::new_from_matrix(matrix)
}

fn process_node(
    scene: &Scene,
    node: &Node,
    dir: &Path,
    current_mat: &mut HashMap<u32, Result<Rc<Material>, Rc<ParseMaterialError>>>,
    _current_bones: &mut HashMap<String, Bone>,
    accum_transform: Pose,
    load_tangent_space: bool,
) -> Result<Vec<MeshBuilder>, ImportError> {
    // process current node and recursively process each in node.children.
    let node_transform = assimp_matrix_to_pose(node.transformation);
    let accum_transform = node_transform.transform(&accum_transform);

    let mut curr_meshes =
        node.meshes
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
                    .ok_or(ImportError::ShouldNotOccur)?
                    .as_ref()
                    .ok_or_else(|| ImportError::NoTexCoords { dir: dir.into() })?
                    .iter()
                    .map(|vec3d| Vector::new([vec3d.x, vec3d.y]))
                    .collect();

                let (opt_normal, opt_tangent) = if load_tangent_space {
                    let normals: Vec<Option<UnitVector<3>>> = mesh
                        .normals
                        .iter()
                        .map(|vec3d| Vector::new([vec3d.x, vec3d.y, vec3d.z]).try_normalize())
                        .collect();

                    let opt_normal = (!normals.is_empty()).then_some(normals);

                    let tangent: Vec<Option<UnitVector<3>>> = mesh
                        .tangents
                        .iter()
                        .map(|vec3d| Vector::new([vec3d.x, vec3d.y, vec3d.z]).try_normalize())
                        .collect();

                    let opt_tangent = (!tangent.is_empty()).then_some(tangent);

                    (opt_normal, opt_tangent)
                } else {
                    (None, None)
                };
                let element_buffer: Vec<ElementArrayElem> = mesh
                    .faces
                    .iter()
                    .flat_map(|face| face.0.iter().copied().map(Into::into))
                    .collect();

                fn is_no_vec_or_element_exists<T>(
                    opt_value: &Option<Vec<T>>,
                    index: usize,
                ) -> Result<Option<&T>, usize> {
                    match opt_value {
                        None => Ok(None),
                        Some(vector) => match vector.get(index) {
                            Some(value) => Ok(Some(value)),
                            None => Err(vector.len()),
                        },
                    }
                }

                let mesh_buffer = element_buffer
                    .as_chunks()
                    .0
                    .iter()
                    .map(|triangle| {
                        triangle.try_map(|index| {
                            let index = index.as_usize();
                            let triangle_position = *positions
                                .get(index)
                                .ok_or(positions.len())
                                .map_err(|actual_len| ImportError::ElementArrayOverflow {
                                    which: "position",
                                    index_asked: index,
                                    actual_len,
                                })?;
                            let triangle_texture = *textures
                                .get(index)
                                .ok_or(textures.len())
                                .map_err(|actual_len| ImportError::ElementArrayOverflow {
                                    which: "texture",
                                    index_asked: index,
                                    actual_len,
                                })?;
                            let triangle_normal = is_no_vec_or_element_exists(&opt_normal, index)
                                .map_err(|actual_len| ImportError::ElementArrayOverflow {
                                    which: "normal",
                                    index_asked: index,
                                    actual_len,
                                })?
                                .copied()
                                .flatten();
                            let triangle_tangent = is_no_vec_or_element_exists(&opt_tangent, index)
                                .map_err(|actual_len| ImportError::ElementArrayOverflow {
                                    which: "tangent",
                                    index_asked: index,
                                    actual_len,
                                })?
                                .copied()
                                .flatten();

                            Ok(IncompleteVertex::new(triangle_position)
                                .texture(triangle_texture)
                                .opt_normal(triangle_normal)
                                .opt_tangent(triangle_tangent)
                                .transform_by(&accum_transform))
                        }) //;
                    })
                    .collect::<Result<Vec<_>, ImportError>>();
                let mesh_buffer = mesh_buffer?;

                let mat = current_mat
                    .entry(mesh.material_index)
                    .or_insert_with_key(|key| parse_material(&scene.materials[*key as usize], dir))
                    .clone()
                    .unwrap_or_default();

                let mut builder = MeshBuilder::new().material(mat);
                builder.extend_triangle(mesh_buffer);

                Ok(builder)
            })
            .collect::<Result<Vec<_>, ImportError>>()?;

    let children = node.children.borrow();
    let child_meshes = children
        .iter()
        .map(|node| {
            process_node(
                scene,
                node,
                dir,
                current_mat,
                _current_bones,
                accum_transform,
                load_tangent_space,
            )
        })
        .collect::<Result<Vec<_>, ImportError>>()?
        .into_iter()
        .flatten();

    curr_meshes.extend(child_meshes);
    Ok(curr_meshes)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ParseMaterialError {
    NewTexture {
        path: PathBuf,
        new_image: NewImageError,
    },
    DuplicateProperty {
        key: String,
    },
}
error_boilerplate!(ParseMaterialError);

fn parse_material(
    material: &russimp::material::Material,
    dir: &Path,
) -> Result<Rc<Material>, Rc<ParseMaterialError>> {
    // get semantic -> key -> data
    let mut material_properties = HashMap::new();

    for property in material.properties.iter().cloned() {
        material_properties
            .entry(property.semantic)
            .or_insert_with(HashMap::new)
            .insert(property.key.clone(), property.data)
            .map_or_else(
                || Ok(()),
                |_| {
                    Err(Rc::new(ParseMaterialError::DuplicateProperty {
                        key: property.key,
                    }))
                },
            )?;
    }

    let mut builder = Material::builder();

    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Diffuse)
        .map(|x| x.get("$tex.file"))
    {
        builder = builder.diffuse({
            let path = [dir, filepath.as_ref()].into_iter().collect::<PathBuf>();
            FlatTexture::builder()
                .image(
                    Image::new_from_srgba(&path)
                        .map_err(|new_image| ParseMaterialError::NewTexture { new_image, path })?,
                )
                .build()
        });
    }
    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Specular)
        .map(|x| x.get("$tex.file"))
    {
        builder = builder.specular({
            let path = [dir, filepath.as_ref()].into_iter().collect::<PathBuf>();
            FlatTexture::builder()
                .image(
                    Image::new_from_srgba(&path)
                        .map_err(|new_image| ParseMaterialError::NewTexture { path, new_image })?,
                )
                .build()
        });
    }
    if let Some(Some(PropertyTypeInfo::String(filepath))) = material_properties
        .get(&TextureType::Height)
        .map(|x| x.get("$tex.file"))
    {
        builder = builder.normal_map({
            let path = [dir, filepath.as_ref()].into_iter().collect::<PathBuf>();
            FlatTexture::builder()
                .image(
                    Image::new_from_rgba(&path)
                        .map_err(|new_image| ParseMaterialError::NewTexture { path, new_image })?,
                )
                .build()
        });
    }
    if let Some(Some(PropertyTypeInfo::FloatArray(arr))) = material_properties
        .get(&TextureType::None)
        .map(|x| x.get("$mat.shininess"))
        && let Some(number) = arr.first()
    {
        builder = builder.shininess(*number);
    }

    Ok(Rc::new(builder.build()))
}
