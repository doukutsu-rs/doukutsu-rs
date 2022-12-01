#version 460

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 UV;
layout (location = 2) in vec4 Color;

layout (location = 0) out vec2 Frag_UV;
layout (location = 1) out vec4 Frag_Color;

layout (std140, binding = 0) uniform VertUBO
{
    mat4 proj;
} ProjMtx;

void main()
{
    Frag_UV = UV;
    Frag_Color = Color;
    gl_Position = ProjMtx.proj * vec4(Position.xy, 0.0, 1.0);
}
