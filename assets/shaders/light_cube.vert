layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform Camera camera;

void main()
{
  gl_Position = camera.projection * camera.view * model * vec4(aPos, 1.0);
}