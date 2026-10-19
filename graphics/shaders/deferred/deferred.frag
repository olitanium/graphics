#version 330 core

layout (location = 0) out vec4 albedo;
layout (location = 1) out vec4 position;
layout (location = 2) out vec4 normal;

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
in vec3 out_position; 
in mat3 tbn;

void main() {
    normal = vec4(tbn * normalize(texture(material.normal_map, out_texture_coord).rgb * 2.0 - 1.0), 1.0);
    position = vec4(out_position, 1.0);
    albedo = vec4(texture(material.diffuse, out_texture_coord).xyz, 1.0);
}