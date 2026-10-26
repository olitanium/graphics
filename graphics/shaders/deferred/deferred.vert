#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;

uniform mat4 model;
uniform mat4 projtimesview;

out vec2 out_texture_coord;
out vec3 out_position; 
out vec3 out_normal; 
out vec3 out_tangent; 
out mat3 tbn;

void main() {
    out_texture_coord = in_texture_coord;

    vec4 world_position = model * vec4(in_position, 1.0);
    mat3 normal_matrix = mat3(model);
    vec3 world_normal = normalize(normal_matrix * in_normal);
    vec3 world_tangent = normalize(normal_matrix * in_tangent);
    world_tangent = normalize(world_tangent - dot(world_tangent, world_normal) * world_normal);

    gl_Position = projtimesview * world_position;

    vec3 bitangent = cross(world_normal, world_tangent);

    tbn = mat3(world_tangent, bitangent, world_normal);

    out_position = world_position.xyz;
    out_normal = world_normal;
    out_tangent = world_tangent;
}