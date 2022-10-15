#version 140

in vec3 v_pos;

void main() {
    gl_FragColor = vec4(v_pos, 1.0);
}