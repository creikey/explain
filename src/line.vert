#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 normal;

out vec2 toEdge;

uniform mat4 projection;
uniform mat3 camera;
uniform float width;

void main()
{
    vec2 newPosition = (camera * vec3(Position, 1.0)).xy;
    toEdge = normal * width;
    newPosition.xy += toEdge;
    gl_Position = projection * vec4(vec3(newPosition, 0.0), 1.0);
}
