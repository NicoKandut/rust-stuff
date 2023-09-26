#version 450

layout(push_constant) uniform PushConstants {
    layout(offset = 64) float time;
} pcs;

layout(binding = 1) uniform sampler2D palette;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in float fragDistance;

layout(location = 0) out vec4 outColor;

void main() {
    // vec4 material_color = texture(palette, vec2(0,0));
    // outColor = material_color;

    // vec4 background = vec4(80.0 / 255.0, 120.0 / 255.0, 254.0 / 255.0, 255.0 / 255.0);
    vec4 background = vec4(0.5,0.5,0.5,1);
    vec3 light = vec3(sin(pcs.time), cos(pcs.time), -2.0);
    vec4 color = vec4(fragColor.xyz * mix(0.5, 1.0, abs(dot(fragNormal, light))), fragColor.w);
    // vec4 color = vec4(material_color.xyz * mix(0.5, 1.0, abs(dot(fragNormal, light))), fragColor.w);
    vec4 weight = vec4(1,2,4,1);
    vec4 lambda = exp(-0.0005 * weight * fragDistance);
    outColor = lambda * color + (1-lambda) * background;
}