#version 330 core

layout (location = 0) out vec4 frag_colour;
layout (location = 1) out vec4 tangent_colour; 

struct Material {
    sampler2D diffuse; // background requires no mask
    sampler2D specular_map; // specular map is a mask 
    sampler2D emission; // emission is a texture and so needs a mask
    sampler2D normal_map; // dictates the direction, in tangent space, of the normal
    sampler2D ambient_occlusion;

    float shininess;
};

uniform Material material;

in vec2 texture_coord;

#define ONE 1

struct Normal {
    vec3 m_normal;
};

in Normal v_normal[ONE];


in vec3 tangent;

void main() { 
    //frag_colour = texture(material.specular_map, texture_coord);
    //frag_colour = vec4(normalize(texture(material.normal_map, texture_coord).rgb * 2.0 - 1.0), 1.0);
    vec3 vec_normal = vec3(
        v_normal[0].m_normal.x,
        v_normal[0].m_normal.y,
        v_normal[0].m_normal.z
    );

    frag_colour = vec4(normalize(vec_normal), 1.0);
    //tangent_colour = vec4(normalize(tangent), 1.0);
    
    if (frag_colour.a < 0.01 ){
        discard;
    }
}