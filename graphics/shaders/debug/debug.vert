#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;


uniform mat4 model;
uniform mat4 projtimesview;
uniform vec4 camera_position;

out vec2 texture_coord;

#define ONE 1

struct Normal {
    vec3 m_normal;
};

out Normal v_normal[ONE];

out vec3 tangent;


void main() {
    texture_coord = in_texture_coord;

    // Normal matrix adjusts normals after non-uniform transformation
    mat3 normal_matrix = mat3(transpose(inverse(model))); // replace with CPU-side calc

    v_normal[0].m_normal = (projtimesview * model * vec4(in_position, 1.0)).xyz;
    tangent = mat3(model) * in_tangent;

    // Vertex position in screen space
    gl_Position = projtimesview * model * vec4(in_position, 1.0);
}