#version 120

varying vec4 v_color;
varying vec3 v_position;

void main() {
    // float dist = pow(max(0, 1-length(v_position)*2), 5);
    gl_FragData[0] = v_color;
}