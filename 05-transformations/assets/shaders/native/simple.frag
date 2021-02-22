#version 330 core
in vec3 ourColorVarying;
in vec2 TexCoord;

uniform sampler2D texture1;
uniform sampler2D texture2;

out vec4 FragColor;

void main()
{
  FragColor = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
} 