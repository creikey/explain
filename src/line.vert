#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 normal;

out vec2 toEdge;

uniform mat4 projection;
uniform mat4 camera;
uniform float width;

void main()
{
    vec3 newPosition = (camera * vec4(Position, 1.0)).xyz;
    toEdge = normal * width;
    newPosition.xy += toEdge;
    gl_Position = projection * vec4(newPosition, 1.0);
}
