use std::iter;
use std::rc::Rc;
use std::sync::LazyLock;

use array_vec::arrvec;
use geometry::{Orientation, Pose, TransformBy};
use graphics::linear_algebra::{UnitVector, Vector, unit_vector};
use graphics::vertex_array::VertexArray;

use crate::modelling::cubic::material::Material;
use crate::modelling::cubic::{
    BoneAnimation,
    MeshBuilder,
    SkeletalAnimation,
    SkeletonBoneIndex,
    SkinningCubic,
};
use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::quad::QuadVertex;
use crate::modelling::{Bone, ExtendedVertex, Skeleton, SkeletonCubic};

pub fn cube_triangles(
    lower_bound: Vector<3>,
    upper_bound: Vector<3>,
) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
    let [low_x, low_y, low_z] = lower_bound.into_inner();
    let [up_x, up_y, up_z] = upper_bound.into_inner();
    let vertex_positions = [
        // x    y    z
        [low_x, low_y, low_z], // 0
        [low_x, low_y, up_z],  // 1
        [low_x, up_y, low_z],  // 2
        [low_x, up_y, up_z],   // 3
        [up_x, low_y, low_z],  // 4
        [up_x, low_y, up_z],   // 5
        [up_x, up_y, low_z],   // 6
        [up_x, up_y, up_z],    // 7
    ]
    .map(Vector::new);

    let tex_coords =
        iter::repeat([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map(Vector::new));

    [
        [5, 4, 6, 7],
        [0, 1, 3, 2],
        [2, 3, 7, 6],
        [1, 0, 4, 5],
        [1, 5, 7, 3],
        [4, 0, 2, 6],
    ]
    .into_iter()
    .zip(tex_coords)
    .flat_map(move |(position_index, [ta, tb, tc, td])| {
        let [a, b, c, d] =
            position_index.map(|index| *vertex_positions.get(index).expect("well defined"));

        let [va, vb, vc, vd] = [
            IncompleteVertex::new(a).texture(ta),
            IncompleteVertex::new(b).texture(tb),
            IncompleteVertex::new(c).texture(tc),
            IncompleteVertex::new(d).texture(td),
        ];

        [[va.clone(), vb, vc.clone()], [va, vc, vd]]
    })
}

pub fn origin_unit_cube() -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
    origin_cube(1.0)
}

pub fn origin_cube(side_length: f32) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
    let upper_bound = Vector::new([side_length / 2.0; 3]);
    let lower_bound = Vector::new([-side_length / 2.0; 3]);

    cube_triangles(lower_bound, upper_bound)
}

pub fn vertex_array_quad(
    lower_bound: Vector<2>,
    upper_bound: Vector<2>,
) -> VertexArray<QuadVertex> {
    let mut builder = VertexArray::builder();
    builder.extend_triangles(quad_triangles(lower_bound, upper_bound).map(|triangle| {
        ExtendedVertex::from_incomplete_triangle(triangle).expect(
            "filled with rec coords and positions, which are the requirements of the QuadVertex",
        )
    }));
    builder.build()
}

pub fn vertex_array_screen_quad() -> VertexArray<QuadVertex> {
    let vertices = QuadVertex::from_incomplete_triangle([
        IncompleteVertex::new(Vector::new([-1.0, -1.0])).texture(Vector::ZERO),
        IncompleteVertex::new(Vector::new([3.0, -1.0])).texture(Vector::new([2.0, 0.0])),
        IncompleteVertex::new(Vector::new([-1.0, 3.0])).texture(Vector::new([0.0, 2.0])),
    ])
    .expect("I have given enough to make a QuadVertex");

    let mut builder = VertexArray::builder();
    builder.push_triangle(vertices);
    builder.build()
}

pub fn quad_triangles(
    lower_bound: Vector<2>,
    upper_bound: Vector<2>,
) -> impl Iterator<Item = [IncompleteVertex<2>; 3]> {
    let [lower_x, lower_y] = lower_bound.into_inner();
    let [upper_x, upper_y] = upper_bound.into_inner();
    let vertices = [
        ([lower_x, lower_y], [0.0, 0.0]),
        ([upper_x, lower_y], [1.0, 0.0]),
        ([upper_x, upper_y], [1.0, 1.0]),
        ([lower_x, upper_y], [0.0, 1.0]),
    ]
    .map(|(position, texture)| {
        IncompleteVertex::new(Vector::new(position)).texture(Vector::new(texture))
    });

    let triangles =
        [[0, 1, 2], [0, 2, 3]].map(|arr_index| arr_index.map(|index| vertices[index].clone()));

    triangles.into_iter()
}

pub fn screen_quad() -> &'static VertexArray<QuadVertex> {
    static SCREEN_QUAD: LazyLock<VertexArray<QuadVertex>> =
        // LazyLock::new(|| vertex_array_quad(Vector::from([-1.0, -1.0]), Vector::new([1.0,
        // 1.0])));
        LazyLock::new(vertex_array_screen_quad);

    &SCREEN_QUAD
}

/// You need max_len number of bones. One of which is the
pub fn tail_with_bone(max_len: usize) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
    const fn make_long_slice(z: f32) -> [Vector<3>; 4] {
        [
            Vector::new([-0.5, -0.5, z]),
            Vector::new([0.5, -0.5, z]),
            Vector::new([0.5, 0.5, z]),
            Vector::new([-0.5, 0.5, z]),
        ]
    }

    (0..=max_len)
        .map(|z| (z, make_long_slice(z as f32)))
        .map_windows(move |&[(bone1, slice1), (bone2, slice2)]| {
            struct BoneVertexTexture {
                bone: usize,
                position: Vector<3>,
                texture: Vector<2>,
            }

            let vertices_in_order = [
                BoneVertexTexture {
                    bone: bone1,
                    position: slice1[0],
                    texture: Vector::new([bone1 as f32 / max_len as f32, 0.0]),
                },
                BoneVertexTexture {
                    bone: bone1,
                    position: slice1[1],
                    texture: Vector::new([bone1 as f32 / max_len as f32, 1.0]),
                },
                BoneVertexTexture {
                    bone: bone1,
                    position: slice1[2],
                    texture: Vector::new([bone1 as f32 / max_len as f32, 0.0]),
                },
                BoneVertexTexture {
                    bone: bone1,
                    position: slice1[3],
                    texture: Vector::new([bone1 as f32 / max_len as f32, 1.0]),
                },
                BoneVertexTexture {
                    bone: bone2,
                    position: slice2[0],
                    texture: Vector::new([bone2 as f32 / max_len as f32, 0.0]),
                },
                BoneVertexTexture {
                    bone: bone2,
                    position: slice2[1],
                    texture: Vector::new([bone2 as f32 / max_len as f32, 1.0]),
                },
                BoneVertexTexture {
                    bone: bone2,
                    position: slice2[2],
                    texture: Vector::new([bone2 as f32 / max_len as f32, 0.0]),
                },
                BoneVertexTexture {
                    bone: bone2,
                    position: slice2[3],
                    texture: Vector::new([bone2 as f32 / max_len as f32, 1.0]),
                },
            ];
            #[rustfmt::skip]
            let triangles = [
                [5, 1, 4], [4, 1, 0],
                [1, 5, 2], [2, 5, 6],
                [6, 7, 2], [2, 7, 3],
                [0, 3, 4], [4, 3, 7],
            ]
            .map(|triangle| {
                triangle.map(|index| {
                    let BoneVertexTexture{ bone, position, texture} = vertices_in_order[index];
                    let weights = arrvec![(bone, 1.0)];
                    IncompleteVertex::new(position).weights(weights).texture(texture)
                })
            });

            triangles
        })
        .flatten()
}

// num_bones does not include the root bone.
pub fn tail_animation(num_bones: usize) -> (Skeleton, SkeletonBoneIndex, SkeletalAnimation) {
    let animation1 = {
        let mut animation = BoneAnimation::new();
        animation
            .push(
                Pose::new_from_orientation_translation(
                    Orientation::new_forward_up(
                        unit_vector![0.25, 0.0, 1.0],
                        // .try_normalize()
                        // .expect("Vector literal not Zero"),
                        UnitVector::Y,
                    )
                    .expect("literals not the same"),
                    Vector::new([0.0, 0.0, 1.0]),
                ),
                1.0,
            )
            .unwrap();
        animation
            .push(
                Pose::new_from_orientation_translation(
                    Orientation::new_forward_up(unit_vector![-0.25, 0.0, 1.0], UnitVector::Y)
                        .expect("literals not the same"),
                    Vector::new([0.0, 0.0, 1.0]),
                ),
                1.0,
            )
            .unwrap();
        animation
    };

    let mut skeleton = Skeleton::builder();
    let mut animation_1 = SkeletalAnimation::builder();

    let mut current_bone = SkeletonBoneIndex::SKELETON_ROOT_BONE;

    let cube_push = Pose::new_from_translation(Vector::new([0.0, 0.0, 1.0]));
    let mut cube_pose = Pose::IDENTITY;

    for _ in 0..num_bones {
        let bone = Bone::builder(&current_bone).bind_pose_relative_to_parent(cube_push);

        animation_1
            .push_animation(animation1.clone(), &current_bone)
            .expect("parent bone always exists");

        cube_pose = cube_pose.transform_by(&cube_push);
        current_bone = skeleton.push_bone(bone).unwrap();
    }

    (skeleton.build(), current_bone, animation_1.build())
}

pub fn tail_skeleton_cubic(
    max_len: usize,
) -> (SkinningCubic, SkeletonBoneIndex, SkeletalAnimation) {
    // MESHES
    let triangles = tail_with_bone(max_len);

    // make meshes.
    let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::white()));
    mesh_builder.extend_triangle(triangles);
    let mesh = mesh_builder.build();
    let vec_mesh = vec![mesh];

    // SKELETON
    let (skeleton, end_bone, animation) = tail_animation(max_len);

    (
        SkinningCubic::debug_new(vec_mesh, skeleton),
        end_bone,
        animation,
    )
}

pub fn tail_simplecubic(max_len: usize) -> (SkeletonCubic, SkeletonBoneIndex, SkeletalAnimation) {
    let triangles = tail_with_bone(max_len);

    let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::white()));
    mesh_builder.extend_triangle(triangles);

    let mut builder = SkeletonCubic::builder();
    builder.push_mesh(&SkeletonBoneIndex::SKELETON_ROOT_BONE, mesh_builder);
    // SKELETON
    let (skeleton, end_bone_index, animation) = tail_animation(max_len);
    // let animation = SkeletalAnimation::new();
    builder = builder.skeleton(skeleton);

    let cubic = builder.build();
    (cubic, end_bone_index, animation)
}
