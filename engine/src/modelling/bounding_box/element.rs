use std::array;
use std::cell::RefCell;
use std::marker::PhantomData;

use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::ShaderSetUniformError;
use graphics::vertex::Vertex;
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;

use crate::modelling::bounding_box::active_element::InnerChildrenOrMeta;
use crate::modelling::{BoundingBox, ExtendedVertex, Volume};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub(crate) enum ChildrenOrMeta<T> {
    Meta(RefCell<T>),
    Children([usize; 2]),
}

#[derive(Debug, Clone)]
pub struct BoundingBoxElement<V: ExtendedVertex<DIMS>, const DIMS: usize, T> {
    pub(crate) lower_bound: Vector<DIMS>,
    pub(crate) upper_bound: Vector<DIMS>,

    pub(crate) triangles: PhantomData<Vec<(Vector<DIMS>, [V; 3])>>,
    pub(crate) start_index: usize,
    pub(crate) len: usize,

    pub(crate) children_or_meta: ChildrenOrMeta<T>,
}

fn make_bound<V: ExtendedVertex<DIMS>, const DIMS: usize>(
    vectors: &[(Vector<DIMS>, [V; 3])],
) -> Option<(Vector<DIMS>, Vector<DIMS>)> {
    let iter = vectors
        .iter()
        .flat_map(|(_, vec)| vec.iter())
        .map(|vert| vert.position());

    let lower_bound = iter.clone().reduce(|array1, array2| {
        Vector::new(array::from_fn(|index| {
            f32::min(array1[index], array2[index])
        }))
    })?;

    let upper_bound = iter.clone().reduce(|array1, array2| {
        Vector::new(array::from_fn(|index| {
            f32::max(array1[index], array2[index])
        }))
    })?;

    assert!(core::iter::zip(upper_bound, lower_bound).all(|(x, y)| x >= y));

    Some((lower_bound, upper_bound))
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T> {
    pub fn new_from_many_inner<F: FnMut(&[(Vector<DIMS>, [V; 3])]) -> T>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<DIMS>, [V; 3])],
        start_index: usize,
        depth: usize,
        init: &mut F,
    ) -> Option<usize> {
        let (lower_bounds, upper_bounds) = make_bound(triangles)?;

        let children_or_meta = if depth != 0 {
            // so_far[destination_index].children =
            let opt_children = {
                let dimensions = upper_bounds - lower_bounds;
                let (max_index, _) = dimensions
                    .into_iter()
                    .enumerate()
                    .max_by(|(_, size), (_, size2)| f32::total_cmp(size, size2))
                    .expect("input vector const DIMS > 0");

                // sort on the dimension given by max_index
                triangles.sort_unstable_by(|(lhs, _), (rhs, _)| {
                    f32::total_cmp(&lhs[max_index], &rhs[max_index])
                });

                type const CHILDREN: usize = 2;
                type const CHILDRENPLUS1: usize = const { CHILDREN + 1 };

                let mut new_partition: [_; CHILDRENPLUS1] = array::from_fn(|index| {
                    let partition_mult = (index) as f32 / CHILDREN as f32;
                    let partition_value = partition_mult * upper_bounds[max_index] + (1.0 - partition_mult) * lower_bounds[max_index];
                    
                    triangles.partition_point(|(centre, _)| centre[max_index] <= partition_value)
                });

                //ensure that no floating point errors happened above
                *new_partition.last_mut().unwrap() = triangles.len();

                /*let points = number_of_partitions.map(|mult| {
                    mult * upper_bounds[max_index] + (1.0 - mult) * lower_bounds[max_index]
                });

                let mut new_partition = points.map(|centre_point| {
                    triangles.partition_point(|(centre, _)| centre[max_index] <= centre_point)
                });*/

                let partitions: [_; CHILDREN] =
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

            match opt_children {
                Some(children) => ChildrenOrMeta::Children(children),
                None => ChildrenOrMeta::Meta(RefCell::new(init(triangles))),
            }
        } else {
            ChildrenOrMeta::Meta(RefCell::new(init(triangles)))
        };

        let destination_index = so_far.len();

        so_far.push(Self {
            triangles: PhantomData,
            start_index,
            len: triangles.len(),
            lower_bound: lower_bounds,
            upper_bound: upper_bounds,
            children_or_meta,
        });

        Some(destination_index)
    }

    pub fn get_triangles<'a>(
        &self,
        all_triangles: &'a [(Vector<DIMS>, [V; 3])],
    ) -> &'a [(Vector<DIMS>, [V; 3])]
    where
        V: PartialEq,
    {
        &all_triangles[self.start_index..(self.start_index + self.len)]
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawRecursiveError<'a> {
    SetUniform(ShaderSetUniformError<'a>),
    VertexArrayDraw(VertexArrayDrawError<'a>),
}

error_boilerplate!(<'a> DrawRecursiveError<'a>);

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
{
    pub fn draw_recursive<
        'a,
        D: FramebufferWithDepth<OUT>,
        P: Transform<Vector<DIMS>>,
        const OUT: usize,
        F: FnMut() -> ColourRGBA,
    >(
        &self,
        bounding_box: &BoundingBox<V, DIMS, T>,
        pose: &P,
        level: usize,
        active_shader: &mut ActiveShaderProgram<
            'a,
            '_,
            '_,
            VertexArray<Vector<DIMS>>,
            (),
            D::Tex,
            OUT,
        >,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        colour: &mut F,
    ) -> Result<(), DrawRecursiveError<'a>> {
        if let InnerChildrenOrMeta::Children([lhs, rhs]) = self.get_children(bounding_box)
            && level != 0
        {
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
            Ok(())
        } else {
            let chosen_colour = colour();

            let (lower_bounds, upper_bounds) = self.transform_by(pose);
            let positions_triangles = Volume::volume(lower_bounds, upper_bounds)
                .map(|triangle| Vector::<DIMS>::from_incomplete_triangle(triangle).unwrap());
            let mut vertex_array_builder = VertexArray::builder();
            vertex_array_builder.extend_triangles(positions_triangles);
            let vertex_array = vertex_array_builder.build();

            active_shader
                .set_uniform("colour", &chosen_colour)
                .map_err(DrawRecursiveError::SetUniform)?;
            active_shader
                .set_uniform("model", &Matrix::<4, 4>::identity())
                .map_err(DrawRecursiveError::SetUniform)?;

            vertex_array
                .draw(active_shader, active_framebuffer)
                .map_err(DrawRecursiveError::VertexArrayDraw)?;

            Ok(())
        }
    }
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> Model for BoundingBoxElement<V, DIMS, T>
where
    Vector<DIMS>: Volume<DIMS>,
{
    type Vertex = Vector<DIMS>;
}
