#version 100

uniform mediump sampler2D t_Texture;
varying mediump vec2 v_Uv;
varying mediump vec4 v_Color;

//uniform mediump mat4 u_MVP;

mediump vec4 Target0;

void main() {
    gl_FragColor = texture2D(t_Texture, v_Uv) * v_Color;
}
