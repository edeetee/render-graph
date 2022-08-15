#version 120
uniform vec2 res;

void main() {
    gl_FragData[0] = vec4(gl_FragCoord.xy/res, 0, 1);
}