#version 330 core

in vec2 texture_coord;

/*layout (location = 0)*/ out vec4 frag_colour;

struct Material {
    sampler2D diffuse;
    sampler2D specular_map;
    sampler2D emission;
    sampler2D normal_map; // dictates the direction, in tangent space, of the normal
    sampler2D ambient_occlusion;

    float shininess;
};

uniform Material material;

struct GenericLight {
    vec3 light_dir;
    
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct GenericOutput {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
};

struct PointLight {
    vec4 position;
  
    vec3 attenuation;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct FarLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct SpotLight {
    vec4 position;
    vec3 direction;

    vec3 attenuation;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float cos_cut_off;
    float outer_cut_off;
    float cos_outer_cut_off;
};

uniform float time;

in PointLight out_lamp;
in FarLight out_sun;
in SpotLight out_torch;

in vec3 tangent_view_direction;

GenericOutput generic_light(GenericLight);
float attenuation(vec3);
vec4 PointLight_illuminate(PointLight);
vec4 FarLight_illuminate(FarLight);
vec4 SpotLight_illuminate(SpotLight);

vec4 diffuse_map = texture(material.diffuse, texture_coord);
vec4 specular_map = texture(material.specular_map, texture_coord);
vec4 emission = texture(material.emission, texture_coord);
vec3 ambient_occlusion = texture(material.ambient_occlusion, texture_coord).rgb;
vec3 normal = normalize(texture(material.normal_map, texture_coord).rgb * 2.0 - 1.0);

void main() {
    vec4 illumination = 
        + PointLight_illuminate(out_lamp)
        + FarLight_illuminate(out_sun)
        + SpotLight_illuminate(out_torch)
    ;

    // TODO: fix alpha with an HDR buffer
    float alpha = max(diffuse_map.a, emission.a);
    vec4 colour = vec4((illumination + emission).xyz, alpha);

    if (colour.a < 0.01) {
        discard;
    }

    frag_colour = vec4(1.0, 0.0, 0.0, 1.0);//colour;

}

GenericOutput generic_light(GenericLight light) {
    // Ambient
    vec4 ambient = vec4(light.ambient, 1.0) * diffuse_map * vec4(ambient_occlusion, 1.0);
    
    // Diffuse
    float diffuse_intensity = max(dot(normal, light.light_dir), 0.0);
    vec4 diffuse = vec4(light.diffuse * diffuse_intensity, 1.0) * diffuse_map;

    // Specular
    vec3 reflect_dir = reflect(-light.light_dir, normal);
    vec3 halfway_dir = normalize(light.light_dir + normalize(tangent_view_direction));

    float cos_reflect_angle = max(dot(normal, halfway_dir), 0.0);

    vec4 specular = vec4(light.specular, 1.0)
        * pow(cos_reflect_angle, material.shininess)
        * specular_map
    ;

    return GenericOutput (
        ambient,
        diffuse,
        specular
    );
}

vec4 attenuation(vec3 factors, float light_dist) {
    float x =  1.0 / (
        factors.x
        + (factors.y * light_dist)
        + (factors.z * light_dist * light_dist)
    );

    return vec4(vec3(x), 1.0);
}

vec4 PointLight_illuminate(PointLight light) {
    vec3 frag_to_light = vec3(light.position);

    GenericLight gen_light = GenericLight(
        normalize(frag_to_light),
        light.ambient,
        light.diffuse,
        light.specular
    );


    GenericOutput gen_out = generic_light(gen_light);
    
    // Attenuation
    float light_dist = length(frag_to_light);
    vec4 light_attenuation = attenuation(light.attenuation, light_dist);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation;
}

vec4 FarLight_illuminate(FarLight light) {
    GenericLight gen_light = GenericLight(
        normalize(-light.direction),
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light);

    return gen_out.ambient + gen_out.diffuse + gen_out.specular;
}

vec4 SpotLight_illuminate(SpotLight light) {
    vec3 frag_to_light = vec3(light.position);
    vec3 light_dir = normalize(frag_to_light);
    
    float theta     = dot(light_dir, normalize(-light.direction));
    float epsilon   = light.cos_cut_off - light.cos_outer_cut_off;
    float intensity = clamp((theta - light.cos_outer_cut_off) / epsilon, 0.0, 1.0);

    GenericLight gen_light = GenericLight(
        light_dir,
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light);

    // Attenuation
    float light_dist = length(frag_to_light);
    vec4 light_attenuation = attenuation(light.attenuation, light_dist);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * intensity;
}
