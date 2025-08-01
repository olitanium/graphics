#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;

#define MAX_LIGHTS 2

struct PointLightVarying {
    vec4 position;
};

struct FarLightVarying {
    vec3 direction;
};

struct SpotLightVarying {
    vec4 position;
    vec3 direction;
};

uniform mat4 model;
uniform mat4 projtimesview;
uniform vec4 camera_position;


uniform PointLightVarying point_vary[MAX_LIGHTS];
uniform int num_point = 1;

uniform FarLightVarying far_vary[MAX_LIGHTS];
uniform int num_far = 1;

uniform SpotLightVarying spot_vary[MAX_LIGHTS];
uniform int num_spot = 1;

out PointLightVarying out_point_vary[MAX_LIGHTS];
out FarLightVarying out_far_vary[MAX_LIGHTS];
out SpotLightVarying out_spot_vary[MAX_LIGHTS];

out vec3 tangent_view_direction;
out vec2 texture_coord;


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

    // Translate all vectors to tangent-space for fragment shader
    
    // lighting
    for (int x = 0; x < num_point; ++x) {
        out_point_vary[x].position = to_tangent * point_vary[x].position; 
    }

    for (int x = 0; x < num_far; ++x) {
        out_far_vary[x].direction = rotate_to_tangent * far_vary[x].direction;
    }

    for (int x = 0; x < num_spot; ++x) {
        out_spot_vary[x].position = to_tangent * spot_vary[x].position;
        out_spot_vary[x].direction = rotate_to_tangent * spot_vary[x].direction;
    }

    // important vectors
    vec4 tangent_camera_pos = to_tangent * camera_position;
    tangent_view_direction = normalize(vec3(tangent_camera_pos));
}