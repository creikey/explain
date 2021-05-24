#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 normal;

out vec2 toEdge;

uniform mat4 projection;
uniform float width;
uniform float camera_zoom;
uniform vec2 camera_offset;
uniform float scale;

void main()
{
    float powFactor = clamp(camera_zoom - scale, -10.0, 10.0); // clamps possible zooming so numbers don't go crazy
    vec2 newPosition = (camera_offset + Position) * pow(2.7182, powFactor);
    toEdge = normal * width;
    newPosition.xy += toEdge;
    gl_Position = projection * vec4(vec3(newPosition, 0.0), 1.0);
}
