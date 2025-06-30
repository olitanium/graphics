#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;

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

uniform mat4 model;
uniform mat4 projtimesview;
uniform vec4 camera_position;

uniform PointLight lamp;
uniform FarLight sun;
uniform SpotLight torch;

out PointLight out_lamp;
out FarLight out_sun;
out SpotLight out_torch;

out vec3 tangent_view_direction;
out vec2 texture_coord;


void main() {
    texture_coord = in_texture_coord;
    
    // Normal matrix adjusts normals after non-uniform transformation
    mat3 normal_matrix = mat3(transpose(inverse(model))); // replace with CPU-side calc
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
    out_lamp = lamp;
    out_lamp.position = to_tangent * lamp.position; 

    out_sun = sun;
    out_sun.direction = mat3(to_tangent) * sun.direction;

    out_torch = torch;
    out_torch.position = to_tangent * torch.position;
    out_torch.direction = mat3(to_tangent) * torch.direction;

        // important vectors
    vec4 tangent_camera_pos = to_tangent * camera_position;
    tangent_view_direction = normalize(vec3(tangent_camera_pos));
}