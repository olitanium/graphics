use core::cell::{RefCell, RefMut};
use core::fmt::Debug;
use core::{array, iter};

use geometry::Transform;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::ShaderSetUniformError;
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;
use zip_with::ArrayZipWith as _;

use crate::modelling::{BoundingBox, ExtendedVertex, Volume};

mod active_element;

#[derive(Debug, Clone, Default)]
pub(super) struct Meta<T> {
    pub(super) data: RefCell<T>,
    pub(super) triangle_start: usize,
    pub(super) triangle_len: usize,
}

impl<T> Meta<T> {
    pub(super) fn get_triangle_and_meta<'a, V>(
        &'a self,
        bounding_box: &'a BoundingBox<V, T>,
    ) -> (&'a [[V; 3]], RefMut<'a, T>)
    where
        V: ExtendedVertex,
    {
        let refmut = self.data.borrow_mut();
        let triangles = bounding_box
            .triangles
            .get(self.triangle_start..self.triangle_start + self.triangle_len)
            .expect(
                "the `Meta` should be contained within a `BoundingBoxElement` contained within a \
                 relevant `BoundingBox`",
            );

        (triangles, refmut)
    }
}

type const NUMBER_OF_CHILDREN: usize = 2;

#[derive(Debug, Clone)]
pub(super) struct Children {
    children: [usize; NUMBER_OF_CHILDREN],
}

impl Children {
    pub(super) fn get_children_elements<'a, V, T>(
        &self,
        bounding_box: &'a BoundingBox<V, T>,
    ) -> [&'a BoundingBoxElement<V, T>; 2]
    where
        V: ExtendedVertex,
    {
        self.children.map(|index| bounding_box.get(index))
    }
}

#[derive(Debug, Clone)]
pub(super) enum ChildrenOrMeta<T> {
    Meta(Meta<T>),
    Children(Children),
}

#[derive(Debug, Clone)]
pub struct BoundingBoxElement<V: ExtendedVertex, T> {
    pub(crate) lower_bound: Vector<{ V::LEN }>,
    pub(crate) upper_bound: Vector<{ V::LEN }>,

    pub(super) children_or_meta: ChildrenOrMeta<T>,
}

fn make_bound_inner<'a, V, I>(iter: I) -> Option<(Vector<{ V::LEN }>, Vector<{ V::LEN }>)>
where
    I: IntoIterator<Item = &'a V>,
    I::IntoIter: Clone,
    V: ExtendedVertex + 'a,
{
    let iter = iter.into_iter().map(V::position);

    // let lower_bound = iter.clone().reduce(|array1, array2| {
    // Vector::new(array::from_fn(|index| {
    // f32::min(array1[index], array2[index])
    // }))
    // })?;

    let lower_bounds_new = iter
        .clone()
        .map(Vector::into_inner)
        .reduce(|vec1, vec2| vec1.zip_with(vec2, f32::min))?;

    // assert_eq!(
    // lower_bound,
    // Vector::new(lower_bounds_new),
    // "TODO: if pass, implement lower_bounds_new for UB and replace"
    // );
    //
    // let upper_bound = iter.clone().reduce(|array1, array2| {
    // Vector::new(array::from_fn(|index| {
    // f32::max(array1[index], array2[index])
    // }))
    // })?;

    let upper_bounds_new = iter
        .map(Vector::into_inner)
        .reduce(|vec1, vec2| vec1.zip_with(vec2, f32::max))?;

    debug_assert!(
        iter::zip(upper_bounds_new, lower_bounds_new).all(|(x, y)| x >= y),
        "all upper bounts should be the same or higher than the LBs"
    );

    Some((Vector::new(lower_bounds_new), Vector::new(upper_bounds_new)))
}

fn make_bound<V>(
    vectors: &[(Vector<{ V::LEN }>, [V; 3])],
) -> Option<(Vector<{ V::LEN }>, Vector<{ V::LEN }>)>
where
    V: ExtendedVertex,
{
    let iter = vectors.iter().flat_map(|(_, vec)| vec.iter());

    make_bound_inner(iter)
}

impl<V, T> BoundingBoxElement<V, T>
where
    V: ExtendedVertex,
{
    pub fn new_from_many_inner<F>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<{ V::LEN }>, [V; 3])],
        start_index: usize,
        depth: usize,
        init: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(&[(Vector<{ V::LEN }>, [V; 3])]) -> T,
    {
        let (lower_bounds, upper_bounds) = make_bound(triangles)?;

        // TODO: rewrite completely.
        #[expect(clippy::cast_precision_loss, reason = "need to rewrite")]
        let children_or_meta = if depth != 0 {
            let opt_children = {
                let dimensions = upper_bounds - lower_bounds;
                let (max_index, _) = dimensions
                    .into_iter()
                    .enumerate()
                    // input vector const DIMS > 0
                    .max_by(|(_, size), (_, size2)| f32::total_cmp(size, size2))?;

                // sort on the dimension given by max_index
                triangles.sort_unstable_by(|(lhs, _), (rhs, _)| {
                    f32::total_cmp(&lhs[max_index], &rhs[max_index])
                });

                const CHILDREN_FLOAT: f32 = NUMBER_OF_CHILDREN as f32;
                type const CHILDRENPLUS1: usize = const { NUMBER_OF_CHILDREN + 1 };

                let mut new_partition: [_; CHILDRENPLUS1] = array::from_fn(|index| {
                    let partition_mult = (index) as f32 / CHILDREN_FLOAT;
                    //let partition_value = partition_mult * upper_bounds[max_index] + (1.0 - partition_mult) * lower_bounds[max_index];
                    let partition_value =  (1.0 - partition_mult).mul_add(lower_bounds[max_index], partition_mult * upper_bounds[max_index]);

                    triangles.partition_point(|(centre, _)| centre[max_index] <= partition_value)
                });

                //ensure that no floating point errors happened above
                // CHILDRENPLUS1 always at least 1, so there is always a last
                *new_partition.last_mut()? = triangles.len();

                /*let points = number_of_partitions.map(|mult| {
                    mult * upper_bounds[max_index] + (1.0 - mult) * lower_bounds[max_index]
                });

                let mut new_partition = points.map(|centre_point| {
                    triangles.partition_point(|(centre, _)| centre[max_index] <= centre_point)
                });*/

                let partitions: [_; NUMBER_OF_CHILDREN] =
                    array::from_fn(|index| (new_partition[index], new_partition[index + 1]));

                let node_not_leaf = partitions.iter().all(|(lower, upper)| lower != upper);

                node_not_leaf
                    .then(|| {
                        partitions.try_map(|(lower_bound_partition, partition)| {
                            let bb_triangle = &mut triangles[lower_bound_partition..partition];

                            Self::new_from_many_inner(
                                so_far,
                                bb_triangle,
                                lower_bound_partition + start_index,
                                depth - 1,
                                init,
                            )
                        })
                    })
                    .flatten()
            };

            opt_children.map_or_else(
                || {
                    ChildrenOrMeta::Meta(Meta {
                        data: RefCell::new(init(triangles)),
                        triangle_start: start_index,
                        triangle_len: triangles.len(),
                    })
                },
                |children| ChildrenOrMeta::Children(Children { children }),
            )
        } else {
            ChildrenOrMeta::Meta(Meta {
                data: RefCell::new(init(triangles)),
                triangle_start: start_index,
                triangle_len: triangles.len(),
            })
        };

        let destination_index = so_far.len();

        so_far.push(Self {
            lower_bound: lower_bounds,
            upper_bound: upper_bounds,
            children_or_meta,
        });

        Some(destination_index)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawRecursiveError {
    SetUniform(ShaderSetUniformError),
    VertexArrayDraw(VertexArrayDrawError),
}

error_boilerplate!(DrawRecursiveError);

impl<V, T> BoundingBoxElement<V, T>
where
    V: ExtendedVertex,
    Vector<{ V::LEN }>: Volume,
{
    pub fn draw_recursive<D, P, const OUT: usize, F>(
        &self,
        bounding_box: &BoundingBox<V, T>,
        pose: &P,
        level: usize,
        active_shader: &mut ActiveShaderProgram<
            '_,
            '_,
            '_,
            VertexArray<Vector<{ V::LEN }>>,
            (),
            D::Tex,
            OUT,
        >,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        colour: &mut F,
    ) -> Result<(), DrawRecursiveError>
    where
        D: FramebufferWithDepth<OUT>,
        P: Transform<Vector<{ V::LEN }>> + Clone,
        F: FnMut() -> ColourRGBA,
    {
        if let ChildrenOrMeta::Children(children) = &self.children_or_meta
            && level != 0
        {
            let [lhs, rhs] = children.get_children_elements(bounding_box);
            lhs.draw_recursive(
                bounding_box,
                pose,
                level - 1,
                active_shader,
                active_framebuffer,
                colour,
            )?;
            rhs.draw_recursive(
                bounding_box,
                pose,
                level - 1,
                active_shader,
                active_framebuffer,
                colour,
            )?;
        } else {
            let chosen_colour = colour();

            let (lower_bounds, upper_bounds) = self.transform_by(pose);
            let positions_triangles = Volume::volume(lower_bounds, upper_bounds)
                .map(|triangle| Vector::<{ V::LEN }>::from_incomplete_triangle(triangle).unwrap());
            let mut vertex_array_builder = VertexArray::builder();
            vertex_array_builder.extend_triangles(positions_triangles);
            let vertex_array = vertex_array_builder.build();

            active_shader
                .set_uniform(&c"colour", &chosen_colour)
                .map_err(DrawRecursiveError::SetUniform)?;
            active_shader
                .set_uniform(&c"model", &Matrix::<4, 4>::identity())
                .map_err(DrawRecursiveError::SetUniform)?;

            vertex_array
                .draw(active_shader, active_framebuffer)
                .map_err(DrawRecursiveError::VertexArrayDraw)?;
        }

        Ok(())
    }
}

impl<V: ExtendedVertex, T> Model for BoundingBoxElement<V, T>
where
    Vector<{ V::LEN }>: Volume,
{
    type Vertex = Vector<{ V::LEN }>;
}
