#version 450

layout(set = 0, binding = 0) uniform UniformBuffer {
    vec2 screen_size;
};

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in uint in_color;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;

vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, cutoff);
}

void main() {
    gl_Position = vec4(2.0 * in_pos.x / screen_size.x - 1.0, 1.0 - 2.0 * in_pos.y / screen_size.y, 0.0, 1.0);
    vec4 color = vec4(in_color & 0xFFu, (in_color >> 8) & 0xFFu, (in_color >> 16) & 0xFFu, (in_color >> 24) & 0xFFu);
    out_color = vec4(linear_from_srgb(color.rgb), color.a / 255.0);
    out_uv = in_uv;
}