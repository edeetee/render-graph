#version 120

uniform mat4 persp_matrix;

attribute vec3 position;
attribute vec3 instance_pos;
attribute vec3 instance_color;
attribute float instance_radius;
varying vec3 v_position;
varying vec3 v_color;

void main() {
    v_position = position*instance_radius;
    v_color = instance_color;
    gl_Position = persp_matrix * vec4(position + instance_pos, 1.0);
}