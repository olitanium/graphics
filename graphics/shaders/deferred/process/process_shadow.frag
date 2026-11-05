#version 330 core

layout(location = 0) out vec4 frag_colour;
layout(location = 1) out vec4 bright_colour;
layout(location = 2) out vec4 residual_colour;

in vec2 texture_coord;

#define MAX_LIGHTS 2

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

    samplerCube depth;
    float far_plane;
};

struct FarLight {
    vec3 direction;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    //vec4 frag_pos_light_space; // calculated here
    mat4 matrix;
    sampler2D depth;
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

    //vec4 frag_pos_light_space; // calculated here
    mat4 matrix;
    sampler2D depth;
};

uniform PointLight point[MAX_LIGHTS];
uniform uint num_point = 0u;

uniform FarLight far[MAX_LIGHTS];
uniform uint num_far = 0u;

uniform SpotLight spot[MAX_LIGHTS];
uniform uint num_spot = 0u;

uniform sampler2D in_texture0;
uniform sampler2D in_texture1;
uniform sampler2D in_texture2;
uniform sampler2D in_texture3;

uniform vec3 camera_position;

float calculate_shadow(mat4 matrix, vec4 frag_pos_light_space, sampler2D depth_map);
float calculate_shadow_point(vec3 frag_to_light, samplerCube depth, float far_plane);

GenericOutput generic_light(GenericLight);
float attenuation(vec3);
vec4 PointLight_illuminate(PointLight);
vec4 FarLight_illuminate(FarLight);
vec4 SpotLight_illuminate(SpotLight);

vec4 albedo = texture(in_texture0, texture_coord);
vec4 position_shini = texture(in_texture1, texture_coord);
vec3 position = position_shini.xyz;
float shininess = position_shini.a;
vec4 normal_spec = texture(in_texture2, texture_coord);
vec3 normal = normal_spec.xyz;
float spec = normal_spec.a;
vec4 emission = texture(in_texture3, texture_coord);

vec3 view_dir = normalize(camera_position - position);

void main() {  
    vec4 illumination = vec4(0.0);
    
    for (uint x = 0u; x < num_point; ++x) {
        illumination += PointLight_illuminate(point[x]);
    }
    for (uint x = 0u; x < num_far; ++x) {
        illumination += FarLight_illuminate(far[x]);
    }
    for (uint x = 0u; x < num_spot; ++x) {
        illumination += SpotLight_illuminate(spot[x]);
    }

    vec4 colour = emission + illumination;
    
    if (colour.a < 0.01) {
        discard;
    }

    frag_colour = colour;
    if (dot(colour.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.0) {
        bright_colour = vec4(colour.rgb, 1.0);
        residual_colour = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        residual_colour = vec4(colour.rgb, 1.0);
        bright_colour = vec4(0.0, 0.0, 0.0, 1.0);
    }

}


GenericOutput generic_light(GenericLight light) {
    // global normal,
    // global albedo,
    // global shininess,
    // global spec,
     
    // Ambient
    vec4 ambient = vec4(light.ambient, 1.0) * albedo;// * vec4(ambient_occlusion, 1.0);
    
    // Diffuse
    float diffuse_intensity = max(dot(normal, light.light_dir), 0.0);
    vec4 diffuse = vec4(light.diffuse * diffuse_intensity, 1.0) * albedo;

    // Specular
    //vec3 reflect_dir = reflect(-light.light_dir, normal);
    vec3 halfway_dir = normalize(light.light_dir + view_dir);

    float cos_reflect_angle = max(dot(normal, halfway_dir), 0.0);

    vec4 specular = vec4(light.specular, 1.0)
        * pow(cos_reflect_angle, shininess)
        * spec
    ;

    return GenericOutput (
        ambient,
        diffuse,
        specular
    );
}

vec4 attenuation(vec3 factors, float light_dist) {
    if (factors == vec3(0.0) || light_dist == 0.0) {
        return vec4(0.0);
    }
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
}

vec4 PointLight_illuminate(PointLight light) {
    // global position
    vec3 frag_to_light = vec3(light.position) - position;

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
    float shadow = calculate_shadow_point(-frag_to_light, light.depth, light.far_plane);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * shadow;
}

vec4 FarLight_illuminate(FarLight light) {
    GenericLight gen_light = GenericLight(
        normalize(-light.direction),
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light);

    // Shadow
    vec4 frag_pos_light_space = light.matrix * vec4(position, 1.0);
    float shadow = calculate_shadow(light.matrix, frag_pos_light_space, light.depth);

    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * shadow;
}

vec4 SpotLight_illuminate(SpotLight light) {
    // global position

    vec3 frag_to_light = vec3(light.position) - position;
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

    // Shadow
    vec4 frag_pos_light_space = light.matrix * vec4(position, 1.0);
    float shadow_factor = calculate_shadow(light.matrix, frag_pos_light_space, light.depth);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * intensity * shadow_factor;
}
