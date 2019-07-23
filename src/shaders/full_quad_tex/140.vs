#version 140

in vec2 pos;
in vec2 uv;

out vec2 v_tex_coords;

void main() {
    v_tex_coords = uv;

    gl_Position = vec4(pos, 0.0, 1.0);
}
