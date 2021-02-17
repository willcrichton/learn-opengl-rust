#version 300 es

layout (location = 0) in vec3 aPos;

struct Camera {
  mat4 view;
  mat4 projection;
};

uniform mat4 model;
uniform Camera camera;

void main()
{
  gl_Position = camera.projection * camera.view * model * vec4(aPos, 1.0);
}