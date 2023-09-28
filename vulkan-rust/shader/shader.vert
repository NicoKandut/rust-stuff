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

layout(location = 0) in vec4 inPosMat;
layout(location = 1) in vec3 inNormal;

layout(location = 0) out float fragMaterial;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out float fragDistance;

void main() {
    vec3 pos = inPosMat.xyz;
    float mat = inPosMat.w;
    // TODO: need wind_affected property
    // vec3 disposition = (1 - inColor + vec4(37/255, 95/255, 36/255, 255/255)).xyz * vec3(sin(51 * pcs.time), cos(37 * pcs.time), 0) * 0.1;
    vec4 world_pos = pcs.model * vec4(pos, 1.0);
    gl_Position = ubo.proj * ubo.view * pcs.model * vec4(pos, 1.0);

    fragDistance = length(world_pos.xyz - ubo.player.xyz);
    fragMaterial = mat;
    fragNormal = inNormal;
}