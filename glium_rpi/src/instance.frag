#version 120

varying vec3 v_color;
const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

void main() {
    vec3 color = v_color;
    gl_FragColor  = vec4(color, 1.0);
}