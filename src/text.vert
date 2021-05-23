#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform mat4 projection;
uniform mat4 camera;

void main()
{
    vec3 newPosition = (camera * vec4(Position, 1.0)).xyz;
    gl_Position = projection * vec4(newPosition, 1.0);
    TexCoord = aTexCoord;
}
