use std::array;
use std::cell::RefCell;
use std::num::NonZeroUsize;

use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::SetUniformError;
use graphics::vertex::Vertex;
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
use graphics::{ActiveShaderProgram, Model};

use crate::modelling::{BoundingBox, ExtendedVertex, Volume};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct BoundingBoxElement<V: ExtendedVertex<DIMS>, const DIMS: usize, T>
where
    Vector<DIMS>: ExtendedVertex<DIMS>,
{
    pub(crate) meta: RefCell<T>,

    pub(crate) lower_bound: Vector<DIMS>,
    pub(crate) upper_bound: Vector<DIMS>,

    pub(crate) triangles: Vec<[V; 3]>,

    pub(crate) children: Option<[NonZeroUsize; 2]>,
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T: Default> Default
    for BoundingBoxElement<V, DIMS, T>
where
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
    Vector<DIMS>: ExtendedVertex<DIMS>,
{
    fn default() -> Self {
        Self {
            lower_bound: -Vector::X,
            upper_bound: Vector::X,
            meta: Default::default(),
            triangles: Default::default(),
            children: Default::default(),
        }
    }
}

fn make_bound<V: ExtendedVertex<DIMS>, const DIMS: usize>(
    vectors: &[V],
    func: fn(f32, f32) -> f32,
) -> Option<Vector<DIMS>> {
    array::from_fn(|index| vectors.iter().map(|vec| vec.position()[index]).reduce(func))
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

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T>
where
    Vector<DIMS>: ExtendedVertex<DIMS>, // Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    pub fn new_from_many_inner<F: FnMut(&[(Vector<DIMS>, [V; 3])]) -> T>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<DIMS>, [V; 3])],
        depth: usize,
        init: &mut F,
    ) -> Option<usize> {
        let vectors: Vec<_> = triangles.iter_mut().flat_map(|&mut (_, arr)| arr).collect();

        let vec_triangles = triangles.iter_mut().map(|&mut (_, arr)| arr).collect();

        let upper_bounds = make_upper_bound(&vectors)?;
        let lower_bounds = make_lower_bound(&vectors)?;

        let destination_index = so_far.len();

        so_far.push(Self {
            meta: RefCell::new(init(triangles)),
            triangles: vec_triangles,
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
                        partitions.try_map(|(lower_bound_partition, partition)| {
                            let bb_triangle = &mut triangles[lower_bound_partition..partition];
                            let out =
                                Self::new_from_many_inner(so_far, bb_triangle, depth - 1, init);
                            out.and_then(NonZeroUsize::new)
                        })
                    })
                    .flatten()
            }
        };

        Some(destination_index)
    }

    pub fn get_triangles(&self) -> &[[V; 3]] {
        &self.triangles
    }
}

#[derive(Debug)]
pub enum DrawRecurisveError<'a> {
    SetUniform(SetUniformError<'a>),
    VertexArrayDraw(VertexArrayDrawError<'a>),
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    pub fn draw_recursive<
        'a,
        D: FramebufferWithDepth<OUT>,
        P: Transform<Vector<DIMS>>,
        const OUT: usize,
        F: Fn(&T) -> ColourRGBA,
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
        colour: &F,
    ) -> Result<(), DrawRecurisveError<'a>> {
        if let Some([lhs, rhs]) = self.get_children(bounding_box)
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
            let chosen_colour = colour(&*self.meta.borrow());

            let (lower_bounds, upper_bounds) = self.transform_by(pose);
            let positions_triangles = Volume::volume(lower_bounds, upper_bounds)
                .map(|triangle| Vector::<DIMS>::from_incomplete_triangle(triangle).unwrap());
            let mut vertex_array_builder = VertexArray::builder();
            vertex_array_builder.extend_triangles(positions_triangles);
            let vertex_array = vertex_array_builder.build();

            active_shader
                .set_uniform("colour", &chosen_colour)
                .map_err(|err| DrawRecurisveError::SetUniform(err))?;
            active_shader
                .set_uniform("model", &Matrix::<4, 4>::identity())
                .map_err(|err| DrawRecurisveError::SetUniform(err))?;

            vertex_array
                .draw(active_shader, active_framebuffer)
                .map_err(|err| DrawRecurisveError::VertexArrayDraw(err))?;

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
