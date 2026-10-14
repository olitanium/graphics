#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec3 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;

uniform mat4 model;
uniform mat4 projtimesview;

out vec2 out_texture_coord;
out vec3 out_normal;
out vec3 out_tangent;

void main() {
    texture_coord = in_texture_coord;

    vec4 world_position = model * vec4(in_position, 1.0);
    mat3 normal_matrix = mat3(model);
    vec3 world_normal = normal_matrix * vec4(in_normal, 1.0);
    vec3 world_tangent = normal_matrix * vec4(in_tangent, 1.0);

    gl_Position = projtimesview * world_position;


    normal = normalize(world_normal);
    tangent = normalize(world_tangent);
}