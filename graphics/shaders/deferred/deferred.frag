#version 330 core

layout (location = 0) out vec4 albedo;
layout (location = 1) out vec4 position_shini;
layout (location = 2) out vec4 normal_spec;
layout (location = 3) out vec4 emission;

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
in mat3 tbn;
in vec3 out_position; 

void main() {
    float specular = texture(material.specular_map, out_texture_coord).r;
    vec4 emission_temp = texture(material.emission, out_texture_coord);
    emission.xyz = emission_temp.xyz;
    emission.a = emission_temp.a;

    //vec3 normal = normalize(out_normal);
    //vec3 bitangent = normalize(cross(normal, out_tangent));
    //vec3 tangent = cross(bitangent, normal);
    //mat3 tbn = mat3(tangent, bitangent, normal);

    vec3 normal_map = texture(material.normal_map, out_texture_coord).rgb * 2.0 - vec3(1.0);
    normal_spec = vec4(normalize(tbn * normal_map), specular);
    position_shini = vec4(out_position, material.shininess);
    albedo = texture(material.diffuse, out_texture_coord);
}