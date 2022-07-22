#version 120

uniform vec2 res;

const float radius = 0.1;

void main() {
    vec2 coord = (gl_FragCoord.xy)/max(res.x, res.y);

    coord = mod(coord-0.5, radius*2);

    // coord -= 0.5;

    float dist = length(coord)-radius;

    float monoValue = smoothstep(0.01, 0, dist);

    gl_FragData[0] = vec4(vec3(monoValue), max(monoValue, 0));
}