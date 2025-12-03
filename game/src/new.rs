use std::num::NonZero;
use std::rc::Rc;

use engine::array_vec::{ArrayVec, arrvec};
use engine::entity::{AggregateEntity, AggregateModelIndex, Mass};
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::camera::ManualPose;
use engine::modelling::cubic::geometry::{BoneAnimation, Orientation, Pose, SkeletalAnimation};
use engine::modelling::cubic::lighting::shadow::{
    ShadowListLights,
    ShadowPointLight,
    ShadowSpotLight,
};
use engine::modelling::cubic::lighting::simple::{ListLights, PointLight, SpotLight};
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{SkeletonBoneIndex, SkeletonCubic, camera};
use engine::modelling::{Bloom, Bone, Line, Quad, SimpleCubic, Skeleton, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder};
use engine::types::TexDim;
use engine::{ColourRGB, ColourRGBA, EngResult, SimpleEntity};

use crate::state::{State, StateError};

impl State {
    pub fn new_internal(size: (TexDim, TexDim)) -> EngResult<Self, StateError> {
        let speed = [1.0, 1.0, 1.0];
        let exposure = 1.0;

        let screen_dims = size;

        let sensitivity = 0.001;

        let hdr_fb = Builder::new_flat().depth().size(screen_dims).build();

        let skybox = SkyBox::new(
            CubeMap::builder()
                .image(
                    "assets/skybox/right.jpg",
                    "assets/skybox/left.jpg",
                    "assets/skybox/top.jpg",
                    "assets/skybox/bottom.jpg",
                    "assets/skybox/front.jpg",
                    "assets/skybox/back.jpg",
                )?
                .build(),
        );

        // let cube_positions = [
        // [0.0, 0.0, 0.0],
        // [2.0, 0.0, 0.0],
        // [-2.0, 0.0, 0.0],
        // [0.0, 2.0, 0.0],
        // [0.0, -2.0, 0.0],
        // [0.0, 0.0, 2.0],
        // [0.0, 0.0, -2.0],
        // ];
        //
        // let containers = cube_positions
        // .into_iter()
        // .map(|arr| {
        // let material = Rc::new(
        // Material::builder()
        // .diffuse(FlatTexture::monochrome(ColourRGBA::new_from_arr_alpha(
        // arr, 1.0,
        // )))
        // .build(),
        // );
        //
        // SimpleCubic::cube(0.1, material)
        // .cull_face(CullFace::BackFace)
        // .build()
        // })
        // .collect();

        let (_, morgan) = make_model("morgan")?;
        let morgan = SimpleEntity::from_cubic(morgan, 20, Mass(1.0), |_| ());

        let origin_cube = SimpleEntity::from_cubic(
            SimpleCubic::cube(1.0, Rc::new(Material::black())).build(),
            20,
            Mass(3.0),
            |_| (),
        )
        .unwrap();

        let bounding_box_level = 0;

        const START_LOC: Vector<3> = Vector::new([-2.0, 0.0, -2.0]);

        let camera = camera::builder()
            .pose(ManualPose::new_fixed_up_from_to(
                START_LOC,
                Vector::ZERO,
                UnitVector::Y,
            ))
            .perspective(90.0, hdr_fb.aspect_ratio(), 0.01, 100.0)
            .build();

        let light_colour = ColourRGB::new([1.0, 1.0, 1.0].map(|x| x * 1.0));
        let light_attenuation_array = [1.0, 0.0, 0.0];
        let point_light = ShadowPointLight::new(
            PointLight {
                position: Vector::default(),
                attenuation: light_attenuation_array,
                ambient: light_colour.map(|x| x * 0.1),
                diffuse: light_colour.map(|x| x * 0.5),
                specular: light_colour,
            },
            TexDim::from(NonZero::new(1000).unwrap()),
        );

        let ns_point_light = PointLight {
            position: Vector::default(),
            attenuation: light_attenuation_array,
            ambient: light_colour.map(|x| x * 0.1),
            diffuse: light_colour.map(|x| x * 0.5),
            specular: light_colour,
        };

        // let sun_colour = ColourRGB::new([1.0, 1.0, 1.0]);
        // let far_light = ShadowFarLight::new(
        // FarLight {
        // direction: Vector::new([0.0, -0.99, 0.001]).normalize(),
        // ambient: sun_colour.map(|x| x * 0.1),
        // diffuse: sun_colour.map(|x| x * 0.5),
        // specular: sun_colour.map(|x| x * 1.0),
        // },
        // (1000.into(), 1000.into()),
        // );
        //
        // let sun2_colour = ColourRGB::new([1.0, 1.0, 1.0]);
        // let far_light2 = ShadowFarLight::new(
        // FarLight {
        // direction: Vector::new([1.0, 0.0, 0.0]).normalize(),
        // ambient: sun2_colour.map(|x| x * 0.0),
        // diffuse: sun2_colour.map(|x| x * 0.5),
        // specular: sun2_colour.map(|x| x * 1.0),
        // },
        // (1000.into(), 1000.into()),
        // );
        //
        let spotlight_colour = ColourRGB::new([1.0; 3]);
        let spotlight = ShadowSpotLight::new(
            SpotLight {
                position: camera.centre(&()),
                direction: camera.direction(&()),
                attenuation: [1.0, 0.09, 0.032],
                ambient: spotlight_colour.map(|x| x * 0.0),
                diffuse: spotlight_colour,
                specular: spotlight_colour,
                cos_cut_off: 20f32.to_radians().cos(),
                cos_outer_cut_off: 25f32.to_radians().cos(),
            },
            const { TexDim::from(NonZero::new(1000).unwrap()) },
        );

        let [dark_tex, light_tex] = hdr_fb.get_all_colour();
        let bloom = Bloom::new(dark_tex, light_tex);

        let quad_to_draw = Quad::screen([hdr_fb.get_colour(0).unwrap()]); //.downcast();

        let light_material = {
            let light_emission = FlatTexture::monochrome(light_colour.to_rgba_with(1.0));

            Material::builder().emission(light_emission).build()
        };

        let light = SimpleCubic::cube(1.0, Rc::new(light_material))
            .cull_face(CullFace::BackFace)
            .scale(0.2)
            .build();

        let line = Line::new(vec![]);

        let (simpletail, end_attachment_point, skeletal_animation) = SkeletonCubic::debug_tail();

        // let simpletail =
        // SkinningEntity::from_skeletoncubic(simpletail, 2, Mass(1.0), |_|
        // ()).unwrap();

        let mut aggregate_tail = AggregateEntity::new(simpletail.clone(), 1, |_| ());
        let offset = Pose::new_from_orientation_translation(
            Orientation::new_forward_up(UnitVector::Y, -UnitVector::Z),
            Vector::new([0.0, 0.5, 0.5]),
        );
        aggregate_tail.push_model(
            simpletail.clone(),
            offset,
            end_attachment_point,
            AggregateModelIndex::ROOT_MODEL_INDEX,
            1,
            |_| (),
        );

        let (grappling_hook, grapple_chain_anim, join_pos, hook_pos) = grappling_hook();

        Ok(Self {
            toggle_b: true,
            reset_time: 0.0,
            time: 0.0,
            game_start: None,
            line,
            angle: 0.0,
            camera,
            exposure,
            sensitivity,
            speed,

            skybox,

            morgan,
            origin_cube,
            bounding_box_level,
            toggle_m: true,

            grapple_chain_anim,
            grappling_hook,
            join_pos,
            hook_pos,

            aggregate_tail,
            simpletail,
            skeletal_animation,

            hdr_fb,
            toggle_g: false,
            bloom,

            light_group: ShadowListLights {
                point: arrvec![point_light],
                spot: arrvec![spotlight],
                ..ShadowListLights::default()
            },

            ns_light_group: ListLights {
                point: arrvec![ns_point_light],
                ..ListLights::default()
            },

            quad_to_draw,
        })
    }
}

pub fn make_model(
    which_model: &str,
) -> EngResult<(Option<SkeletalAnimation>, SimpleCubic), StateError> {
    use engine::PostProcess as P;
    let model = match which_model {
        "morgan" => (
            None,
            SimpleCubic::import(
                "assets/Teddy Statue/Teddy Statue.obj",
                vec![P::Triangulate, P::OptimizeGraph, P::OptimizeMeshes],
                true,
            )?
            .cull_face(CullFace::BackFace)
            .scale(0.01)
            .apply_transformation(Pose::new_from_translation(Vector::new([
                0.0, -1.2773916, 0.42744067,
            ])))
            .build(),
        ),
        "backpack" => (
            None,
            SimpleCubic::import(
                "assets/backpack/backpack.obj",
                vec![
                    P::Triangulate,
                    P::FlipUVs,
                    P::OptimizeGraph,
                    P::OptimizeMeshes,
                ],
                false,
            )?
            .material(Rc::new(Material::white()))
            .build(),
        ),
        "bigguy" => (
            None,
            SimpleCubic::import(
                "assets/vamp.fbx",
                vec![
                    P::Triangulate,
                    P::FlipUVs,
                    P::OptimizeGraph,
                    P::OptimizeMeshes,
                ],
                true,
            )?
            .scale(0.01)
            .build(),
        ),
        // "oliver" => {
        // let material = Rc::new(
        // Material::builder()
        // .diffuse(FlatTexture::monochrome(ColourRGBA::new([
        // 1.0, 0.0, 0.0, 1.0,
        // ])))
        // .build(),
        // );
        //
        // let mut builder = SimpleCubic::builder();
        // const LENGTH_TAIL: usize = 10;
        // let cube_push = Pose::new_from_translation(Vector::new([0.0, 0.0, 1.0]));
        // let mut cube_pose = Pose::IDENTITY;
        //
        // builder.push_cube(cube_pose, material, 1.0, SkeletonBoneIndex::SKELETON_ROOT_BONE);
        //
        // for current_bone in 1..LENGTH_TAIL {
        // cube_pose = cube_pose.transform_by(&cube_push);
        //
        // let material = Rc::new(
        // Material::builder()
        // .diffuse(FlatTexture::grayscale(
        // current_bone as f32 / LENGTH_TAIL as f32,
        // 1.0,
        // ))
        // .build(),
        // );
        //
        // builder.push_cube(cube_pose, material, 1.0, current_bone);
        // }
        // let (skeleton, _, animation_1) = test_models::tail_animation(LENGTH_TAIL);
        // (
        // Some(animation_1),
        // builder
        // .cull_face(CullFace::BackFace)
        // .skeleton(skeleton)
        // .build(),
        // )
        // }
        _ => (None, SimpleCubic::empty()),
    };
    Ok(model)
}

fn grappling_hook() -> (
    AggregateEntity<(), SimpleCubic>,
    SkeletalAnimation,
    (AggregateModelIndex, SkeletonBoneIndex),
    AggregateModelIndex,
) {
    // Need origin cube
    // Need chain
    // Need hook

    const ORIGIN_BOX_SIZE: f32 = 1.0;
    let origin_cube = SimpleCubic::cube(
        ORIGIN_BOX_SIZE,
        Rc::new(Material::from_colour(ColourRGBA::RED)),
    )
    .build();

    let mut chain = SimpleCubic::builder();
    let mut skeleton_builder = Skeleton::builder();
    let mut animation_builder = SkeletalAnimation::builder();
    let mut end_bone = SkeletonBoneIndex::SKELETON_ROOT_BONE;
    let mut bones_back_to_front = Vec::new();
    // No chain link is bound to the root bone, so that the root of the origin cube
    // and the
    let white_material = Rc::new(Material::white());

    const EXPECT_MESSAGE: &str = "have not yet finished the animation";
    const SEC_PER_UNIT: f32 = 0.05;
    const CHAIN_LINK_SIZE: f32 = 0.5;
    const LEN: usize = 20;
    for index in 1..=LEN {
        // Create Animation, using end_bone as the current bone
        let mut bone_animation = BoneAnimation::new();
        // do not move for LEN-`index` timesteps
        bone_animation
            .push(Pose::IDENTITY, (LEN - index) as f32 * SEC_PER_UNIT)
            .expect(EXPECT_MESSAGE);
        // then move one distance in one timesteps
        bone_animation
            .push(Pose::IDENTITY, 1.0 * SEC_PER_UNIT)
            .expect(EXPECT_MESSAGE);
        bone_animation
            .push(
                Pose::new_from_translation(Vector::new([0.0, 0.0, CHAIN_LINK_SIZE])),
                (LEN + index) as f32 * SEC_PER_UNIT,
            )
            .expect(EXPECT_MESSAGE);
        bone_animation
            .push_final_non_repeat(Pose::IDENTITY)
            .expect(EXPECT_MESSAGE);

        animation_builder
            .push_animation(bone_animation, end_bone)
            .expect("end bone exists");

        // Create new bone to correspond to the created animation
        let new_bone = Bone::builder(end_bone).bind_pose_relative_to_parent(Pose::IDENTITY);

        end_bone = skeleton_builder
            .push_bone(new_bone)
            .expect("new_bone parent exists");
        bones_back_to_front.push(end_bone);
        // attach a cube to this bone
        chain.push_cube(
            Pose::IDENTITY,
            white_material.clone(),
            CHAIN_LINK_SIZE,
            end_bone,
        );
    }

    let mut bone_animation = BoneAnimation::new();
    bone_animation
        .push_final_non_repeat(Pose::new_from_translation(Vector::new([
            0.0,
            0.0,
            CHAIN_LINK_SIZE * 0.5,
        ])))
        .expect(EXPECT_MESSAGE);

    animation_builder
        .push_animation(bone_animation, end_bone)
        .expect("end bone exists");
    let new_bone = Bone::builder(end_bone).bind_pose_relative_to_parent(
        Pose::new_from_translation(Vector::new([0.0, 0.0, CHAIN_LINK_SIZE * 0.5])),
    );
    end_bone = skeleton_builder
        .push_bone(new_bone)
        .expect("new_bone parent exists");

    let skeleton = skeleton_builder.build();
    let animation = animation_builder.build();
    let chain = chain.skeleton(skeleton).build();

    // `end_bone` now has the bone which I should attach the hook to (plus offset)

    const HOOK_SIZE: f32 = 2.0;
    let hook =
        SimpleCubic::cube(HOOK_SIZE, Rc::new(Material::from_colour(ColourRGBA::GREEN))).build();

    let mut aggregate = AggregateEntity::new(origin_cube, 1, |_| ());
    let chain_attachment_point = aggregate
        .push_model(
            chain,
            Pose::new_from_translation(Vector::new([
                0.0,
                0.0,
                0.5 * (ORIGIN_BOX_SIZE - CHAIN_LINK_SIZE),
            ])),
            SkeletonBoneIndex::SKELETON_ROOT_BONE,
            AggregateModelIndex::ROOT_MODEL_INDEX,
            0,
            |_| (),
        )
        .expect("Root model at ROOT_MODEL_INDEX");
    let hook_attachment_point = aggregate
        .push_model(
            hook,
            Pose::new_from_translation(Vector::new([0.0, 0.0, 0.5 * (HOOK_SIZE)])),
            end_bone,
            chain_attachment_point,
            1,
            |_| (),
        )
        .expect("chain attachment at given index");

    (
        aggregate,
        animation,
        (chain_attachment_point, end_bone),
        hook_attachment_point,
    )
}
