#version 300 es
precision highp float;
in vec2 TexCoord;

uniform sampler2D texture1;
uniform sampler2D texture2;

out vec4 FragColor;

void main()
{
  FragColor = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
} 