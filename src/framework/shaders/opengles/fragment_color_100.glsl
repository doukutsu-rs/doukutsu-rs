#version 100

precision mediump float;

varying vec2 Frag_UV;
varying vec4 Frag_Color;

void main()
{
    gl_FragColor = Frag_Color;
}
