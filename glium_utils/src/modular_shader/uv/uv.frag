#version 120

void main() {
    gl_FragData[0] = vec4(gl_FragCoord.xy*0.01, 0, 1);
}