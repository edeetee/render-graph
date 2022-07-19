#version 120

uniform sampler2D feedback_texture;
uniform vec2 size;
uniform vec2 displace;
uniform float feedback_mult;

void main() {
    vec2 coord = (gl_FragCoord.xy+displace)/size;
    // vec2 coord = gl.gl_FragCoord.xy;
    vec4 feedback_color = texture2D(feedback_texture, coord);

    gl_FragData[0] = feedback_color*feedback_mult;
    // gl_FragData[0] = vec4(coord,0,1);
}