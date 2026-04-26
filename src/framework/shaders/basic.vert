
uniform mat4 ProjMtx;

in vec2 Position;
in vec2 UV;
in vec4 Color;

out highp vec2 Frag_UV;
out highp vec4 Frag_Color;

void main()
{
    Frag_UV = UV;
    Frag_Color = Color;
    gl_Position = ProjMtx * vec4(Position.xy, 0.0, 1.0);
}
