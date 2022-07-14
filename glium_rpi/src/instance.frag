#version 120

varying vec3 v_color;

void main() {
    gl_FragData[0]  = vec4(v_color, 1.0);
}