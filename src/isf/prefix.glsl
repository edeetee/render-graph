#version 130

precision highp float;
precision highp int;

const int PASSINDEX = 0;
uniform vec2 res;
#define RENDERSIZE res;
vec2 isf_FragNormCoord = gl_FragCoord.xy/RENDERSIZE;

#define IMG_PIXEL(sampler,coord) texture2D(sampler,coord*textureSize(sampler, 0))
#define IMG_NORM_PIXEL(sampler, coord) texture2D(sampler, coord)
#define IMG_THIS_PIXEL(sampler) IMG_THIS_NORM_PIXEL(sampler)
#define IMG_THIS_NORM_PIXEL(sampler) texture2D(sampler,isf_FragNormCoord)

