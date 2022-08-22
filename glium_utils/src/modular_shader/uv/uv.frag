#version 120
uniform vec2 res;
uniform vec2 scale;
uniform bool centered;

void main() {
    vec2 relative = gl_FragCoord.xy/res;
    
    if(centered){
        relative = relative*2.0-1.0;
    }

    gl_FragData[0] = vec4(scale*relative, 0, 1);
}