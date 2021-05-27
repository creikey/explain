#version 330 core

layout (location = 0) in vec2 Position; // in world coordinates
layout (location = 1) in vec2 normal;

out vec2 toEdge;

uniform mat4 projection;
uniform float width;
uniform float camera_zoom;
uniform vec2 camera_offset;
uniform float scale;

/*
fn world_to_canvas(&self, world_coordinate: P2) -> P2 {
        (world_coordinate) / 2.0f32.powf(self.zoom) - self.offset
}
*/
vec2 world_to_canvas(vec2 world_coordinate, float zoom) {
    return ((world_coordinate) / pow(2.0, zoom)) - camera_offset;
}

/*
fn local_to_canvas(&self, object_scale: f32, local_coordinate: P2) -> P2 {
    local_coordinate * 2.0f32.powf(object_scale - self.zoom) - self.offset
}
*/

vec2 local_to_canvas(vec2 local_coordinate) {
    return (local_coordinate * pow(2.0, scale - camera_zoom)) - camera_offset;
}


void main()
{
    // convert world to canvas coordinates!

    // float powFactor = clamp(camera_zoom - scale, -10.0, 10.0); // clamps possible zooming so numbers don't go crazy
    // vec2 newPosition = camera_offset + (Position * pow(2.0, powFactor));
    // vec2 newPosition = world_to_canvas(Position);
    vec2 newPosition = local_to_canvas(Position);
    toEdge = normal * width;
    newPosition.xy += toEdge;
    gl_Position = projection * vec4(vec3(newPosition, 0.0), 1.0);
}
