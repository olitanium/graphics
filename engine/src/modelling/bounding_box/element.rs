use std::{array, cell::Cell, rc::Rc};

use graphics::{colour::ColourRGBA, linear_algebra::Vector};

use crate::modelling::{BoundingBox, CubicVertex, advanced_vertex::{ExtendedVertex, IncompleteVertex}, cubic::{Mesh, MeshBuilder, material::Material}, test_models};

#[derive(Debug, Clone)]
pub struct BoundingBoxElement<const DIMS: usize> {
    pub(crate) collision: Cell<bool>,

    pub(crate) lower_bound: Vector<DIMS>,
    pub(crate) upper_bound: Vector<DIMS>,

    pub(crate) lower_bound_index: usize,
    pub(crate) upper_bound_index: usize,

    pub(crate) children: Option<[usize; 2]>,
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

impl<const DIMS: usize> BoundingBoxElement<DIMS> {
    pub fn new_from_many_inner<V: ExtendedVertex<DIMS>>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<DIMS>, [V; 3])],
        lower_bound_index: usize,
        upper_bound_index: usize,
        depth: usize,
    ) -> Option<usize> {
        let vectors: Vec<_> = triangles.into_iter().flat_map(|(_, arr)| *arr).collect();
        let upper_bounds = make_upper_bound(&vectors)?;
        let lower_bounds = make_lower_bound(&vectors)?;

        let destination_index = so_far.len();

        so_far.push(Self {
            collision: Cell::new(false),
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

        Some(destination_index)
    }

    pub fn get_triangles<'a, V: ExtendedVertex<DIMS>>(
        &self,
        bounding_box: &'a BoundingBox<V, DIMS>,
    ) -> &'a [[V; 3]] {
        &bounding_box.triangles[self.lower_bound_index..self.upper_bound_index]
    }

}


impl BoundingBoxElement<3> {
    fn make_cube(&self) -> impl Iterator<Item = [IncompleteVertex; 3]> {
        test_models::vertex_array_cube(self.lower_bound, self.upper_bound)
    }

    pub fn make_model<'a, V: CubicVertex>(
        &'a self,
        bounding_box: &'a BoundingBox<V, 3>,
        level: usize,
    ) -> Box<dyn Iterator<Item = MeshBuilder> + 'a> {
        if let Some([lhs, rhs]) = self.get_children(bounding_box)
            && level != 0
        {
            let iter = [
                lhs.make_model(bounding_box, level - 1),
                rhs.make_model(bounding_box, level - 1),
            ]
            .into_iter()
            .flatten();
            Box::new(iter)
            //.collect()
        } else {
            let triangles = self.make_cube();
            //TODO: Horribly inefficient
            let material = if self.collision.get() {Rc::new(Material::from_colour(ColourRGBA::RED))} else {Rc::new(Material::from_colour(ColourRGBA::BLACK))};
            let mut mesh = Mesh::builder().material(material);
            mesh.extend_triangle(triangles);
            Box::new(Some(mesh).into_iter())
        }
    }
}
