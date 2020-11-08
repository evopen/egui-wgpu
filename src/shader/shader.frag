#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform sampler font_sampler;
layout(set = 1, binding = 0) uniform texture2D font_texture;


void main() {
    out_color = in_color * texture(sampler2D(font_texture, font_sampler), in_uv).r;
}