#version 140

uniform sampler2D tex;
uniform float dim;

in vec2 v_tex_coords;

out vec4 f_color;

void main() {
    f_color = texture(tex, v_tex_coords) * vec4(1.0 - vec3(dim, dim, dim), 1.0);
}
