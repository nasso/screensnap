#version 140

uniform vec4 bounds;

in vec2 pos;

void main() {
    gl_Position = vec4((bounds.xy + (pos.xy) * bounds.zw) * 2.0 - 1.0, 0.0, 1.0);
}
