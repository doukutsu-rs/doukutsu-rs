#version 110

uniform mat4 ProjMtx;
uniform sampler2D Texture;
uniform float Time;
uniform float Scale;
uniform vec2 FrameOffset;
varying vec4 Frag_Color;

void main()
{
    vec2 resolution_inv = vec2(ProjMtx[0][0], -ProjMtx[1][1]) * 0.5;
    vec2 uv = gl_FragCoord.xy * resolution_inv;
    uv.y += 1.0;
    vec2 wave = uv;
    wave.x += sin((-FrameOffset.y * resolution_inv.y + uv.x * 16.0) + Time / 20.0) * Scale * resolution_inv.x;
    wave.y -= cos((-FrameOffset.x * resolution_inv.x + uv.y * 16.0) + Time / 5.0) * Scale * resolution_inv.y;
    float off = 0.35 * Scale * resolution_inv.y;
    float off2 = 2.0 * off;

    vec3 color = texture2D(Texture, wave).rgb * 0.25;
    color += texture2D(Texture, wave + vec2(0, off)).rgb * 0.125;
    color += texture2D(Texture, wave + vec2(0, -off)).rgb * 0.125;

    color.rg += texture2D(Texture, wave + vec2(-off, -off)).rg * 0.0625;
    color.rg += texture2D(Texture, wave + vec2(-off, 0)).rg * 0.125;
    color.rg += texture2D(Texture, wave + vec2(-off, off)).rg * 0.0625;
    color.b += texture2D(Texture, wave + vec2(-off2, -off)).b * 0.0625;
    color.b += texture2D(Texture, wave + vec2(-off2, 0)).b * 0.125;
    color.b += texture2D(Texture, wave + vec2(-off2, off)).b * 0.0625;

    color.rg += texture2D(Texture, wave + vec2(off, off)).gb * 0.0625;
    color.rg += texture2D(Texture, wave + vec2(off, 0)).gb * 0.125;
    color.rg += texture2D(Texture, wave + vec2(off, -off)).gb * 0.0625;
    color.b += texture2D(Texture, wave + vec2(off2, off)).r * 0.0625;
    color.b += texture2D(Texture, wave + vec2(off2, 0)).r * 0.125;
    color.b += texture2D(Texture, wave + vec2(off2, -off)).r * 0.0625;

    color *= (1.0 - Frag_Color.a);
    color += Frag_Color.rgb * Frag_Color.a;
    gl_FragColor = vec4(color, 1.0);
}
