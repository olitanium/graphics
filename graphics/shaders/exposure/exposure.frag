#version 330 core

in vec2 texture_coord;
out vec4 frag_colour;

uniform sampler2D in_texture;
uniform float exposure = 1.0;

void main() {  
    // Regular
    vec3 hdr_colour = texture(in_texture, texture_coord).rgb;
    frag_colour = vec4(vec3(1.0) - exp(-hdr_colour * exposure), 1.0);
}

