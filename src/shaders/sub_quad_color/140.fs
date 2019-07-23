#version 140

uniform vec4 bounds;
uniform vec4 color;

out vec4 f_color;

void main() {
    f_color = color;
}
