use alloc::rc::Rc;
use core::num::NonZero;

use engine::entity::Mass;
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals as _;
use engine::geometry::Pose;
use engine::linear_algebra::{Vector, vector};
use engine::modelling::cubic::lighting::shadow::ShadowPointLight;
use engine::modelling::cubic::lighting::simple::PointLight;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{Mesh, SimpleCubic};
use engine::modelling::{Bloom, IncompleteVertex, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, FlatTexture, TextureHasBuilder as _};
use engine::types::TexDim;
use engine::{ColourRGBA, SimpleEntity};

use crate::camera::GameCamera;
use crate::player::Player;
use crate::state::{State, StateError};
use crate::sun::Sun;

impl State {
    pub fn new_internal(screen_dims: (TexDim, TexDim)) -> Self {
        let exposure = 1.0;

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
                //.grayscale(0.0)
                .build(),
        );

        let origin_slope = {
            let mut builder = SimpleCubic::builder();

            let mut mesh_builder = Mesh::builder();
            mesh_builder.extend_triangle(
                [
                    [[-1.0, 0.0, 0.0], [-1.0, 0.0, 1.0], [1.0, 1.0, 1.0]],
                    [[-1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, -1.0]],
                ]
                .into_iter()
                .map(|arr| {
                    arr.map(|vec @ [x, _y, z]| {
                        IncompleteVertex::new(Vector::new(vec)).texture(vector![x, z])
                    })
                }),
            );

            builder.push_mesh(mesh_builder);
            builder = builder.cull_face(CullFace::BackFace);

            builder
        };

        const ORIGIN_CUBE_SIDE_LENGTH: f32 = 1.0;
        let origin_slope = SimpleEntity::from_simple_cubic(
            origin_slope
                .apply_transformation(Pose::new_from_translation(vector![
                    0.0,
                    -ORIGIN_CUBE_SIDE_LENGTH * 0.5,
                    0.0,
                ]))
                .build(),
            0,
            Mass::new_inverse(0.0),
            |_| (),
        );

        const SMALL_CUBE_SIDE_LENGTH: f32 = 0.3;
        let small_cube = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(SMALL_CUBE_SIDE_LENGTH, Rc::new(Material::white()))
                .apply_transformation(Pose::new_from_translation(Vector::new([
                    0.0,
                    SMALL_CUBE_SIDE_LENGTH * 0.5,
                    0.0,
                ])))
                .build(),
            0,
            Mass::new_inverse(0.0),
            |_| (),
        );

        let player = Player::new();
        let game_camera = GameCamera::new(&hdr_fb);

        let mut bear = make_model("bear").unwrap();
        let bear_colour = ColourRGBA::new([0.5, 0.0, 0.0, 1.0]);
        bear.temp_set_all_material(Rc::new(
            Material::builder()
                .emission(FlatTexture::monochrome(bear_colour))
                .build(),
        ));
        let sun = SimpleEntity::from_simple_cubic(bear, 20, Mass::new_inverse(1.0), |_| ());

        let light_attenuation_array = [0.0, 0.0, 1.0];

        let inner_point_light = PointLight {
            position: Vector::default(),
            attenuation: light_attenuation_array,
            ambient: bear_colour.to_rgb().map(|x| x * 0.1),
            diffuse: bear_colour.to_rgb().map(|x| x * 0.5),
            specular: bear_colour.to_rgb(),
        };

        let point_light = ShadowPointLight::new(
            inner_point_light.clone(),
            TexDim::from_non_zero(const { NonZero::new(1000).unwrap() }),
        );

        let sun_duplicate_light = ShadowPointLight::new(
            inner_point_light,
            TexDim::from_non_zero(const { NonZero::new(1000).unwrap() }),
        );

        let sun = Sun {
            inner: sun.into(),
            light: point_light,
        };

        let bloom = Bloom::new(hdr_fb.size());

        Self {
            toggle_b: true,

            game_camera,
            exposure,

            skybox,

            sun,
            sun_duplicate_light,
            origin_slope: origin_slope.into(),
            small_cube: small_cube.into(),
            player,

            toggle_m: true,

            hdr_fb,
            toggle_g: true,
            toggle_h: true,
            bloom,
        }
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
        .map_err(StateError::Import)?
        .cull_face(CullFace::BackFace)
        .apply_transformation(Pose::new_from_translation(Vector::new([0.0, -133.0, 43.0])))
        .scale(0.006)
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
        .map_err(StateError::Import)?
        .material(Rc::new(Material::white()))
        .build(),
        _ => SimpleCubic::empty(),
    };
    Ok(model)
}
