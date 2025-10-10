use std::iter;
use std::rc::Rc;

use engine::array_vec::ArrayVec;
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::camera;
use engine::modelling::cubic::camera::ManualPose;
use engine::modelling::cubic::geometry::{Animation, Orientation, Pose};
use engine::modelling::cubic::lighting::shadow::{
    ShadowListLights,
    ShadowPointLight,
    ShadowSpotLight,
};
use engine::modelling::cubic::lighting::simple::{ListLights, PointLight, SpotLight};
use engine::modelling::cubic::material::Material;
use engine::modelling::{Bloom, Bone, Cubic, Line, Quad, Skeleton, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder};
use engine::types::TexDim;
use engine::{ColourRGB, ColourRGBA, EngResult, Entity};

use crate::state::{State, StateError};

impl State {
    pub fn new_(size: (TexDim, TexDim)) -> EngResult<Self, StateError> {
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

                Cubic::cube(0.1, material)
                    .cull_face(CullFace::BackFace)
                    .build()
            })
            .collect();

        let mut imported = Entity::from_cubic(make_model("morgan")?, 10, 1.0).unwrap();
        // let imported = Entity::from_cubic(Cubic::cube(1.0,
        // Rc::new(Material::white())).build(), 10, 1.0).unwrap();
        // imported.velocity = Vector::new([0.0, 0.0, 0.05]);
        *imported.model.root_mut() = Pose::new_from_translation(Vector::new([0.0, 0.0, -1.1]));

        let mut imported_other = Entity::from_cubic(make_model("morgan")?, 10, 3.0).unwrap();
        // imported_other.velocity = Vector::new([0.0, 0.0, -0.05]);
        *imported_other.model.root_mut() = Pose::new_from_translation(Vector::new([0.0, 0.0, 1.1]));

        let bounding_box_level = 0;
        /*let imported_bounding_box_model = imported.bounding_box().map_or_default(|bbox| {
            bbox.make_model(bounding_box_level, Rc::new(Material::white()))
                .build()
        });
        let imported_other_bounding_box_model =
            imported_other.bounding_box().map_or_default(|bbox| {
                bbox.make_model(bounding_box_level, Rc::new(Material::white()))
                    .build()
            });
*/
        const START_LOC: [f32; 3] = [0.0, 0.0, -2.0];

        let camera = camera::builder()
            .pose(
                ManualPose::new_fixed_forward_up(
                    Vector::new(START_LOC),
                    UnitVector::Z,
                    UnitVector::Y,
                ), // Demo
            )
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
            1000.into(),
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
            1000.into(),
        );

        let [dark_tex, light_tex] = hdr_fb.get_all_colour();
        let bloom = Bloom::new(dark_tex, light_tex);

        let quad_to_draw = Quad::screen([hdr_fb.get_colour(0).unwrap()]); //.downcast();

        let light_material = {
            let light_emission = FlatTexture::monochrome(light_colour.to_rgba_with(1.0));

            Material::builder().emission(light_emission).build()
        };

        let light = Cubic::cube(1.0, Rc::new(light_material))
            .cull_face(CullFace::BackFace)
            .scale(0.2)
            .build();

        let line = Line::new(vec![]);

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
            blank: Cubic::empty(),
            /*imported_bounding_box_model,
            imported_other_bounding_box_model,*/

            bounding_box_level,

            which_animation: 0,

            hdr_fb,
            do_bloom: false,
            bloom,

            light_group: ShadowListLights {
                point: ArrayVec::try_from([point_light])
                    .expect("size leq SHADOW_SHADER_MAX_LIGHTS"),
                // spot: ArrayVec::try_from([spotlight])
                //    .expect("size leq MAX"),
                ..ShadowListLights::default()
            },

            ns_light_group: ListLights {
                point: ArrayVec::try_from([ns_point_light])
                    .expect("size leq SHADOW_SHADER_MAX_LIGHTS"),
                ..ListLights::default()
            },

            quad_to_draw,
        })
    }
}

pub fn make_model(which_model: &str) -> EngResult<Cubic, StateError> {
    use engine::PostProcess as P;
    let model = match which_model {
        "morgan" => Cubic::import(
            "assets/Teddy Statue/Teddy Statue.obj",
            vec![P::Triangulate, P::OptimizeGraph, P::OptimizeMeshes],
        )?
        .scale(0.01)
        .build(),
        "backpack" => Cubic::import(
            "assets/backpack/backpack.obj",
            vec![
                P::Triangulate,
                P::FlipUVs,
                P::OptimizeGraph,
                P::OptimizeMeshes,
            ],
        )?
        .material(Rc::new(Material::white()))
        .build(),
        "oliver" => {
            let material = Rc::new(
                Material::builder()
                    .diffuse(FlatTexture::monochrome(ColourRGBA::new([
                        1.0, 0.0, 0.0, 1.0,
                    ])))
                    .build(),
            );

            const TAIL_GRADIENT: f32 = 0.9;
            const TAIL_EPSILON: f32 = 0.1;
            let animation1 = {
                let mut animation = Animation::new();
                animation
                    .push(
                        Pose::new_from_orientation_translation(
                            Orientation::new_forward_up(
                                Vector::new([0.25, 0.0, 1.0]).normalize(),
                                UnitVector::Y,
                            ),
                            Vector::new([0.0, 0.0, 0.5 + TAIL_GRADIENT / 2.0 - TAIL_EPSILON]),
                        ),
                        1.0,
                    )
                    .unwrap();
                animation
                    .push(
                        Pose::new_from_orientation_translation(
                            Orientation::new_forward_up(
                                Vector::new([-0.25, 0.0, 1.0]).normalize(),
                                UnitVector::Y,
                            ),
                            Vector::new([0.0, 0.0, 0.5 + TAIL_GRADIENT / 2.0 - TAIL_EPSILON]),
                        ),
                        1.0,
                    )
                    .unwrap();
                animation
            };

            let animation2 = {
                let mut animation = Animation::new();
                animation
                    .push(
                        Pose::new_from_orientation_translation(
                            Orientation::new_forward_up(
                                Vector::new([0.0, 0.25, 1.0]).normalize(),
                                Vector::new([0.0, 1.0, 0.0]).normalize(),
                            ),
                            Vector::new([0.0, 0.0, 0.5 + TAIL_GRADIENT / 2.0 - TAIL_EPSILON]),
                        ),
                        1.0,
                    )
                    .unwrap();
                animation
                    .push(
                        Pose::new_from_orientation_translation(
                            Orientation::new_forward_up(
                                Vector::new([0.0, -0.25, 1.0]).normalize(),
                                Vector::new([0.0, 1.0, 0.0]).normalize(),
                            ),
                            Vector::new([0.0, 0.0, 0.5 + TAIL_GRADIENT / 2.0 - TAIL_EPSILON]),
                        ),
                        1.0,
                    )
                    .unwrap();
                animation
            };

            let mut skeleton = Skeleton::default();
            let animation_rc = Rc::new([animation1, animation2]);
            let mut builder = Cubic::builder();
            let mut current_bone = Skeleton::ROOT_BONE;
            builder.push_cube(material, 1.0, current_bone);

            const LENGTH_TAIL: usize = 10;

            for (index, animation) in iter::repeat_n(animation_rc, LENGTH_TAIL - 1).enumerate() {
                let bone = Bone::builder(current_bone).animations(animation).build();

                current_bone = skeleton.push_bone(bone).unwrap();
                let material = Rc::new(
                    Material::builder()
                        .diffuse(FlatTexture::grayscale(
                            index as f32 / LENGTH_TAIL as f32,
                            1.0,
                        ))
                        .build(),
                );

                builder.push_cube(material, 1.0, current_bone);
            }

            builder
                .cull_face(CullFace::BackFace)
                .skeleton(skeleton)
                .build()
        }
        _ => Cubic::empty(),
    };
    Ok(model)
}
