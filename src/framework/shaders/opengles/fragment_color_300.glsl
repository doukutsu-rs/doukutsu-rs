#version 300 es

precision mediump float;

in vec2 Frag_UV;
in vec4 Frag_Color;
out vec4 outColor;

void main()
{
    outColor = Frag_Color;
}
