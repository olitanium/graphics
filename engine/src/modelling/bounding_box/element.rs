use std::array;
use std::num::NonZeroUsize;

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::texture::{FlatTexture, Texture};
use graphics::vertex::Vertex;
use graphics::vertex_array::VertexArray;

use crate::EngResult;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, ExtendedVertex, Volume};

#[derive(Debug)]
pub struct BoundingBoxElement<const DIMS: usize>
where
    Vector<DIMS>: Vertex,
{
    pub(crate) vertex_array: VertexArray<Vector<DIMS>>,
    pub(crate) texture_choice: usize,

    pub(crate) lower_bound: Vector<DIMS>,
    pub(crate) upper_bound: Vector<DIMS>,

    pub(crate) lower_bound_index: usize,
    pub(crate) upper_bound_index: usize,

    pub(crate) children: Option<[NonZeroUsize; 2]>,
}

fn make_bound<V: ExtendedVertex<DIMS>, const DIMS: usize>(
    vectors: &[V],
    func: fn(f32, f32) -> f32,
) -> Option<Vector<DIMS>> {
    array::from_fn(|index| {
        vectors
            .into_iter()
            .map(|vec| vec.position().get(index).copied().unwrap_or_default())
            .reduce(func)
    })
    .transpose()
    .map(Vector::new)
}

fn make_upper_bound<V: ExtendedVertex<DIMS>, const DIMS: usize>(
    vectors: &[V],
) -> Option<Vector<DIMS>> {
    make_bound(vectors, f32::max)
}

fn make_lower_bound<V: ExtendedVertex<DIMS>, const DIMS: usize>(
    vectors: &[V],
) -> Option<Vector<DIMS>> {
    make_bound(vectors, f32::min)
}

impl<const DIMS: usize> BoundingBoxElement<DIMS>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    pub fn new_from_many_inner<V: ExtendedVertex<DIMS>>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<DIMS>, [V; 3])],
        lower_bound_index: usize,
        upper_bound_index: usize,
        depth: usize,
    ) -> Option<NonZeroUsize> {
        let vectors: Vec<_> = triangles
            .into_iter()
            .flat_map(|&mut (_, arr)| arr)
            .collect();

        let upper_bounds = make_upper_bound(&vectors)?;
        let lower_bounds = make_lower_bound(&vectors)?;

        let destination_index = so_far.len();

        let positions_triangles = Volume::volume(lower_bounds, upper_bounds)
            .map(|triangle| Vector::<DIMS>::from_incomplete_triangle(triangle).unwrap());
        let mut vertex_array_builder = VertexArray::builder();
        vertex_array_builder.extend_triangles(positions_triangles);
        let vertex_array = vertex_array_builder.build();

        so_far.push(Self {
            vertex_array,
            texture_choice: 0,
            lower_bound_index,
            upper_bound_index,
            lower_bound: lower_bounds,
            upper_bound: upper_bounds,
            children: None,
        });

        if depth != 0 {
            so_far[destination_index].children = {
                let dimensions = upper_bounds - lower_bounds;
                let (max_index, _) = dimensions
                    .into_iter()
                    .enumerate()
                    .max_by(|(_, size), (_, size2)| {
                        f32::partial_cmp(size, size2).expect(
                            "`vectors` has values, so was filled the previous step. No NAN in \
                             this vector",
                        )
                    })
                    .expect("input vector const DIMS > 0");

                // sort on the dimension given by max_index
                triangles.sort_unstable_by(|(lhs, _), (rhs, _)| {
                    f32::partial_cmp(&lhs[max_index], &rhs[max_index])
                        .expect("no NAN in input vectors permitted")
                });

                const CHILDREN: usize = 2;
                const CHILDRENPLUS1: usize = CHILDREN + 1;
                let number_of_partitions: [_; CHILDRENPLUS1] =
                    array::from_fn(|index| (index) as f32 / CHILDREN as f32);
                let points = number_of_partitions.map(|mult| {
                    mult * upper_bounds[max_index] + (1.0 - mult) * lower_bounds[max_index]
                });

                let mut new_partition = points.map(|centre_point| {
                    triangles.partition_point(|(centre, _)| centre[max_index] <= centre_point)
                });
                new_partition[CHILDRENPLUS1 - 1] = triangles.len();

                let partitions: [_; CHILDREN] =
                    array::from_fn(|index| (new_partition[index], new_partition[index + 1]));

                let node_not_leaf = partitions.iter().all(|(lower, upper)| lower != upper);

                node_not_leaf
                    .then(|| {
                        let children = partitions.try_map(|(lower_bound_partition, partition)| {
                            let bb_triangle = &mut triangles[lower_bound_partition..partition];
                            let new_lower_bound_index = lower_bound_index + lower_bound_partition;
                            let new_upper_bound_index = lower_bound_index + partition;

                            let out = Self::new_from_many_inner(
                                so_far,
                                bb_triangle,
                                new_lower_bound_index,
                                new_upper_bound_index,
                                depth - 1,
                            );
                            out
                        });

                        children
                    })
                    .flatten()
            }
        };

        Some(NonZeroUsize::new(destination_index)?)
    }

    pub fn get_triangles<'a, V: ExtendedVertex<DIMS>>(
        &self,
        bounding_box: &'a BoundingBox<V, DIMS>,
    ) -> &'a [[V; 3]] {
        &bounding_box.triangles[self.lower_bound_index..self.upper_bound_index]
    }

    pub fn draw_recursive<
        'a,
        V: ExtendedVertex<DIMS>,
        M,
        D: FramebufferWithDepth<OUT>,
        const OUT: usize,
    >(
        &self,
        bounding_box: &BoundingBox<V, DIMS>,
        level: usize,
        root: Pose,
        textures: &'a [FlatTexture],
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, V, M, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> EngResult<(), !> {
        if let Some([lhs, rhs]) = self.get_children(bounding_box)
            && level != 0
        {
            lhs.draw_recursive(
                bounding_box,
                level - 1,
                root,
                textures,
                active_shader,
                active_framebuffer,
            )?;
            rhs.draw_recursive(
                bounding_box,
                level - 1,
                root,
                textures,
                active_shader,
                active_framebuffer,
            )?;
            Ok(())
        } else {
            // TODO: remoe unwrap
            let texture = textures
                .get(self.texture_choice)
                .or_else(|| textures.get(0))
                .unwrap();
            active_shader.register_texture([("texture".into(), texture as &dyn Texture)]);

            Ok(())
        }
    }
}
