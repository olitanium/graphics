#version 330 core
layout (location = 0) in vec3 position;

uniform mat4 projtimesview;
uniform mat4 model; 

void main() {
    vec4 pos = projtimesview * model * vec4(position, 1.0);

    gl_Position = pos;
}