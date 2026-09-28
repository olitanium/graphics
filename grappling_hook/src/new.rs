use alloc::rc::Rc;
use core::num::NonZero;

use engine::entity::{AggregateEntity, AggregateModelIndex, Mass};
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::geometry::{Orientation, Pose};
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::lighting::shadow::{ShadowPointLight, ShadowSpotLight};
use engine::modelling::cubic::lighting::simple::{PointLight, SpotLight};
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{
    AggregateCalcAnim,
    CameraBuilder,
    ManualPoseFixedUp,
    SimpleCubic,
    SkinningCubic,
};
use engine::modelling::{Bloom, Line, SkeletonCubic, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder};
use engine::types::TexDim;
use engine::{ColourRGB, SimpleEntity};

use crate::bear::Bear;
use crate::grappling_hook::GrapplingHook;
use crate::state::{State, StateError};

impl State {
    pub fn new_internal(size: (TexDim, TexDim)) -> Result<Self, StateError> {
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
                )
                .unwrap()
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

        let bear = make_model("bear").unwrap();
        let mut bear = SimpleEntity::from_simple_cubic(bear, 20, Mass::new_inverse(1.0), |_| ());
        bear.physics.pose = Pose::new_from_translation(Vector::new([0.0, 0.0, 4.5]));

        let bear = Bear { inner: bear };

        let origin_cube = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(1.0, Rc::new(Material::black())).build(),
            20,
            const { Mass::new(3.0) },
            |_| (),
        );

        let bounding_box_level = 0;

        const START_LOC: Vector<3> = Vector::new([-3.0, 0.0, 0.0]);

        let camera = CameraBuilder::new()
            .pose(ManualPoseFixedUp::new_from_to(START_LOC, Vector::ZERO, UnitVector::Y).unwrap())
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
            TexDim::from_non_zero(const { NonZero::new(1000).unwrap() }),
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
            TexDim::from_non_zero(const { NonZero::new(1000).unwrap() }),
        );

        let bloom = Bloom::new(size);

        let light_material = {
            let light_emission = FlatTexture::monochrome(light_colour.to_rgba_with(1.0));

            Material::builder().emission(light_emission).build()
        };

        let _light: SkeletonCubic = SimpleCubic::cube(1.0, Rc::new(light_material))
            .cull_face(CullFace::BackFace)
            .scale(0.2)
            .build()
            .into();

        let line = Line::new(vec![]);

        let (simpletail, end_attachment_point, skeletal_animation) = SkinningCubic::debug_tail();

        // let simpletail =
        // SkinningEntity::from_skeletoncubic(simpletail, 2, Mass(1.0), |_|
        // ()).unwrap();

        let mut aggregate_tail = AggregateEntity::new(simpletail.clone(), 1, |_| ());
        let offset = Pose::new_from_orientation_translation(
            Orientation::new_forward_up(UnitVector::Y, -UnitVector::Z).unwrap(),
            Vector::new([0.0, 0.5, 0.5]),
        );
        let _ = aggregate_tail
            .push_model(
                simpletail.clone(),
                offset,
                &end_attachment_point,
                &AggregateModelIndex::ROOT_MODEL_INDEX,
                1,
                |_| (),
            )
            .unwrap();

        let grappling_hook = GrapplingHook::new(3);
        let grapple_animation = todo!();

        Ok(Self {
            toggle_b: false,
            reset_time: f32::NEG_INFINITY,
            time: 0.0,
            game_start: None,
            line,
            angle: 0.0,
            camera,
            exposure,
            sensitivity,
            speed,

            skybox,

            bear: Some(bear),
            origin_cube,
            bounding_box_level,
            toggle_m: false,

            // grapple_chain_anim,
            // grappling_hook,
            // join_pos,
            // hook_pos,
            grappling_hook,
            grapple_animation,

            aggregate_tail,
            simpletail,
            skeletal_animation,

            hdr_fb,
            toggle_g: false,
            toggle_h: false,
            bloom,

            point_light,
            spotlight,
            ns_point_light,
        })
    }
}

pub fn make_model(which_model: &str) -> Result<SimpleCubic, StateError> {
    use engine::PostProcess as P;
    let model = match which_model {
        "bear" => SimpleCubic::import(
            "assets/Teddy Statue/Teddy Statue.obj",
            vec![P::Triangulate, P::OptimizeGraph, P::OptimizeMeshes],
            true,
        )
        .unwrap()
        .cull_face(CullFace::BackFace)
        .scale(0.01)
        .apply_transformation(Pose::new_from_translation(Vector::new([
            0.0, -1.2773916, 0.42744067,
        ])))
        .build(),
        "backpack" => SimpleCubic::import(
            "assets/backpack/backpack.obj",
            vec![
                P::Triangulate,
                P::FlipUVs,
                P::OptimizeGraph,
                P::OptimizeMeshes,
            ],
            false,
        )
        .unwrap()
        .material(Rc::new(Material::white()))
        .build(),
        "bigguy" => SimpleCubic::import(
            "assets/vamp.fbx",
            vec![
                P::Triangulate,
                P::FlipUVs,
                P::OptimizeGraph,
                P::OptimizeMeshes,
            ],
            true,
        )
        .unwrap()
        .scale(0.01)
        .build(),
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
        _ => SimpleCubic::empty(),
    };
    Ok(model)
}
