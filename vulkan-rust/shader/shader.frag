#version 450

layout(push_constant) uniform PushConstants {
    layout(offset = 64) float time;
} pcs;

layout(binding = 1) uniform sampler2D palette;

layout(location = 0) in float fragMaterial;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in float fragDistance;

layout(location = 0) out vec4 outColor;

void main() {
    // MATERIAL
    vec4 material_color = textureLod(palette, vec2(fragMaterial + 0.5, 0.5), 0);
    
    // ILLUMINATION
    vec3 light = normalize(vec3(cos(pcs.time), sin(pcs.time), 2.0));
    float illumination = max(dot(fragNormal, light), 0.0);
    float ambient = 0.2;
    vec4 illuminated_color = vec4(material_color.xyz * min(illumination + ambient, 1.0), material_color.w);
    
    // DISTANCE FOG
    vec4 fog_background = vec4(0.5,0.5,0.5,1);
    vec4 fog_weight = vec4(1,2,4,1);
    vec4 lambda = exp(-0.00005 * fog_weight * fragDistance);

    outColor = lambda * illuminated_color + (1-lambda) * fog_background;
    // outColor = material_color;
    // outColor = illuminated_color;
    // outColor = vec4(fragNormal, 1.0);
}