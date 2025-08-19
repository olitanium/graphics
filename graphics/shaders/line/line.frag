#version 330 core

out vec4 frag_colour;
in vec3 colour;

void main() {
    frag_colour = vec4(colour, 1.0);
}