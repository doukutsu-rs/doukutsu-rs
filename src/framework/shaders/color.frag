#ifdef GLES
precision mediump float;
#endif

in highp vec2 Frag_UV;
in highp vec4 Frag_Color;
out vec4 outColor;

void main()
{
    outColor = Frag_Color;
}
