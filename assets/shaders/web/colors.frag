#version 300 es
precision highp float;

uniform vec3 objectColor;
uniform vec3 lightColor;

out vec4 FragColor;

void main()
{
  FragColor = vec4(lightColor * objectColor, 1.0);
}