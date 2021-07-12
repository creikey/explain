#version 330 core

out vec4 Color;

in vec2 TexCoord;

uniform sampler2D ourTexture;
uniform float scale;

void main()
{
	float dist = texture(ourTexture, TexCoord).r;
	float alpha = smoothstep(0.5 - (3.0/(scale*40.0)), 0.5, dist);
	// float alpha = float(dist > 0.5);
	Color = vec4(vec3(0.0), alpha);
}