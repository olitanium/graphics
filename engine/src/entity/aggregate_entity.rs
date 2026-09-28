use core::fmt::{self, Debug, Display};
use core::{iter, mem};
use std::collections::HashMap;

use geometry::Pose;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;

use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{
    AggregateCalcAnim,
    BuildFixingBoneError,
    CalcAnimUnitIndex,
    CalculatedAnimation,
    PushAnimationError,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
    SkeletonBoneIndex,
};
use crate::modelling::{Collision, CubicVertex, Skeleton};

mod group;
pub use group::Group as AggregateGroup;

mod error;
pub use error::PushModelError;

mod from_to;
pub use from_to::ToAggregateEntity;

mod entity_inner;
pub use entity_inner::EntityInner;

pub struct AggregateEntity<M: EntityInner, T>
where
    M::Vertex: CubicVertex,
{
    base_model: M,
    base_bounding_box: Option<M::BoundingBox<T>>,
    pub base_pose: Pose,
    children: Vec<Option<AggregateUnit<M, T>>>,
}

impl<T, M: EntityInner + Debug> Debug for AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
    M::BoundingBox<T>: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(AggregateEntity))
            .field("base_model", &self.base_model)
            .field("base_bounding_box", &self.base_bounding_box)
            .field("base_pose", &self.base_pose)
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Debug)]
struct AggregateUnit<M: EntityInner, T>
where
    M::Vertex: CubicVertex,
{
    model: M,
    bounding_box: Option<M::BoundingBox<T>>,
    offset: Pose,
    skeleton_attachment_point: SkeletonBoneIndex,
    parent_unit: AggregateModelIndex,
    children: Vec<AggregateModelIndex>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[must_use = "AggregateModelIndex is a handle into the entity, it is used to set animations and as \
              an attachment point for further units"]
pub struct AggregateModelIndex(pub(crate) usize);

impl AggregateModelIndex {
    pub const ROOT_MODEL_INDEX: Self = Self(0);

    pub(crate) const fn inner_clone(&self) -> Self {
        Self(self.0)
    }
}

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
{
    pub fn new<F>(base_model: M, depth: usize, init: F) -> Self
    where
        F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T,
    {
        let bounding_box = base_model.make_bounding_box(depth, init);

        Self {
            base_bounding_box: bounding_box,
            base_model,
            base_pose: Pose::IDENTITY,
            children: Vec::new(),
        }
    }
}

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
{
    pub fn push_model<F>(
        &mut self,
        other_model: M,
        offset: Pose,
        attachment_point: &SkeletonBoneIndex,
        parent_unit: &AggregateModelIndex,

        bounding_box_depth: usize,
        bounding_box_init: F,
    ) -> Result<AggregateModelIndex, PushModelError>
    where
        F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T,
    {
        // check that the parent unit is Some and the bone exists
        self.check_parent_and_bone(attachment_point, parent_unit)?;

        let bounding_box = other_model.make_bounding_box(bounding_box_depth, bounding_box_init);

        let child = AggregateUnit {
            model: other_model,
            bounding_box,
            offset,
            skeleton_attachment_point: attachment_point.clone_inner(),
            parent_unit: parent_unit.inner_clone(),
            children: Vec::new(),
        };
        // get index of the first None

        let child_at = self.push_aggregate_unit(child);

        Ok(child_at)
    }

    fn check_parent_and_bone(
        &self,
        attachment_point: &SkeletonBoneIndex,
        parent_unit: &AggregateModelIndex,
    ) -> Result<(), PushModelError> {
        let model = if parent_unit == &AggregateModelIndex::ROOT_MODEL_INDEX {
            &self.base_model
        } else {
            &self
                .children
                .get(parent_unit.0 - 1)
                .ok_or(PushModelError::ParentUnitDoesNotExist)?
                .as_ref()
                .ok_or(PushModelError::ParentUnitDoesNotExist)?
                .model
        };

        model
            .skeleton()
            .get(attachment_point)
            .is_bone()
            .ok_or(PushModelError::BoneInParentDoesNotExist)
    }
}

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
{
    fn push_aggregate_unit(&mut self, unit: AggregateUnit<M, T>) -> AggregateModelIndex {
        let parent_unit = unit.parent_unit.inner_clone();
        let child_at = if let Some((index, child_dest)) = self
            .children
            .iter_mut()
            .enumerate()
            // skip here ensures that the parent is always above the child.
            // note that the `self.children` is offset by 1, so if the parent is ROOT (0)
            // no children are skipped, if the parent is 1, only the first (the parent itself) is
            // skipped, etc.
            .skip(unit.parent_unit.0)
            .find(|(_, opt_unit)| opt_unit.is_none())
        {
            *child_dest = Some(unit);
            AggregateModelIndex(index + 1)
        } else {
            self.children.push(Some(unit));
            let output = self.children.len();
            AggregateModelIndex(output)
        };

        if parent_unit != AggregateModelIndex::ROOT_MODEL_INDEX {
            let parent_mut = self
                .children
                .get_mut(parent_unit.0 - 1)
                .expect("parent in aggregate unit must exist")
                .as_mut()
                .expect("parent in aggregate unit must exist");
            parent_mut.children.push(child_at.inner_clone());
        }

        child_at
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "I want to remove the `AggregateModelIndex` from the user to ensure that they do \
                  not double drop"
    )]
    pub fn remove(&mut self, index: AggregateModelIndex) {
        if index == AggregateModelIndex::ROOT_MODEL_INDEX {
            return;
        }
        if let Some(Some(unit)) = self.children.get_mut(index.0 - 1) {
            for unit_index in mem::take(&mut unit.children) {
                self.remove(unit_index);
            }
        }
        if let Some(opt_empty_unit) = self.children.get_mut(index.0 - 1) {
            *opt_empty_unit = None;
        }
    }

    // fn split_off_inner<E: FromToAggregateEntity<T, Model = M>>(&mut self,
    // index: AggregateModelIndex, animation: Option<AggregateCalcAnim>) ->
    // Option<Self> { if index == AggregateModelIndex::ROOT_MODEL_INDEX {
    // return None;
    // }
    //
    // if let Some(Some(unit)) = self.children.get_mut(index.0 -
    // 1).map(mem::take) { let offset = unit.offset;
    // let base_pose = if let Some(animation) = animation {animation.get(index)}
    // else {Pose::IDENTITY}; let aggregate_unit = Self {
    // base_model: unit.model,
    // base_bounding_box: unit.bounding_box,
    // children: vec![],
    // };
    //
    //
    //
    // model.push_model(add_model, offset, attachment_point, parent_unit,
    // bounding_box_depth, bounding_box_init) }
    //
    // todo!()
    //
    // }

    // get the one to delete and check it's existence. take it from the vec and
    // create an AggregateEntity and locate it's children. for each child,
    // recursively, pop it, add it to the new AggrEnt and recursively do so with
    // it's children
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CalculateAnimationError<'a> {
    BadAnimationInput {
        error: SignaturesDoNotMatchError,
        parent: &'a AggregateModelIndex,
    },
    FixAnimationUnitDoesNotExist,
    FixAnimationBoneDoesNotExist,
}
error_boilerplate!(<'a> CalculateAnimationError<'a>);

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
{
    pub fn calculate_animation<'a>(
        &'a self,
        fix_anim: Option<(&'a AggregateModelIndex, &'a SkeletonBoneIndex, Pose)>,
        mut animations: HashMap<&'a AggregateModelIndex, (&'a SkeletalAnimation, f32)>,
    ) -> Result<AggregateCalcAnim, CalculateAnimationError<'a>> {
        let indices = (1..).map(AggregateModelIndex);
        let first_animation = animations.remove(&AggregateModelIndex::ROOT_MODEL_INDEX);

        let mut anim_builder =
            AggregateCalcAnim::builder(self.base_pose, self.base_model.skeleton(), first_animation)
                .map_err(|error| CalculateAnimationError::BadAnimationInput {
                    error,
                    parent: &AggregateModelIndex::ROOT_MODEL_INDEX,
                })?;

        let mut parent_locations = HashMap::from([(
            AggregateModelIndex::ROOT_MODEL_INDEX,
            CalcAnimUnitIndex::ROOT_UNIT_INDEX,
        )]);

        for (
            AggregateUnit {
                model,
                offset,
                skeleton_attachment_point: attachment_point,
                parent_unit: parent_model_unit,
                ..
            },
            index,
        ) in self
            .children
            .iter()
            .zip(indices)
            .filter_map(|(opt, index)| opt.as_ref().map(|unit| (unit, index)))
        {
            let animation = animations.get(&index).copied();
            let parent_unit_index = parent_locations
                .get(parent_model_unit)
                .expect("parent comes before child in `self.children`, per the push_unit method");
            let destination = anim_builder
                .push(
                    *parent_unit_index,
                    attachment_point,
                    *offset,
                    model.skeleton(),
                    animation,
                )
                .map_err(|error| match error {
                    PushAnimationError::SignaturesDoNotMatch(error) => {
                        CalculateAnimationError::BadAnimationInput {
                            error,
                            parent: parent_model_unit,
                        }
                    }
                    _ => unreachable!("should be well formed animation tree"),
                })?;
            parent_locations.insert(index, destination);
        }

        let anim = if let Some((unit_index, bone_index, pose)) = fix_anim {
            let unit_index = *parent_locations
                .get(unit_index)
                .ok_or(CalculateAnimationError::FixAnimationUnitDoesNotExist)?;
            anim_builder
                .build_fixing_bone(unit_index, bone_index, pose)
                .map_err(|error| match error {
                    BuildFixingBoneError::BoneDoesNotExist => {
                        CalculateAnimationError::FixAnimationBoneDoesNotExist
                    }
                    BuildFixingBoneError::UnitDoesNotExist => {
                        unreachable!("checked above when getting CalcAnimUnitIndex")
                    }
                })?
        } else {
            anim_builder.build()
        };

        Ok(anim)
    }

    pub(crate) fn zip_calculated_animation_opt<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = Option<(
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        )>,
    > {
        let mut aggr_iter = aggr.vec.iter();
        iter::once(Some((
            &self.base_model,
            self.base_bounding_box.as_ref(),
            aggr_iter
                .next()
                .expect("CalcAnim must have been created from this entity"),
        )))
        .chain(self.children.iter().map(move |child| {
            child.as_ref().map(|aggr| {
                (
                    &aggr.model,
                    aggr.bounding_box.as_ref(),
                    aggr_iter
                        .next()
                        .expect("CalcAnim must have been created from this entity"),
                )
            })
        }))
    }

    pub(crate) fn zip_calculated_animation<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = (
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        ),
    > {
        self.zip_calculated_animation_opt(aggr).flatten()
    }

    pub(crate) fn zip_calculated_animation_get_index_opt<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = Option<(
            AggregateModelIndex,
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        )>,
    > {
        self.zip_calculated_animation_opt(aggr)
            .enumerate()
            .map(|(index, opt)| opt.map(|(x, y, z)| (AggregateModelIndex(index), x, y, z)))
    }

    pub(crate) fn zip_calculated_animation_get_index<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = (
            AggregateModelIndex,
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        ),
    > {
        self.zip_calculated_animation_get_index_opt(aggr).flatten()
    }
}

pub enum DrawError<'a, M: EntityInner + 'a>
where
    M::Vertex: CubicVertex,
{
    ModelDraw(M::DrawError<'a>),
}

impl<'a, M: EntityInner + 'a> Debug for DrawError<'a, M>
where
    M::Vertex: CubicVertex,
    M::DrawError<'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawError::ModelDraw(err) => Debug::fmt(err, f),
        }
    }
}

impl<'a, M: EntityInner + 'a> Display for DrawError<'a, M>
where
    M::Vertex: CubicVertex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[expect(clippy::absolute_paths, reason = "there already is a mod error;")]
impl<'a, M: EntityInner + 'a> core::error::Error for DrawError<'a, M> where M::Vertex: CubicVertex {}

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
{
    pub(crate) fn draw<'a, 'b, const MAX: usize, const OUT: usize, F: FramebufferWithDepth<OUT>>(
        &'a self,
        active_sp: &mut ActiveShaderProgram<'b, '_, 'a, M, ListLights<MAX>, F::Tex, OUT>,
        active_fb: &mut ActiveFramebuffer<'_, '_, OUT, F>,
        calculated_animation: &AggregateCalcAnim,
    ) -> Result<(), DrawError<'b, M>> {
        for (model, _, calc) in self.zip_calculated_animation(calculated_animation) {
            model
                .draw(active_sp, active_fb, calc)
                .map_err(|err| DrawError::<M>::ModelDraw(err))?;
        }

        Ok(())
    }
}

impl<M: EntityInner, T> AggregateEntity<M, T>
where
    <M as Model>::Vertex: CubicVertex,
{
    pub fn push<E>(
        &mut self,
        entity: E,
        offset: Pose,
        parent_unit: &AggregateModelIndex,
        skeleton_attachment: &SkeletonBoneIndex,
    ) -> Result<AggregateModelIndex, PushModelError>
    where
        E: ToAggregateEntity<M, BoundingBoxInner = T>,
    {
        self.check_parent_and_bone(skeleton_attachment, parent_unit)?;

        let (model, bounding_box) = entity.to_pair_model_bbox();
        let unit = AggregateUnit {
            model,
            bounding_box,
            offset,
            skeleton_attachment_point: skeleton_attachment.clone_inner(),
            parent_unit: parent_unit.inner_clone(),
            children: Vec::new(),
        };

        Ok(self.push_aggregate_unit(unit))
    }
}

impl<M: EntityInner, T> Collision for AggregateEntity<M, T>
where
    M::Vertex: CubicVertex,
    for<'a> <M::BoundingBox<T> as Collision>::Transform<'a>:
        From<(&'a Skeleton, &'a CalculatedAnimation)>,
{
    type FuncInputs = (
        AggregateModelIndex,
        <M::BoundingBox<T> as Collision>::FuncInputs,
    );
    type Meta = <M::BoundingBox<T> as Collision>::Meta;
    type Transform<'a> = AggregateCalcAnim;
    type Vertex = <M::BoundingBox<T> as Collision>::Vertex;
    type VertexTransform<'a> = ();

    fn collides<
        Other: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, Other::FuncInputs, &mut Other::Meta) -> bool,
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        _self_vertex_transform: &Self::VertexTransform<'_>,
        target: &Other,
        target_transform: &Other::Transform<'_>,
        target_vertex_transform: &Other::VertexTransform<'_>,
        mut alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [Other::Vertex; 3])>> {
        let mut out = Vec::new();
        self.zip_calculated_animation_get_index(self_transform)
            .filter_map(|(x, y, opt_z, t)| opt_z.map(|z| (x, y, z, t)))
            .flat_map(|(index, model, bbox, animation)| {
                bbox.collides(
                    &model.box_transform(animation),
                    &model.vertex_transform(animation),
                    target,
                    target_transform,
                    target_vertex_transform,
                    |x, y, z, t| alter_meta((index.inner_clone(), x), y, z, t),
                )
                .unwrap_or_default()
            })
            .collect_into(&mut out);

        if out.is_empty() { None } else { Some(out) }
    }
}

// impl<'a> TempModelTransform for &'a AggregateCalcAnim {
// fn get_root_pose(&self) -> Pose {
// self.root().root()
// }
// }
