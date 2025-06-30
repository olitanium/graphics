#version 330 core
in vec4 position;

uniform vec3 light_position;
uniform float far_plane;

//uniform float time;

void main()
{
    // get distance between fragment and light source
    float light_distance = length(position.xyz - light_position);
    
    // map to [0;1] range by dividing by far_plane
    light_distance = light_distance / far_plane;
    
    // write this as modified depth
    gl_FragDepth = light_distance;
}