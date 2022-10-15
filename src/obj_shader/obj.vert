#version 140

uniform mat4 proj_matrix;
uniform mat4 view_matrix;

in vec3 position;

out vec3 v_pos;

void main() {
    v_pos = position;
    gl_Position = proj_matrix * view_matrix * vec4(position, 1.0);
    // gl_Position = vec4(position, 1.0);
}