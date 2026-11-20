use alloc::rc::Rc;
use core::num::NonZero;

use engine::entity::Mass;
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals as _;
use engine::geometry::Pose;
use engine::linear_algebra::{Vector, vector};
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::lighting::shadow::ShadowPointLight;
use engine::modelling::cubic::lighting::simple::PointLight;
use engine::modelling::cubic::material::Material;
use engine::modelling::{Bloom, SkyBox};
use engine::texture::{CubeMap, FlatTexture, Image, TextureHasBuilder as _, WrapType};
use engine::types::TexDim;
use engine::{ColourRGB, SimpleEntity};

use crate::camera::GameCamera;
use crate::player::Player;
use crate::state::State;
use crate::sun::Sun;

impl State {
    pub fn new_internal(screen_dims: (TexDim, TexDim)) -> Self {
        let exposure = 1.0;

        let gbuffer = Builder::new_flat().depth().size(screen_dims).build();

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
                //.grayscale(0.0)
                .build(),
        );

        const ORIGIN_CUBE_SIDE_LENGTH: f32 = 0.5;
        let origin_cube = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(
                ORIGIN_CUBE_SIDE_LENGTH,
                Rc::new(
                    Material::builder()
                        .diffuse(
                            FlatTexture::builder()
                                .image(
                                    Image::new_from_srgba("assets/brickwall/brickwall.jpg")
                                        .unwrap(),
                                )
                                .wrap_s_t(WrapType::ClampToEdge)
                                .build(),
                        )
                        .normal_map(
                            FlatTexture::builder()
                                .image(Image::new_from_rgba("assets/brickwall/normal.png").unwrap())
                                .wrap_s_t(WrapType::ClampToEdge)
                                .build(),
                        )
                        .shininess(32.0)
                        .build(),
                ),
            )
            .apply_transformation(const { Pose::new_from_translation(vector![0.0, 0.0, 0.0,]) })
            .build(),
            0,
            Mass::new_inverse(0.0),
            |_| (),
        );

        const SMALL_CUBE_SIDE_LENGTH: f32 = 0.2;
        let small_cube = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(SMALL_CUBE_SIDE_LENGTH, Rc::new(Material::white()))
                .apply_transformation(
                    const {
                        Pose::new_from_translation(vector![
                            0.0,
                            ORIGIN_CUBE_SIDE_LENGTH.mul_add(0.5, SMALL_CUBE_SIDE_LENGTH * 0.5),
                            0.0,
                        ])
                    },
                )
                .build(),
            0,
            Mass::new_inverse(0.0),
            |_| (),
        );

        let player = Player::new();

        let game_camera = GameCamera::new::<3, _>(&gbuffer);

        let mut bear = make_model();
        let bear_colour = const { ColourRGB::WHITE.map(const |x| x * 3.0).to_rgba_with(1.0) };

        bear.temp_set_all_material(Rc::new(
            Material::builder()
                .emission(FlatTexture::monochrome(bear_colour))
                .build(),
        ));
        let sun = SimpleEntity::from_simple_cubic(bear, 20, Mass::new_inverse(1.0), |_| ());

        let light_attenuation_array = [0.0, 0.0, 1.0];

        let inner_point_light = ShadowPointLight::new(
            PointLight {
                position: Vector::default(),
                attenuation: light_attenuation_array,
                ambient: bear_colour.to_rgb().map(|x| x * 0.2),
                diffuse: bear_colour.to_rgb().map(|x| x * 0.0),
                specular: bear_colour.to_rgb().map(|_| 0.0),
            },
            const { TexDim::from_non_zero(NonZero::new(1000).unwrap()) },
        );

        let sun = Sun {
            inner: sun,
            light: inner_point_light,
        };

        let hdr_fb = Builder::new_flat().size(gbuffer.size()).depth().build();
        let ambient_occlusion_buffer = Builder::new_flat().size(gbuffer.size()).build();

        let bloom = Bloom::new(gbuffer.size());

        Self {
            game_camera,
            exposure,
            ambient_occlusion_buffer,

            skybox,
            hdr_out: hdr_fb,

            sun,
            origin_cube,
            small_cube,
            player,

            toggle_m: 1.0,

            gbuffer,
            toggle_g: 2,
            bloom,
        }
    }
}

pub fn make_model() -> SimpleCubic {
    // use engine::PostProcess as P;
    // let model = SimpleCubic::import(
    // "assets/Teddy Statue/Teddy Statue.obj",
    // vec![P::Triangulate, P::OptimizeGraph, P::OptimizeMeshes],
    // true,
    // )
    // .expect("asset should exist and be correct")
    // .cull_face(CullFace::BackFace)
    // .apply_transformation(Pose::new_from_translation(vector![0.0, -133.0,
    // 43.0])) .scale(0.006)
    // .build();

    let model = SimpleCubic::cube(1.0, Rc::new(Material::white())).build();

    // let backpack = SimpleCubic::import(
    // "assets/backpack/backpack.obj",
    // vec![
    // P::Triangulate,
    // P::FlipUVs,
    // P::OptimizeGraph,
    // P::OptimizeMeshes,
    // ],
    // false,
    // )
    // .unwrap()
    // .material(Rc::new(Material::white()))
    // .build();
    model
}
