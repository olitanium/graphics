#version 330 core

in vec2 texture_coord;

layout (location = 0) out vec4 frag_colour;
layout (location = 1) out vec4 bright_colour;

#define MAX_LIGHTS 2

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

struct PointLightVarying {
    vec4 position;
    vec3 frag_to_light;
};

struct PointLight {
    vec3 attenuation;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    samplerCube depth;
    float far_plane;
};

struct FarLightVarying {
    vec3 direction;
    vec4 frag_pos_light_space;
};

struct FarLight {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    mat4 matrix;
    sampler2D depth;
};

struct SpotLightVarying {
    vec4 position;
    vec3 direction;
    vec4 frag_pos_light_space;
};

struct SpotLight {
    vec3 attenuation;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float cos_cut_off;
    float outer_cut_off;
    float cos_outer_cut_off;
    mat4 matrix;
    sampler2D depth;
};


uniform float time;

in PointLightVarying out_point_vary[MAX_LIGHTS];
uniform PointLight point[MAX_LIGHTS];
uniform int num_point;

in FarLightVarying out_far_vary[MAX_LIGHTS];
uniform FarLight far[MAX_LIGHTS];
uniform int num_far;

in SpotLightVarying out_spot_vary[MAX_LIGHTS];
uniform SpotLight spot[MAX_LIGHTS];
uniform int num_spot;

in vec3 tangent_view_direction;
in vec3 frag_pos_world_space;

float calculate_shadow(mat4 matrix, vec4 frag_pos_light_space, sampler2D depth_map);
float calculate_shadow_point(vec3 frag_to_light, samplerCube depth, float far_plane);

GenericOutput generic_light(GenericLight);
float attenuation(vec3);
vec4 PointLight_illuminate(PointLight, PointLightVarying);
vec4 FarLight_illuminate(FarLight, FarLightVarying);
vec4 SpotLight_illuminate(SpotLight, SpotLightVarying);


vec4 diffuse_map = texture(material.diffuse, texture_coord);
vec4 specular_map = texture(material.specular_map, texture_coord);
vec4 emission = texture(material.emission, texture_coord);
vec3 ambient_occlusion = texture(material.ambient_occlusion, texture_coord).rgb;
vec3 normal = normalize(texture(material.normal_map, texture_coord).rgb * 2.0 - 1.0);

void main() {
    
    vec4 illumination = vec4(0);
    
    for (int x = 0; x < num_point; ++x) {
        illumination += PointLight_illuminate(point[x], out_point_vary[x]);
    }
    for (int x = 0; x < num_far; ++x) {
        illumination += FarLight_illuminate(far[x], out_far_vary[x]);
    }
    for (int x = 0; x < num_spot; ++x) {
        illumination += SpotLight_illuminate(spot[x], out_spot_vary[x]);
    }


    // TODO: fix alpha with an HDR buffer
    float alpha = max(diffuse_map.a, emission.a);
    vec4 colour = vec4((illumination + emission).xyz, alpha);
    
    //if (colour.a < 0.01) {
    //    discard;
    //}

    //colour = vec4(frag_pos_world_space, 1.0);

    frag_colour = colour;
    if (dot(colour.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.0) {
        bright_colour = vec4(colour.rgb, 1.0);
    } else {
        bright_colour = vec4(0.0, 0.0, 0.0, 1.0);
    }
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

float calculate_shadow(mat4 matrix, vec4 frag_pos_light_space, sampler2D depth_map) {
    vec3 projection_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    projection_coords = projection_coords * 0.5 + 0.5;

    float closest_depth = texture(depth_map, projection_coords.xy).r;
    float current_depth = projection_coords.z;

    if (current_depth > 1.0) { 
        // If too far away from light, assume in shadow 
        return 0.0; 
    }

    // NOTES: +1.0 means far away
    //         0.0 means close
    // is current depth closer to zero than the closest depth? if so return 1.0
    // 0.0 < current_depth < closest_depth ? 1.0
    return current_depth < closest_depth ? 1.0 : 0.0;
    return current_depth - 0.00005 < closest_depth ? 1.0 : 0.0;    
}

float calculate_shadow_point(vec3 frag_to_light, samplerCube depth, float far_plane) {
    float closest_depth = texture(depth, frag_to_light).r;// * far_plane;
    
    float current_depth = length(frag_to_light)/far_plane;
    
    return current_depth < closest_depth ? 1.0 : 0.0;
    return current_depth - 0.005 < closest_depth ? 1.0 : 0.0;
}

vec4 PointLight_illuminate(PointLight light, PointLightVarying light_vary) {
    vec3 frag_to_light = vec3(light_vary.position /* - frag_position */);

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

    // Shadow

    float shadow = calculate_shadow_point(light_vary.frag_to_light, light.depth, light.far_plane);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * shadow;
}

vec4 FarLight_illuminate(FarLight light, FarLightVarying light_vary) {
    GenericLight gen_light = GenericLight(
        normalize(-light_vary.direction),
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light);


    // Shadow
    float shadow_factor = calculate_shadow(light.matrix, light_vary.frag_pos_light_space, light.depth);

    return gen_out.ambient + ( gen_out.diffuse + gen_out.specular ) * shadow_factor;
}

vec4 SpotLight_illuminate(SpotLight light, SpotLightVarying light_vary) {
    vec3 frag_to_light = vec3(light_vary.position);
    vec3 light_dir = normalize(frag_to_light);
    
    float theta     = dot(light_dir, normalize(-light_vary.direction));
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

    // Shadow
    float shadow_factor = calculate_shadow(light.matrix, light_vary.frag_pos_light_space, light.depth);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * intensity * shadow_factor;
}
