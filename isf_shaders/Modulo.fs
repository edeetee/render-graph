/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "INPUTS": [
        {
            "DEFAULT": 0.25,
            "NAME": "modulo",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "NAME": "offset",
            "TYPE": "float"
        },
        {
			"NAME": "inputImage",
			"TYPE": "image"
		}
    ],
    "ISFVSN": "2"
}
*/

void main() {
	//	determine if we are on an even or odd line
	//	math goes like..
	//	mod(((coord+offset) / width),2)
	
    // out_color = vec4(1.0,0.0,0.0,0.0);
    vec4 srcPixel = IMG_THIS_PIXEL(inputImage);
	// float coord = isf_FragNormCoord[0];

	// if (vertical)	{
	// 	coord = isf_FragNormCoord[1];
	// }
	// if (width == 0.0)	{
	// 	out_color = color1;
	// }
	// else if(mod(((coord+offset) / width),2.0) < 2.0 * splitPos)	{
	// 	out_color = color1;
	// }
    
    gl_FragColor = vec4(mod(srcPixel.xyz, vec3(modulo)), srcPixel.a);
	// gl_FragColor = vec4((isf_FragNormCoord * 10), 0.0, 1.0);
}