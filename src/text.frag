#version 330 core

out vec4 Color;

in vec2 TexCoord;

uniform sampler2D ourTexture;


void main()
{
	//Color = vec4(0.0, 0.0, 0.0, 1.0);
	float dist =  texture(ourTexture, TexCoord).r;
	if(dist > 0.5) {
		Color = vec4(vec3(0.0), 1.0);
	} else {
		Color = vec4(0.0, 0.0, 0.0, 0.0);
	}
}
