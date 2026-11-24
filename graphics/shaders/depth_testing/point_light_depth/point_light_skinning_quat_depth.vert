#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec3 in_tangent;
layout (location = 4) in vec4 in_boneindex;
layout (location = 5) in vec4 in_boneweights;

#define MAX_BONES 20
uniform mat2x4 bones_quat[MAX_BONES];
uniform mat4 projtimesview;


void main() {
    mat2x4 b = mat2x4(0.0);
    for(int i = 0; i < 4; ++i) {
        int bone_index = int(in_boneindex[i]);
        b += bones_quat[bone_index] * in_boneweights[i];
    }

    // DEFINE MODEL MATRIX;

    float inv_b0mag = 1.0 / length(b[0]);
    vec4 c0 = b[0] * inv_b0mag;
    vec4 ce = b[1] * inv_b0mag;

    float w0 = c0.w;
    float x0 = c0.x;
    float y0 = c0.y;
    float z0 = c0.z;
    float we = ce.w;
    float xe = ce.x;
    float ye = ce.y;
    float ze = ce.z;

    mat4 model = mat4(
        vec4(1.0 - 2.0*y0*y0 - 2.0*z0*z0, 2.0*x0*y0 + 2.0*w0*z0,       2.0*x0*z0 - 2.0*w0*y0,       0.0),
        vec4(2.0*x0*y0 - 2.0*w0*z0,       1.0 - 2.0*x0*x0 - 2.0*z0*z0, 2.0*y0*z0 + 2.0*w0*x0,       0.0),
        vec4(2.0*x0*z0 + 2.0*w0*y0,       2.0*y0*z0 - 2.0*w0*x0,       1.0 - 2.0*x0*x0 - 2.0*y0*y0, 0.0),
        vec4(2.0 * vec3(
            -we*x0 + xe*w0 - ye*z0 + ze*y0,
            -we*y0 + xe*z0 + ye*w0 - ze*x0,
            -we*z0 -xe*y0 + ye*x0 + ze*w0
        ), 1.0)
    );

    
    // Vertex position in screen space
    gl_Position = model * vec4(in_position, 1.0);
}