#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

layout (std140) uniform Globals {
    mat4 u_MVP;
};

layout (std140) uniform WaterShaderParams {
    vec2 u_Resolution;
    float u_Tick;
};

void main() {
    vec2 wave = v_Uv;
    wave.x += sin(v_Uv.x * 40.0 + u_Tick / 20.0) * (sin(u_Tick / 10.0) * 0.01);
    wave.y -= cos(v_Uv.y * 20.0 + u_Tick / 5.0) * (sin(u_Tick / 20.0) * 0.01);
    float off = 0.4 / u_Resolution.y;
    vec4 color = texture(t_Texture, wave);
    color.r = texture(t_Texture, wave + off).r;
    color.b = texture(t_Texture, wave - off).b;
    Target0 = vec4(0.7, 0.8, 1.2, 1.0) * color * v_Color;
}
