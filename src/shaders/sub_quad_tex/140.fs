#version 140

uniform sampler2D tex;
uniform vec4 bounds;
uniform float opacity;

in vec2 uv;

out vec4 f_color;

void main() {
    f_color = vec4(texture(tex, uv).rgb, opacity);
}
