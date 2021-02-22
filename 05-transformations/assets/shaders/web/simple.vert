#version 300 es
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 transform;

out vec3 ourColorVarying;
out vec2 TexCoord;

void main()
{
  gl_Position = transform * vec4(aPos, 1.0);
  ourColorVarying = aColor;
  TexCoord = aTexCoord;
}