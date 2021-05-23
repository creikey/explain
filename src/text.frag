#version 330 core

out vec4 Color;

in vec2 TexCoord;

uniform sampler2D ourTexture;


void main()
{
	float dist =  texture(ourTexture, TexCoord).r;
	Color = vec4(vec3(0.0), smoothstep(0.40, 0.5, dist));
}
