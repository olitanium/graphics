#version 330 core

in vec2 texture_coord;
out vec4 frag_colour;

uniform uint num_point = 0u;
uniform uint num_far = 0u;
uniform uint num_spot = 0u;

uniform sampler2D in_texture0;
uniform sampler2D in_texture1;
uniform sampler2D in_texture2;
uniform sampler2D in_texture3;

vec4 albedo = texture(in_texture0, texture_coord);
vec4 position_in = texture(in_texture1, texture_coord);
vec3 position = position_in.xyz;
vec3 normal = normalize(texture(in_texture2, texture_coord).rgb * 2.0 - 1.0);

#define MAX_LIGHTS 2

void main() {  
    vec4 colour = vec4(0.0);
    frag_colour = albedo;
}

