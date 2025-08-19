#version 330 core

struct Vertex {
    vec3 position;
    vec3 colour;
};

uniform Vertex vertex[100];

uniform mat4 projtimesview;

out vec3 colour;

void main() {
    gl_Position = projtimesview * vec4(vertex[gl_VertexID].position, 1.0);
    colour = vertex[gl_VertexID].colour;
}