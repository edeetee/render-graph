#version 120

uniform mat4 persp_matrix;

attribute vec3 position;
attribute vec3 world_position;
varying vec3 v_position;
varying vec3 v_color;

void main() {
    v_position = position;
    v_color = vec3(1.0, 1.0, 1.0);
    gl_Position = persp_matrix * vec4(position * 0.001 + world_position, 1.0);
}