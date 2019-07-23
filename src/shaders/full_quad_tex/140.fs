#version 140

uniform sampler2D tex;
uniform float opacity;

in vec2 v_tex_coords;

out vec4 f_color;

void main() {
    f_color = vec4(texture(tex, v_tex_coords).rgb, opacity);
}
