use std::rc::Rc;

use engine::entity::{AggregateEntity, AggregateModelIndex, ToAggregateEntity};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::SkeletonBoneIndex;
use engine::modelling::cubic::geometry::{
    AggregateCalcAnim,
    BoneAnimation,
    Pose,
    SkeletalAnimation,
};
use engine::modelling::cubic::material::Material;
use engine::modelling::{Bone, Collision, Skeleton, SkeletonCubic, TempModelTransform};
use engine::{ColourRGBA, Transform};

pub struct GrapplingHook {
    pub entity: AggregateEntity<(), SkeletonCubic>,
    pub chain: AggregateModelIndex,
    pub chain_end_bone: SkeletonBoneIndex,
    pub hook: Option<AggregateModelIndex>,
    pub chain_animation: SkeletalAnimation,
    len: f32,
    hook_state: HookState,
}

#[derive(Debug)]
enum HookState {
    Still,
    Extending {
        length: f32,
        velocity: f32,
    },
    CarryingPlayer {
        length: f32,
        velocity: f32,
        fixed_pose: Pose,
    },
    GrabbedObject {
        length: f32,
        attachment: AggregateModelIndex,
        velocity: f32,
    },
}

impl GrapplingHook {
    pub fn new(len: usize) -> Self {
        // Need origin cube
        // Need chain
        // Need hook

        const ORIGIN_BOX_SIZE: f32 = 1.0;
        let origin_cube = SkeletonCubic::cube(
            ORIGIN_BOX_SIZE,
            Rc::new(Material::from_colour(ColourRGBA::RED)),
        )
        .build();

        let mut chain = SkeletonCubic::builder();
        let mut skeleton_builder = Skeleton::builder();
        let mut animation_builder = SkeletalAnimation::builder();
        let mut bones_root_to_tip = vec![SkeletonBoneIndex::SKELETON_ROOT_BONE];
        // No chain link is bound to the root bone, so that the root of the origin cube
        // and the
        let white_material = Rc::new(Material::white());

        const EXPECT_MESSAGE: &str = "have not yet finished the animation";
        const CHAIN_LINK_SIZE: f32 = 0.5;
        const SEC_PER_UNIT: f32 = CHAIN_LINK_SIZE;

        for _ in 1..len + 1 {
            // bone 0 is the root so we need one more than LEN
            // Create new bone to correspond to the created animation
            let new_bone = Bone::builder(bones_root_to_tip.last().expect("filled"))
                .bind_pose_relative_to_parent(Pose::IDENTITY);

            let end_bone = skeleton_builder
                .push_bone(new_bone)
                .expect("new_bone parent exists");

            bones_root_to_tip.push(end_bone);
            // create skeleton, and save to a vector
        }

        for bone in &bones_root_to_tip {
            // add cube to each bone
            chain.push_cube(
                Pose::IDENTITY,
                white_material.clone(),
                CHAIN_LINK_SIZE,
                &bone,
            )
        }

        // array_windows is a way to ensure that the last index is skipped
        for (
            mut index,
            [
                parent,
                _animation_owner, /* this bone here is the one who is actually owning the
                                   * animation */
            ],
        ) in bones_root_to_tip.array_windows().enumerate()
        {
            index += 1; // effectively enumerating the second element, the animation owner, of the Window
            // this is because the root node is never animated.
            // index == 1 means first box

            // Create Animation, using bone as the current bone
            let mut bone_animation = BoneAnimation::new();
            // do not move for `LEN-index` timesteps
            bone_animation
                .push(Pose::IDENTITY, (len - index) as f32 * SEC_PER_UNIT)
                .expect(EXPECT_MESSAGE);
            // then move one distance in one timesteps
            bone_animation
                .push(Pose::IDENTITY, SEC_PER_UNIT)
                .expect(EXPECT_MESSAGE);
            bone_animation
                .push_final_non_repeat(
                    Pose::new_from_translation(Vector::new([0.0, 0.0, CHAIN_LINK_SIZE])),
                    //(2 * (index - 1)) as f32 * SEC_PER_UNIT,
                )
                .expect(EXPECT_MESSAGE);
            // bone_animation
            // .push(
            // Pose::new_from_translation(Vector::new([0.0, 0.0, CHAIN_LINK_SIZE])),
            // SEC_PER_UNIT,
            // )
            // .expect(EXPECT_MESSAGE);
            // bone_animation
            // .push_final_non_repeat(Pose::IDENTITY)
            // .expect(EXPECT_MESSAGE);

            animation_builder
                .push_animation(bone_animation, parent)
                .expect("end bone exists");
        }

        let end_of_chain = bones_root_to_tip
            .last()
            .expect("at least going to be the ROOT_BONE");

        let mut bone_animation = BoneAnimation::new();
        bone_animation
            .push_final_non_repeat(Pose::new_from_translation(Vector::new([
                0.0,
                0.0,
                CHAIN_LINK_SIZE * 0.5,
            ])))
            .expect(EXPECT_MESSAGE);

        animation_builder
            .push_animation(bone_animation, &end_of_chain)
            .expect("end bone exists");

        let new_bone = Bone::builder(&end_of_chain).bind_pose_relative_to_parent(
            Pose::new_from_translation(Vector::new([0.0, 0.0, CHAIN_LINK_SIZE * 0.5])),
        );

        let hook_attachment_bone = skeleton_builder
            .push_bone(new_bone)
            .expect("new_bone parent exists");

        let skeleton = skeleton_builder.build();
        let animation = animation_builder.build();
        let chain = chain.skeleton(skeleton).build();

        // `end_bone` now has the bone which I should attach the hook to (plus offset)

        const HOOK_SIZE: f32 = 2.0;
        let hook =
            SkeletonCubic::cube(HOOK_SIZE, Rc::new(Material::from_colour(ColourRGBA::GREEN)))
                .build();

        let mut aggregate = AggregateEntity::new(origin_cube, 1, |_| ());
        let chain_attachment_point = aggregate
            .push_model(
                chain,
                Pose::new_from_translation(Vector::new([
                    0.0,
                    0.0,
                    0.5 * (ORIGIN_BOX_SIZE - CHAIN_LINK_SIZE),
                ])),
                &SkeletonBoneIndex::SKELETON_ROOT_BONE,
                &AggregateModelIndex::ROOT_MODEL_INDEX,
                0,
                |_| (),
            )
            .expect("Root model at ROOT_MODEL_INDEX");
        let hook_attachment_point = aggregate
            .push_model(
                hook,
                Pose::new_from_translation(Vector::new([0.0, 0.0, 0.5 * (HOOK_SIZE)])),
                &hook_attachment_bone,
                &chain_attachment_point,
                1,
                |_| (),
            )
            .expect("chain attachment at given index");

        Self {
            entity: aggregate,
            chain: chain_attachment_point,
            chain_end_bone: hook_attachment_bone,
            hook: Some(hook_attachment_point),
            chain_animation: animation,
            hook_state: HookState::Still,
            len: len as f32 * CHAIN_LINK_SIZE,
        }
    }
}

#[derive(Debug)]
pub struct UpdateInput<'a, A, B, C>
where
    A: Collision + 'a,
    B: Collision + ToAggregateEntity<(), SkeletonCubic> + 'a,
    C: Collision + 'a,
    A::Transform<'a>: Clone,
    B::Transform<'a>: Clone,
    C::Transform<'a>: Clone,
{
    pub grapple_points: Vec<(&'a A, A::Transform<'a>)>,
    pub carryable_objects: Vec<Option<(B, B::Transform<'a>)>>,
    pub other_entities: Vec<(&'a C, C::Transform<'a>)>,
}

impl<'a, A, B, C> Default for UpdateInput<'a, A, B, C>
where
    A: Collision + 'a,
    B: Collision + ToAggregateEntity<(), SkeletonCubic> + 'a,
    C: Collision + 'a,
    for<'b> A::Transform<'b>: Clone,
    for<'b> B::Transform<'b>: Clone,
    for<'b> C::Transform<'b>: Clone,
{
    fn default() -> Self {
        Self {
            grapple_points: vec![],
            carryable_objects: vec![],
            other_entities: vec![],
        }
    }
}

const VELOCITY: f32 = 1.0;

impl GrapplingHook {
    pub fn update<'a, A, B, C>(
        &mut self,
        timestep: f32,
        target: &mut UpdateInput<'a, A, B, C>,
    ) -> AggregateCalcAnim
    where
        A: Collision + 'a,
        B: Collision + ToAggregateEntity<(), SkeletonCubic> + 'a,
        C: Collision + 'a,
        for<'b> A::Transform<'b>: Clone,
        for<'b> B::Transform<'b>: Clone,
        for<'b> C::Transform<'b>: Clone,
    {
        eprintln!("{:?}", self.hook_state);
        match &mut self.hook_state {
            HookState::Still => self
                .entity
                .calculate_animation(None, [(&self.chain, (&self.chain_animation, 0.0))].into())
                .expect("chain_animation is well defined for the chain element"),
            HookState::Extending { length, velocity } => {
                *length += *velocity * timestep;

                if *length < 0.0 {
                    self.hook_state = HookState::Still;
                    return self.update(0.0, target);
                }

                if *length > self.len {
                    *velocity = -*velocity;
                    *length = self.len;
                    return self.update(0.0, target);
                }

                let mut self_transform = self
                    .entity
                    .calculate_animation(
                        None,
                        [(&self.chain, (&self.chain_animation, *length))].into(),
                    )
                    .expect("chain_animation is well defined for the chain element");

                for (grapple_targets, grapple_transform) in target.grapple_points.iter().cloned() {
                    let mut collided = false;

                    self.entity.collides(
                        &self_transform,
                        grapple_targets,
                        grapple_transform.clone(),
                        |(ref self_aggr, _), _, _, _| {
                            if Some(self_aggr) == self.hook.as_ref() {
                                collided = true;
                            }
                        },
                    );

                    if collided {
                        let hook_pos = self_transform
                            .get(self.hook.as_ref().expect("cannot hook without a hook!"))
                            .expect("hook still exists")
                            .root();
                        // this one needs to be fixed for ever.

                        let new_state = HookState::CarryingPlayer {
                            length: *length,
                            velocity: VELOCITY,
                            fixed_pose: hook_pos,
                        };
                        self.hook_state = new_state;

                        return self_transform;
                    }
                }

                for opt_carry in target.carryable_objects.iter_mut() {
                    if let Some((ref_target, target_transform)) = opt_carry.as_ref() {
                        let mut collided = false;

                        self.entity.collides(
                            &self_transform,
                            ref_target,
                            target_transform.clone(),
                            |(ref self_aggr, _), _, _, _| {
                                if Some(self_aggr) == self.hook.as_ref() {
                                    collided = true;
                                }
                            },
                        );

                        if collided {
                            let (grapple_target, target_transform) =
                                opt_carry.take().expect("Asserted Some above");

                            let hook_pos = self_transform
                                .get(self.hook.as_ref().expect("cannot hook without a hook!"))
                                .expect("hook still exists")
                                .root();

                            let target_pose = target_transform.get_root_pose();

                            let offset = target_pose.transform(&hook_pos.inverse());

                            let destination = self
                                .entity
                                .push(
                                    grapple_target,
                                    offset,
                                    self.hook.as_ref().expect("cannot hook without a hook!"),
                                    &SkeletonBoneIndex::SKELETON_ROOT_BONE,
                                )
                                .expect("hook exists");
                            let length = *length;

                            self.hook_state = HookState::GrabbedObject {
                                length,
                                attachment: destination,
                                velocity: VELOCITY,
                            };

                            self_transform = self
                                .entity
                                .calculate_animation(
                                    None,
                                    [(&self.chain, (&self.chain_animation, length))].into(),
                                )
                                .expect("animation after grabbing");

                            return self_transform;
                        }
                    }
                }

                for (other, other_tranform) in target.other_entities.iter().cloned() {
                    let mut collided = false;

                    self.entity.collides(
                        &self_transform,
                        other,
                        other_tranform.clone(),
                        |(ref self_aggr, _), _, _, _| {
                            if Some(self_aggr) == self.hook.as_ref() {
                                collided = true;
                            }
                        },
                    );

                    if collided {
                        *velocity = -*velocity;
                        return self_transform;
                    }
                }

                self_transform
            }
            &mut HookState::CarryingPlayer {
                velocity,
                ref mut length,
                fixed_pose,
            } => {
                *length -= velocity * timestep;

                if *length < 0.0 {
                    *length = 0.0;
                }

                let transform = self
                    .entity
                    .calculate_animation(
                        Some((
                            self.hook
                                .as_ref()
                                .expect("cannot carry a player without a hook"),
                            &SkeletonBoneIndex::SKELETON_ROOT_BONE,
                            fixed_pose,
                        )),
                        [(&self.chain, (&self.chain_animation, *length))].into(),
                    )
                    .expect("chain_animation is well defined for the chain element");

                if *length == 0.0 {
                    self.hook_state = HookState::Still;
                    self.entity.base_pose = transform.root().root()
                }

                transform
            }
            HookState::GrabbedObject {
                length, velocity, ..
            } => {
                *length -= *velocity * timestep;

                if *length < 0.0 {
                    self.hook_state = HookState::Still;
                    // FIXME
                    // LOSES ACCESS TO THE ITEM POINTER
                    self.update(0.0, target)
                } else {
                    self.entity
                        .calculate_animation(
                            None,
                            [(&self.chain, (&self.chain_animation, *length))].into(),
                        )
                        .expect("animation after grabbing")
                }
            }
        }
    }
}

impl GrapplingHook {
    pub fn start(&mut self) {
        if matches!(self.hook_state, HookState::Still) {
            self.hook_state = HookState::Extending {
                length: 0.0,
                velocity: VELOCITY,
            }
        };
    }
}
