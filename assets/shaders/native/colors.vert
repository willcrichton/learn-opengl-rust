#version 330 core

struct Camera {  
  vec3 view_pos;
  mat4 view;
  mat4 projection;
};

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoords;

uniform mat4 model;
uniform Camera camera;

out vec3 Normal;
out vec3 FragPos;
out vec2 TexCoords;

void main()
{
  Normal = mat3(transpose(inverse(model))) * aNormal;
  FragPos = vec3(model * vec4(aPos, 1.0));
  TexCoords = aTexCoords;
  gl_Position = camera.projection * camera.view * model * vec4(aPos, 1.0);
}