use std::iter;
use std::rc::Rc;

use engine::array_vec::ArrayVec;
use engine::buffers::framebuffer_builder;
use engine::modelling::cubic::camera::CameraPose;
use engine::modelling::cubic::geometry::{Animation, Orientation, Pose, };
use engine::modelling::cubic::lighting::shadow::{
    ShadowFarLight,
    ShadowListLights,
    ShadowPointLight,
    ShadowSpotLight,
};
use engine::modelling::cubic::lighting::simple::{FarLight, ListLights, PointLight, SpotLight};
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::{Bloom, Bone, Cubic, Quad, Skeleton, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder};
use engine::modelling::cubic::material::Material;
use engine::types::TexDim;
use engine::{ColourRGB, ColourRGBA, Error, Result};
use engine::modelling::cubic::camera;

use crate::state::State;

impl State {
    pub fn new_(size: (TexDim, TexDim)) -> Result<Self> {
        let speed = [1.0, 1.0, 1.0];
        let exposure = 1.0;

        let screen_dims = size;

        let sensitivity = 0.001;

        let hdr_fb = framebuffer_builder().depth().size(screen_dims).build();

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
            //[0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            //[-2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            //[0.0, -2.0, 0.0],
            [0.0, 0.0, 2.0],
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

                Cubic::cube(0.5, material)
                    .cull_face(CullFace::BackFace)
                    .build()
            })
            .collect();

        const WHICH_MODEL: &str = "oliver";
        use engine::PostProcess as P;
        let imported = match WHICH_MODEL {
            "morgan" => {
                Cubic::import(
                    "assets/Teddy Statue/Teddy Statue.obj",
                    vec![
                        P::Triangulate,
                        P::GenerateNormals,
                        P::CalculateTangentSpace,
                        // P::FlipUVs,
                        P::OptimizeGraph,
                        P::OptimizeMeshes,
                    ],
                ).map_err(|error| Error::Other(error.to_string()))?
                    .scale(0.01)
                    .build()
            }
            "backpack" => Cubic::import(
                "assets/backpack/backpack.obj",
                vec![
                    P::Triangulate,
                    P::GenerateNormals,
                    P::CalculateTangentSpace,
                    P::FlipUVs,
                    P::OptimizeGraph,
                    P::OptimizeMeshes,
                ],
                ).map_err(|error| Error::Other(error.to_string()))?
                .build(),
            "oliver" => {
                let material = Rc::new(Material::builder().diffuse(FlatTexture::white()).build());

                const TAIL_GRADIENT: f32 = 0.9;
                const TAIL_EPSILON: f32 = 0.1;
                let animation1 = {
                    let mut animation = Animation::new();
                    animation
                        .push(
                            Pose::new_from_orientation_translation(
                                Orientation::new_forward_up(
                                    Vector::new([0.25, 0.0, 1.0]).normalize(),
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
                                    Vector::new([-0.25, 0.0, 1.0]).normalize(),
                                    Vector::new([0.0, 1.0, 0.0]).normalize(),
                                ),
                                Vector::new([0.0, 0.0, 0.5 + TAIL_GRADIENT / 2.0 - TAIL_EPSILON]),
                                // scale: TAIL_GRADIENT
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
                                // scale: TAIL_GRADIENT
                            ),
                            1.0,
                        )
                        .unwrap();
                    animation
                };

                let mut skeleton = Skeleton::default();
                let animation_rc = Rc::new([animation1, animation2]);
                let mut builder = Cubic::builder();
                let mut current_bone = 0;
                builder = builder.push_cube(material.clone(), 1.0, current_bone);

                const LENGTH_TAIL: usize = 10;

                for (material, animation) in
                    iter::repeat_n((material, animation_rc), LENGTH_TAIL - 1)
                {
                    let bone = Bone::builder(current_bone)
                        .all_animations(animation)
                        .build();

                    current_bone = skeleton.push_bone(bone).unwrap();
                    builder = builder.push_cube(material, 1.0, current_bone);
                }

                builder
                    .cull_face(CullFace::BackFace)
                    .skeleton(skeleton)
                    .build()
            }
            _ => Cubic::empty(),
        };

        const START_LOC: [f32; 3] = [0.0, 1.0, 3.0];

        let camera = camera::builder()
            .pose(
                CameraPose::new_fixed_up_from_to(
                    START_LOC.into(),
                    [0.0, 0.0, 0.0].into(),
                    UnitVector::new_unchecked([0.0, 1.0, 0.0]),
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

        let sun_colour = ColourRGB::new([1.0, 1.0, 1.0]);
        let far_light = ShadowFarLight::new(
            FarLight {
                direction: Vector::new([0.0, -0.99, 0.001]).normalize(),
                ambient: sun_colour.map(|x| x * 0.1),
                diffuse: sun_colour.map(|x| x * 0.5),
                specular: sun_colour.map(|x| x * 1.0),
            },
            (1000.into(), 1000.into()),
        );

        let sun2_colour = ColourRGB::new([1.0, 1.0, 1.0]);
        let far_light2 = ShadowFarLight::new(
            FarLight {
                direction: Vector::new([1.0, 0.0, 0.0]).normalize(),
                ambient: sun2_colour.map(|x| x * 0.0),
                diffuse: sun2_colour.map(|x| x * 0.5),
                specular: sun2_colour.map(|x| x * 1.0),
            },
            (1000.into(), 1000.into()),
        );

        let spotlight_colour = ColourRGB::new([1.0; 3]);
        let spotlight = ShadowSpotLight::new(
            SpotLight {
                position: camera.centre(()),
                direction: camera.direction(()),
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

        let string = String::new();

        let orientation1 = Orientation::new_from_to(
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.0, 1.0].into(),
            Vector::from([0.0, 1.0, 0.0]).normalize(),
        );
        let orientation2 = Orientation::new_from_to(
            [0.0, 0.0, 0.0].into(),
            [0.01, 0.0, -1.0].into(),
            Vector::from([0.0, 1.0, 0.0]).normalize(),
        );


        let quad_to_draw = Quad::screen(hdr_fb.get_all_colour()).downcast();

        let light_material = {
            let light_emission = FlatTexture::monochrome(light_colour.to_rgba_with(1.0));

            Material::builder().emission(light_emission).build()
        };

        let light = Cubic::cube(1.0, Rc::new(Material::blank()))
            .material(Rc::new(light_material))
            .cull_face(CullFace::BackFace)
            .scale(0.2)
            .build();

        Ok(Self {
            time: 0.0,
            string,

            camera,
            exposure,
            sensitivity,
            speed,

            light,
            containers,
            skybox,

            imported,
            which_animation: 0,

            hdr_fb,
            do_bloom: false,
            bloom,

            light_group: ShadowListLights {
                point: ArrayVec::try_from([point_light])
                    .expect("size leq SHADOW_SHADER_MAX_LIGHTS"),
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
