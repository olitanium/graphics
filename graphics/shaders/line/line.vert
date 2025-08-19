#version 330 core

uniform vec3 in_position[10];
uniform vec3 in_colour[10];

uniform mat4 projtimesview;

out vec3 colour;

void main() {
    gl_Position = projtimesview * vec4(in_position[gl_VertexID], 1.0);
    colour = in_colour[gl_VertexID];
}