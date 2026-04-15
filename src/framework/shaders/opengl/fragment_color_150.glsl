#version 150 core

in vec2 Frag_UV;
in vec4 Frag_Color;
out vec4 outColor;

void main()
{
    outColor = Frag_Color;
}
