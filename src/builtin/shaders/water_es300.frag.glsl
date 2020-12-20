#version 300 es

precision mediump float;

uniform sampler2D t_Texture;
in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

layout (std140) uniform Globals {
    mat4 u_MVP;
};

layout (std140) uniform WaterShaderParams {
    vec2 u_Resolution;
    vec2 u_FramePos;
    float u_Tick;
};

void main() {
    vec2 wave = v_Uv;
    wave.x += sin((-u_FramePos.y / u_Resolution.y + v_Uv.x * 16.0) + u_Tick / 20.0) * 2.0 / u_Resolution.x;
    wave.y -= cos((-u_FramePos.x / u_Resolution.x + v_Uv.y * 16.0) + u_Tick / 5.0) * 2.0 / u_Resolution.y;
    float off = 0.4 / u_Resolution.y;
    vec4 color = texture(t_Texture, wave);
    color.r = texture(t_Texture, wave + off).r;
    color.b = texture(t_Texture, wave - off).b;

    Target0 = (vec4(0.4, 0.6, 0.8, 1.0) * 0.3) + (color * v_Color * 0.7);
}

/*
precision mediump float;

uniform sampler2D t_Texture;
varying vec2 v_Uv;
varying vec4 v_Color;

uniform mat4 u_MVP;
uniform vec2 u_Resolution;
uniform vec2 u_FramePos;
uniform float u_Tick;

void main() {
    vec2 wave = v_Uv;
    wave.x += sin((-u_FramePos.y / u_Resolution.y + v_Uv.x * 16.0) + u_Tick / 20.0) * 2.0 / u_Resolution.x;
    wave.y -= cos((-u_FramePos.x / u_Resolution.x + v_Uv.y * 16.0) + u_Tick / 5.0) * 2.0 / u_Resolution.y;
    float off = 0.4 / u_Resolution.y;
    vec4 color = texture2D(t_Texture, wave);
    color.r = texture2D(t_Texture, wave + off).r;
    color.b = texture2D(t_Texture, wave - off).b;

    gl_FragColor = (vec4(0.4, 0.6, 0.8, 1.0) * 0.3) + (color * v_Color * 0.7);
}*/
