#ifdef GLES
precision mediump float;
#endif

uniform sampler2D Texture;

in highp vec2 Frag_UV;
in highp vec4 Frag_Color;
out vec4 outColor;

void main()
{
    outColor = Frag_Color * texture(Texture, Frag_UV.st);
}
