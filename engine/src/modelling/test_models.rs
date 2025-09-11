use std::iter;
use std::sync::LazyLock;

use graphics::linear_algebra::Vector;
use graphics::types::ElementArrayElem;
use graphics::vertex::IncompleteVertex;
use graphics::vertex_array::VertexArray;

use crate::modelling::quad::QuadVertex;

pub fn vertex_array_cube(
    lowerbounds: Vector<3>,
    upperbounds: Vector<3>,
) -> impl Iterator<Item = [IncompleteVertex; 3]> {
    let [low_x, low_y, low_z] = lowerbounds.into_inner();
    let [up_x, up_y, up_z] = upperbounds.into_inner();
    let vertex_positions = [
        // x    y    z
        [low_x, low_y, low_z], // 0
        [low_x, low_y, up_z],  // 1
        [low_x, up_y, low_z],  // 2
        [low_x, up_y, up_z],   // 3
        [up_x, low_y, low_z],  // 4
        [up_x, low_y, up_z],   // 5
        [up_x, up_y, low_z],   // 6
        [up_x, up_y, up_z],    // 7
    ]
    .map(Vector::new);

    let tex_coords =
        iter::repeat([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map(Vector::new));

    let triangles = [
        [5, 4, 6, 7],
        [0, 1, 3, 2],
        [2, 3, 7, 6],
        [1, 0, 4, 5],
        [1, 5, 7, 3],
        [4, 0, 2, 6],
    ]
    .into_iter()
    .zip(tex_coords)
    .flat_map(move |(position_index, [ta, tb, tc, td])| {
        let [a, b, c, d] =
            position_index.map(|index| *vertex_positions.get(index).expect("well defined"));

        let [va, vb, vc, vd] = [
            IncompleteVertex::new(a, ta),
            IncompleteVertex::new(b, tb),
            IncompleteVertex::new(c, tc),
            IncompleteVertex::new(d, td),
        ];

        // [a, b, c], [a, c, d]
        [[va, vb, vc], [va, vc, vd]]
    });

    // TODO: Clean Up
    /*let mut vertex_array_builder = VertexArray::builder();

    for incomplete_triangle in triangles {
        vertex_array_builder.push_incomplete_triangle(incomplete_triangle);
    }

    vertex_array_builder.build()*/
    triangles
}

pub fn origin_unit_cube() -> impl Iterator<Item = [IncompleteVertex; 3]> {
    origin_cube(1.0)
}

pub fn origin_cube(side_length: f32) -> impl Iterator<Item = [IncompleteVertex; 3]> {
    let upperbounds = Vector::new([side_length / 2.0; 3]);
    let lowerbounds = Vector::new([-side_length / 2.0; 3]);

    vertex_array_cube(lowerbounds, upperbounds)
}

pub fn vertex_array_quad(
    lower_x: f32,
    upper_x: f32,
    lower_y: f32,
    upper_y: f32,
) -> VertexArray<QuadVertex> {
    let mut quad_vertices = [
        QuadVertex {
            position: Vector::new([lower_x, lower_y]),
            tex_coordinate: Vector::new([0.0, 0.0]),
        },
        QuadVertex {
            position: Vector::new([upper_x, lower_y]),
            tex_coordinate: Vector::new([1.0, 0.0]),
        },
        QuadVertex {
            position: Vector::new([upper_x, upper_y]),
            tex_coordinate: Vector::new([1.0, 1.0]),
        },
        QuadVertex {
            position: Vector::new([lower_x, lower_y]),
            tex_coordinate: Vector::new([0.0, 0.0]),
        },
        QuadVertex {
            position: Vector::new([upper_x, upper_y]),
            tex_coordinate: Vector::new([1.0, 1.0]),
        },
        QuadVertex {
            position: Vector::new([lower_x, upper_y]),
            tex_coordinate: Vector::new([0.0, 1.0]),
        },
    ];

    // TODO: FIX this 
    quad_vertices.reverse();

    let element_array = [5, 4, 3, 2, 1, 0].map(ElementArrayElem::new);

    VertexArray::builder()
        .vertices(quad_vertices.to_vec())
        .element_array(element_array.to_vec())
        .build()
}

pub fn screen_quad() -> &'static VertexArray<QuadVertex> {
    static SCREEN_QUAD: LazyLock<VertexArray<QuadVertex>> =
        LazyLock::new(|| vertex_array_quad(-1.0, 1.0, -1.0, 1.0));

    &SCREEN_QUAD
}
