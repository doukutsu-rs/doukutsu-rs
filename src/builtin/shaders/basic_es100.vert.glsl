#version 300 es

attribute mediump vec2 a_Pos;
attribute mediump vec2 a_Uv;
attribute mediump vec4 a_VertColor;

attribute mediump vec4 a_Src;
attribute mediump vec4 a_TCol1;
attribute mediump vec4 a_TCol2;
attribute mediump vec4 a_TCol3;
attribute mediump vec4 a_TCol4;
attribute mediump vec4 a_Color;

uniform mediump mat4 u_MVP;

varying mediump vec2 v_Uv;
varying mediump vec4 v_Color;

void main() {
    v_Uv = a_Uv * a_Src.zw + a_Src.xy;
    v_Color = a_Color * a_VertColor;
    mat4 instance_transform = mat4(a_TCol1, a_TCol2, a_TCol3, a_TCol4);
    vec4 position = instance_transform * vec4(a_Pos, 0.0, 1.0);

    gl_Position = u_MVP * position;
}
