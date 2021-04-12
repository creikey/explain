#version 330 core

out vec4 Color;
in vec2 toEdge;

uniform float width;

void main()
{
	//Color = vec4(1.0, 0.0, 0.0, 0.5);
	Color = vec4(vec3(0.0), mix(0.0, 1.0, width - length(toEdge) ));
}
