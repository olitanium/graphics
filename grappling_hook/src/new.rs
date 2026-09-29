use alloc::rc::Rc;
use core::num::NonZero;

use engine::entity::Mass;
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::geometry::Pose;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::lighting::shadow::{ShadowPointLight, ShadowSpotLight};
use engine::modelling::cubic::lighting::simple::{PointLight, SpotLight};
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{CameraBuilder, ManualPoseFixedUp, SimpleCubic};
use engine::modelling::{Bloom, SkyBox};
use engine::shader_program::CullFace;
use engine::texture::{CubeMap, TextureHasBuilder};
use engine::types::TexDim;
use engine::{ColourRGB, SimpleEntity};

use crate::bear::Bear;
use crate::grappling_hook::{GrapplingHook, UpdateInput};
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

        let mut grappling_hook = GrapplingHook::new(3);
        let grapple_animation =
            grappling_hook.update(0.0, &mut UpdateInput::<Bear, Bear, Bear>::default());

        Ok(Self {
            toggle_b: false,
            time: 0.0,
            game_start: None,
            camera,
            exposure,
            sensitivity,
            speed,

            skybox,

            bear: Some(bear),
            origin_cube,
            bounding_box_level,
            toggle_m: false,

            grappling_hook,
            grapple_animation,

            hdr_fb,
            toggle_g: false,
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
        _ => SimpleCubic::empty(),
    };
    Ok(model)
}
