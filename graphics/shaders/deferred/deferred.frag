#version 330 core

layout (location = 0) out vec3 position;
layout (location = 1) out vec4 albedo;
layout (location = 2) out vec3 normal;
layout (location = 3) out vec3 tangent;

struct Material {
    sampler2D diffuse;
    sampler2D specular_map;
    sampler2D emission;
    sampler2D normal_map; // dictates the direction, in tangent space, of the normal
    sampler2D ambient_occlusion;

    float shininess;
};

uniform Material material;

in vec2 out_texture_coord;
in vec3 out_normal;
in vec3 out_tangent;

void main() {
    normal = texture(material.normal_map, out_texture_coord)
    tangent = 

}