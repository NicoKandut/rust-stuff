#version 450

layout(push_constant) uniform PushConstants {
    layout(offset = 64) vec3 light;
} pcs;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in float fragDistance;

layout(location = 0) out vec4 outColor;

void main() {
    // vec4 background = vec4(80.0 / 255.0, 120.0 / 255.0, 254.0 / 255.0, 255.0 / 255.0);
    vec4 background = vec4(0.5,0.5,0.5,1);
    vec4 color = vec4(fragColor.xyz * clamp(dot(fragNormal, pcs.light), 0.6, 1.2), 1);
    vec4 weight = vec4(1,2,4,1);
    vec4 lambda = exp(-0.0005 * weight * fragDistance);
    outColor = lambda * color + (1-lambda) * background;
    // outColor = fragColor;
}