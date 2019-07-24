#version 140

uniform vec4 bounds;

in vec2 pos;

out vec2 uv;

void main() {
    vec2 pos2d = bounds.xy + pos.xy * bounds.zw;

    uv = pos2d;

    gl_Position = vec4(pos2d * 2.0 - 1.0, 0.0, 1.0);
}
