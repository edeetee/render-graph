#version 120

uniform mat4 persp_matrix;

attribute vec3 position;
attribute vec3 instance_pos;
attribute vec4 instance_rgba;
attribute float instance_radius;

varying vec3 v_position;
varying vec4 v_color;
// varying float v_length;

void main() {
    v_position = position;
    v_color = instance_rgba;
    gl_Position = persp_matrix * vec4(position*instance_radius + instance_pos, 1.0);
}