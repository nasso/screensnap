#version 140

in vec2 pos;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = pos;

    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}
