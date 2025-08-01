#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;

#define MAX_LIGHTS 2

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

uniform mat4 model;
uniform mat4 projtimesview;
uniform vec4 camera_position;


uniform PointLightVarying point_vary[MAX_LIGHTS];
uniform PointLight point[MAX_LIGHTS];
uniform int num_point;

uniform FarLightVarying far_vary[MAX_LIGHTS];
uniform FarLight far[MAX_LIGHTS];
uniform int num_far;

uniform SpotLightVarying spot_vary[MAX_LIGHTS];
uniform SpotLight spot[MAX_LIGHTS];
uniform int num_spot;

out PointLightVarying out_point_vary[MAX_LIGHTS];
out FarLightVarying out_far_vary[MAX_LIGHTS];
out SpotLightVarying out_spot_vary[MAX_LIGHTS];

out vec3 tangent_view_direction;
out vec2 texture_coord;
out vec3 frag_pos_world_space;


void main() {
    texture_coord = in_texture_coord;
    
    // Normal matrix adjusts normals after non-uniform transformation
    mat3 normal_matrix = mat3(transpose(inverse(model))); // TODO: replace with CPU-side calc
    //mat3 normal_matrix = mat3(model);
    vec3 world_normal = normal_matrix * normalize(in_normal);
    vec3 world_tangent = mat3(model) * normalize(in_tangent); // unsure if this is the right correction matrix

    // Vertex Position in world space
    vec4 vertex_position = model * vec4(in_position, 1.0);

    // Vertex position in screen space
    gl_Position = projtimesview * vertex_position;

    // Vertex Position in tangent space
    // (0, 0, 0, 1)

    // World space to tangent space matrix
    vec3 T = normalize(world_tangent);
    vec3 N = normalize(world_normal);
    T = normalize(T - dot(T, N) * N);
    vec3 B = cross(N, T);
    mat3 TBN = mat3(T, B, N);
    
    mat3 rotate_to_tangent = transpose(TBN);
    
    mat4 to_tangent = mat4(rotate_to_tangent) * mat4(
        1.0, 0.0, 0.0,        0.0,
        0.0, 1.0, 0.0,        0.0,
        0.0, 0.0, 1.0,        0.0,
        -vertex_position.xyz, 1.0
    );

    // important vectors
    vec4 tangent_camera_pos = to_tangent * camera_position;
    tangent_view_direction = normalize(vec3(tangent_camera_pos));

    // Translate all vectors to tangent-space for fragment shader
    
    // lighting
    for (int x = 0; x < num_point; ++x) {
        out_point_vary[x].position = to_tangent * point_vary[x].position; 
        // SHADOW
        out_point_vary[x].frag_to_light = (model * vec4(in_position, 1.0)).xyz - point_vary[x].position.xyz;
    }

    for (int x = 0; x < num_far; ++x) {
        out_far_vary[x].direction = rotate_to_tangent * far_vary[x].direction;
        // SHADOW
        out_far_vary[x].frag_pos_light_space = far[x].matrix * vertex_position;
    }

    for (int x = 0; x < num_spot; ++x) {
        out_spot_vary[x].position = to_tangent * spot_vary[x].position;
        out_spot_vary[x].direction = rotate_to_tangent * spot_vary[x].direction;
        // SHADOW
        out_spot_vary[x].frag_pos_light_space = spot[x].matrix * vertex_position;
    }
}