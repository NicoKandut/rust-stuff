#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec4 player;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
    float time;
} pcs;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out float fragDistance;

void main() {
    // TODO: need wind_affected property
    // vec3 disposition = (1 - inColor + vec4(37/255, 95/255, 36/255, 255/255)).xyz * vec3(sin(51 * pcs.time), cos(37 * pcs.time), 0) * 0.1;
    vec4 world_pos = pcs.model * vec4(inPosition, 1.0);
    gl_Position = ubo.proj * ubo.view * pcs.model * vec4(inPosition, 1.0);

    fragDistance = length(world_pos.xyz - ubo.player.xyz);
    fragColor = inColor;
    fragNormal = inNormal;
}