use std::iter;

use graphics::linear_algebra::Vector;
use graphics::types::ElementArrayElem;
use graphics::vertex::IncompleteVertex;
use graphics::vertex_array::VertexArray;

use crate::modelling::SimpleVertex;
use crate::modelling::quad::QuadVertex;

pub fn vertex_array_cube(side_length: f32) -> VertexArray<SimpleVertex> {
    let vertex_positions = [
        // x    y    z
        [0.0, 0.0, 0.0], // 0
        [0.0, 0.0, 1.0], // 1
        [0.0, 1.0, 0.0], // 2
        [0.0, 1.0, 1.0], // 3
        [1.0, 0.0, 0.0], // 4
        [1.0, 0.0, 1.0], // 5
        [1.0, 1.0, 0.0], // 6
        [1.0, 1.0, 1.0], // 7
    ]
    .map(|vertex| vertex.map(|elem| (elem - 0.5) * side_length))
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
    .flat_map(|(position_index, [ta, tb, tc, td])| {
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

    let mut vertex_array_builder = VertexArray::builder();

    for incomplete_triangle in triangles {
        vertex_array_builder.push_incomplete_triangle(&incomplete_triangle);
    }

    vertex_array_builder.build()
}

pub fn vertex_array_quad(
    lower_x: f32,
    upper_x: f32,
    lower_y: f32,
    upper_y: f32,
) -> VertexArray<QuadVertex> {
    let quad_vertices = [
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
            position: Vector::new([lower_x, upper_y]),
            tex_coordinate: Vector::new([0.0, 1.0]),
        },
    ];

    let element_array = [0, 1, 2, 0, 2, 3].map(ElementArrayElem::new);

    VertexArray::builder()
        .vertices(quad_vertices)
        .element_array(element_array)
        .build()

    // let vertex_buffer = VertexBuffer::new(&quad_vertices);
    // let element_array_buffer = ElementArrayBuffer::new(&element_array);
    // VertexArray::new(vertex_buffer, /*&OFFSETS,*/ element_array_buffer)
}
