#version 140

uniform mat4 proj_matrix;
uniform mat4 view;
uniform mat4 model;

in vec3 position;
in vec3 normal;

// out vec3 v_normal;
out vec3 v_color;

void main() {
    vec4 model_pos = model * vec4(position, 1.0);
    vec4 model_norm = model * vec4(normal, 1.0);
    v_color = model_norm.xyz;
    gl_Position = proj_matrix * view * model_pos;
    // v_normal = 
    // gl_Position = vec4(position, 1.0);
}