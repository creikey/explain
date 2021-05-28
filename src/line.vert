#version 330 core

layout (location = 0) in vec2 Position; // in world coordinates
layout (location = 1) in vec2 normal;

out vec2 toEdge;

uniform mat4 projection;
uniform float width;
uniform vec2 offset;
uniform float scale;

void main()
{
    vec2 newPosition = scale*Position + offset;
    toEdge = normal * width;
    newPosition.xy += toEdge;
    gl_Position = projection * vec4(vec3(newPosition, 0.0), 1.0);
}
