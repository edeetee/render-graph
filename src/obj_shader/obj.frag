#version 140

in vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}