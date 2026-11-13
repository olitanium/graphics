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

const float RADIUS = 1.0;

void main() {  
    vec3 tangent_dir = texture(random_tangent, gl_FragCoord.xy / kernel_size).xyz;
    vec3 tangent = normalize(tangent_dir - dot(normal, tangent_dir) * normal);
    vec3 bitangent = cross(normal, tangent);

    // from tangent space to world space.
    mat3 tbn = mat3(tangent, bitangent, normal);

    vec4 position_clip_space =  look_at * vec4(position, 1.0);
    position_clip_space /= position_clip_space.w;

    float total = 0.0;
    vec3 sample_in_world_space_total = vec3(0.0);
    vec3 sample_in_clip_space_total = vec3(0.0);

    for (uint i = 0u; i < sample_vecs_len; ++i) {
        vec3 sample = sample_vecs[i];
        //vec3 sample = vec3(0.0, 0.0, 1.0);

        vec3 sample_in_world_space = position + RADIUS * (tbn * sample);

        sample_in_world_space_total += sample_in_world_space / sample_vecs_len;

        /*if (sample_in_world_space != position) {
            ambient_occlusion = vec4(1.0, 0.0, 0.0, 1.0);
            return;
        }*/

        vec4 sample_in_clip_space = look_at * vec4(sample_in_world_space, 1.0);
        sample_in_clip_space /= sample_in_clip_space.w;
        sample_in_clip_space = sample_in_clip_space * 0.5 + 0.5;

        sample_in_clip_space_total += sample_in_clip_space.xyz / sample_vecs_len;


        /*// DEBUG assert sample_in_clip_space for sample = vec3(0.0) is equal to texture_coord
        vec2 should_be_small = sample_in_clip_space.xy * 0.5 + 0.5 - texture_coord;
        float very_small = dot(should_be_small, should_be_small);
        if (abs(very_small) > 0.001) {
            ambient_occlusion = vec4(1.0, 0.0, 0.0, 1.0);
            return;
        }*/
        
        float screen_depth_at_sample = texture(depth_buffer, sample_in_clip_space.xy).r;
        float sample_depth = sample_in_clip_space.z;
    
        if (toggle_clip == 0) {
            total += sample_depth;
        } else if (toggle_clip == 1){
            total += screen_depth_at_sample;
        } else if (toggle_clip == 2) {
            if (sample_depth < screen_depth_at_sample + 0.01 ) {
                float range_check = smoothstep(0.0, 1.0, RADIUS / abs(sample_depth - screen_depth_at_sample));
                total += range_check;
            }
        }
    }
    total /= sample_vecs_len;


    ambient_occlusion = vec4(vec3(total), 1.0);
}