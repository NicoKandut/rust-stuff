#version 450

layout(push_constant) uniform PushConstants {
    layout(offset = 64) vec3 light;
} pcs;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor * clamp(dot(fragNormal, pcs.light), 0.2, 1), 1.0);
}