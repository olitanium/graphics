#version 330 core

layout(location = 0) out vec4 ambient_occlusion;

in vec2 texture_coord;

uniform sampler2D in_texture0;
uniform sampler2D in_texture1;
uniform sampler2D in_texture2;
uniform sampler2D in_texture3;

vec4 albedo = texture(in_texture0, texture_coord);
vec4 position_shini = texture(in_texture1, texture_coord);
vec3 position = position_shini.xyz;
float shininess = position_shini.a;
vec4 normal_spec = texture(in_texture2, texture_coord);
vec3 normal = normal_spec.xyz;
float spec = normal_spec.a;
vec4 emission = texture(in_texture3, texture_coord);

uniform mat4 look_at;
uniform vec3 sample_vecs[128];
uniform uint sample_vecs_len;
uniform uint kernel_size;
uniform sampler2D random_tangent;
uniform uint frames = 0u;
uniform sampler2D depth_buffer;

uniform int toggle_clip;

float radius = 0.05;

void main() {  
    if (position == vec3(0.0)) {
        discard;
    }
    vec3 tangent_dir = texture(random_tangent, gl_FragCoord.xy / kernel_size).xyz;
    vec3 tangent = normalize(tangent_dir - dot(normal, tangent_dir) * normal);
    vec3 bitangent = cross(normal, tangent);

    // from tangent space to world space.
    mat3 tbn = mat3(tangent, bitangent, normal);

    //vec4 position_clip_space =  look_at * vec4(position, 1.0);
    //position_clip_space /= position_clip_space.w;

    float occlusion = 0.0;

    vec3 total = vec3(0.0);
    //vec3 out_of_bounds = vec3(0.0);
    //vec3 sample_in_world_space_total = vec3(0.0);
    //vec3 sample_in_clip_space_total = vec3(0.0);
    //float sample_in_clip_space_behind = 0.0;

    for (uint i = 0u; i < sample_vecs_len; ++i) {
        vec3 sample = sample_vecs[i];

        vec3 sample_in_world_space = position + radius * (tbn * sample);

        //sample_in_world_space_total += sample_in_world_space / sample_vecs_len;

        vec4 sample_in_clip_space = look_at * vec4(sample_in_world_space, 1.0);
        sample_in_clip_space /= sample_in_clip_space.w;
        // [-1.0, 1.0] => [0.0, 1.0] for all x, y, z.
        // z = 1.0 means far away.
        sample_in_clip_space.xyz = sample_in_clip_space.xyz * 0.5 + 0.5;


        //sample_in_clip_space_total += sample_in_clip_space.xyz / sample_vecs_len;

        float screen_depth_at_sample;
        if (sample_in_clip_space.x < 0.0 || sample_in_clip_space.x > 1.0 || sample_in_clip_space.y < 0.0 || sample_in_clip_space.y > 1.0) {
            screen_depth_at_sample = 1.0;
        } else  {
            screen_depth_at_sample = texture(depth_buffer, sample_in_clip_space.xy).r;
            if (screen_depth_at_sample == 0.0) {
                screen_depth_at_sample = 1.0;
            }
        }

        float sample_depth = sample_in_clip_space.z;
    
        total += sample_depth;
    
        if (sample_depth <= screen_depth_at_sample + 0.01) {
            //float range_check = smoothstep(1.0, 0.0, (screen_depth_at_sample - sample_depth) / radius);
            //occlusion += range_check;
            occlusion += 1.0;
        }
    }
    
    ambient_occlusion = vec4(vec3(occlusion / sample_vecs_len), 1.0);
}