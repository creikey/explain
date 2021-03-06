#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform mat4 projection;
uniform vec2 offset;
uniform float scale;

void main()
{
    vec2 newPosition = (vec3(scale*Position + offset, 1.0)).xy;
    gl_Position = projection * vec4(vec3(newPosition, 0.0), 1.0);
    TexCoord = aTexCoord;
}
