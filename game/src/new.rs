use std::num::NonZero;
use std::rc::Rc;

use engine::array_vec::{ArrayVec, arrvec};
use engine::entity::{AggregateEntity, Mass};
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::camera::ManualPose;
use engine::modelling::cubic::geometry::{Orientation, Pose, SkeletalAnimation};
use engine::modelling::cubic::lighting::shadow::{
    ShadowListLights,
    ShadowPointLight,
    ShadowSpotLight,
};
use engine::modelling::cubic::lighting::simple::{ListLights, PointLight, SpotLight};
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{SkeletonCubic, camera};
use engine::modelling::{Bloom, Line, Quad, SimpleCubic, Skeleton, SkyBox, test_models};
use engine::shader_program::CullFace;

use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder};
use engine::types::TexDim;
use engine::{ColourRGB, ColourRGBA, EngResult, Entity, TransformBy};

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

        let cube_positions = [
            [0.0, 0.0, 0.0],
            //[2.0, 0.0, 0.0],
            //[-2.0, 0.0, 0.0],
            //[0.0, 2.0, 0.0],
            //[0.0, -2.0, 0.0],
            //[0.0, 0.0, 2.0],
            //[0.0, 0.0, -2.0],
        ];

        let containers = cube_positions
            .into_iter()
            .map(|arr| {
                let material = Rc::new(
                    Material::builder()
                        .diffuse(FlatTexture::monochrome(ColourRGBA::new_from_arr_alpha(
                            arr, 1.0,
                        )))
                        .build(),
                );

                SimpleCubic::cube(0.1, material)
                    .cull_face(CullFace::BackFace)
                    .build()
            })
            .collect();

        let (animation, model) = make_model("morgan")?;
        let imported = Entity::from_cubic(model, 20, Mass(1.0), |_| ()).unwrap();

        let mut imported_other = Entity::from_cubic(
            SimpleCubic::cube(1.0, Rc::new(Material::black())).build(),
            20,
            Mass(3.0),
            |_| (),
        )
        .unwrap();
        imported_other.velocity = Vector::new([0.0, 0.0, -0.05]);
        imported_other.pose = Pose::new_from_translation(Vector::new([0.0, 0.0, 2.1]));

        let bounding_box_level = 0;

        const START_LOC: Vector<3> = Vector::new([-2.0, 1.7, 0.0]);

        let camera = camera::builder()
            .pose(ManualPose::new_fixed_forward_up(
                START_LOC,
                UnitVector::X,
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

        let (simpletail, skeletal_animation) = SkeletonCubic::debug_tail();

        // let simpletail =
        // SkinningEntity::from_skeletoncubic(simpletail, 2, Mass(1.0), |_|
        // ()).unwrap();

        let mut aggregate_tail = AggregateEntity::new(simpletail.clone(), 1, |_| ());
        let offset = Pose::new_from_orientation_translation(
            Orientation::new_forward_up(UnitVector::Y, -UnitVector::Z),
            Vector::new([0.0, 0.5, 0.5]),
        );
        aggregate_tail.push(simpletail.clone(), offset, 9, 0, 1, |_| ());

        Ok(Self {
            show_bbs: true,
            time: 0.0,
            game_start: None,
            line,
            angle: 0.0,
            camera,
            exposure,
            sensitivity,
            speed,

            light,
            containers,
            skybox,

            testbool: false,
            imported,
            imported_other,
            bounding_box_level,
            draw_models: true,

            aggregate_tail,
            simpletail,
            skeletal_animation,
            which_animation: animation,

            hdr_fb,
            do_bloom: false,
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

pub fn make_model(which_model: &str) -> EngResult<(Option<SkeletalAnimation>, SimpleCubic), StateError> {
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
        "oliver" => {
            let material = Rc::new(
                Material::builder()
                    .diffuse(FlatTexture::monochrome(ColourRGBA::new([
                        1.0, 0.0, 0.0, 1.0,
                    ])))
                    .build(),
            );

            let mut builder = SimpleCubic::builder();
            const LENGTH_TAIL: usize = 10;
            let cube_push = Pose::new_from_translation(Vector::new([0.0, 0.0, 1.0]));
            let mut cube_pose = Pose::IDENTITY;

            builder.push_cube(cube_pose, material, 1.0, Skeleton::ROOT_BONE);

            for current_bone in 1..LENGTH_TAIL {
                cube_pose = cube_pose.transform_by(&cube_push);

                let material = Rc::new(
                    Material::builder()
                        .diffuse(FlatTexture::grayscale(
                            current_bone as f32 / LENGTH_TAIL as f32,
                            1.0,
                        ))
                        .build(),
                );

                builder.push_cube(cube_pose, material, 1.0, current_bone);
            }
            let (skeleton, animation_1) = test_models::tail_animation(LENGTH_TAIL);
            (
                Some(animation_1),
                builder
                    .cull_face(CullFace::BackFace)
                    .skeleton(skeleton)
                    .build(),
            )
        }
        _ => (None, SimpleCubic::empty()),
    };
    Ok(model)
}
