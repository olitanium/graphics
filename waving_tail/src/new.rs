use engine::ColourRGB;
use engine::entity::{AggregateEntity, AggregateModelIndex};
use engine::framebuffer::Builder;
use engine::framebuffer::traits::FramebufferInternals;
use engine::geometry::{Orientation, Pose};
use engine::linear_algebra::{UnitVector, Vector, vector};
use engine::modelling::cubic::lighting::simple::PointLight;
use engine::modelling::cubic::{CameraBuilder, ManualPoseFixedUp};
use engine::modelling::{Bloom, SkyBox, test_models};
use engine::texture::{CubeMap, TextureHasBuilder};
use engine::types::TexDim;

use crate::state::{State, StateError};

impl State {
    pub fn new_internal(size: (TexDim, TexDim)) -> Result<Self, StateError> {
        let speed = [1.0, 1.0, 1.0];
        let exposure = 1.0;
        const START_LOC: Vector<3> = vector![0.0, 0.0, -1.0];

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
        let camera = CameraBuilder::new()
            .pose(ManualPoseFixedUp::new_from_to(START_LOC, Vector::ZERO, UnitVector::Y).unwrap())
            .perspective(90.0, hdr_fb.aspect_ratio(), 0.01, 100.0)
            .build();

        let light_colour = ColourRGB::new([1.0, 1.0, 1.0].map(|x| x * 1.0));
        let light_attenuation_array = [1.0, 0.0, 0.0];

        let point_light = PointLight {
            position: Vector::default(),
            attenuation: light_attenuation_array,
            ambient: light_colour.map(|x| x * 0.1),
            diffuse: light_colour.map(|x| x * 0.5),
            specular: light_colour,
        };

        let bloom = Bloom::new(size);

        let (simple_tail, end_attachment_point, animation) = test_models::tail_skeleton_cubic(10);

        let mut tail = AggregateEntity::new(simple_tail.clone(), 1, |_| ());
        let offset = Pose::new_from_orientation_translation(
            Orientation::new_forward_up(UnitVector::Y, -UnitVector::Z).unwrap(),
            vector![0.0, 0.5, 0.5],
        );

        let other_tail_index = tail
            .push_model(
                simple_tail.clone(),
                offset,
                &end_attachment_point,
                &AggregateModelIndex::ROOT_MODEL_INDEX,
                1,
                |_| (),
            )
            .unwrap();

        Ok(Self {
            camera,
            exposure,
            sensitivity,
            speed,

            skybox,

            hdr_fb,
            bloom,

            tail,
            tail_animation: animation,
            other_tail_index,
            most_recent_animation: None,

            point_light,
        })
    }
}
