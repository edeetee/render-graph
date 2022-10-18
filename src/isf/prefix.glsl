#version 130

precision highp float;
precision highp int;

uniform int PASSINDEX = 0;
uniform vec2 res;
uniform int FRAMEINDEX = 0;
uniform float FRAMEDELTA = 0.0;
uniform float TIME = 0.0;

vec2 RENDERSIZE = res;
vec2 isf_FragNormCoord = gl_FragCoord.xy/RENDERSIZE;

#define IMG_PIXEL(sampler,coord) texture2D(sampler,coord*textureSize(sampler, 0))
#define IMG_NORM_PIXEL(sampler, coord) texture2D(sampler, coord)
#define IMG_THIS_PIXEL(sampler) IMG_THIS_NORM_PIXEL(sampler)
#define IMG_THIS_NORM_PIXEL(sampler) texture2D(sampler,isf_FragNormCoord)