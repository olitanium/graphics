#version 330 core

in vec2 texture_coord;
out vec4 frag_colour;

uniform sampler2D in_texture0;
uniform sampler2D in_texture1;
uniform sampler2D in_texture2;
uniform sampler2D in_texture3;

uniform float exposure = 2.0;

vec3 hdr_colour = texture(in_texture0, texture_coord).rgb;

void main() {  
    // Regular
    frag_colour = vec4(vec3(1.0) - exp(-hdr_colour * exposure), 1.0);
    //frag_colour = vec4(hdr_colour, 1.0);
}

