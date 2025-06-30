#version 330 core 

layout (location = 0) in vec3 in_position;

uniform mat4 model;
uniform mat4 projtimesview;

void main()
{
    gl_Position = projtimesview * model * vec4(in_position, 1.0);
}  