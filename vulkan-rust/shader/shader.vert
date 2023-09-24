#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out float fragDistance;

void main() {
    vec4 world_pos = pcs.model * vec4(inPosition, 1.0);
    gl_Position = ubo.proj * ubo.view * pcs.model * vec4(inPosition, 1.0);

    fragDistance = length(world_pos.xyz);
    fragColor = inColor;
    fragNormal = inNormal;
}