#version 330 core
layout (location = 0) out vec4 frag_colour;
layout (location = 1) out vec4 bright_colour;

in vec3 tex_coords;

uniform samplerCube skybox;

void main()
{    
    vec4 colour = texture(skybox, tex_coords);

    if (dot(colour.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.0) {
        bright_colour = vec4(colour.rgb, 1.0);
        frag_colour = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        bright_colour = vec4(0.0, 0.0, 0.0, 1.0);
        frag_colour = vec4(colour.rgb, 1.0);
    }
}