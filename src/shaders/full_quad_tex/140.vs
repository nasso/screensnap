#version 140

in vec2 pos;

out vec2 uv;

void main() {
    uv = pos;

    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}
