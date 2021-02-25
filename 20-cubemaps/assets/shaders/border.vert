layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform Camera camera;

out vec3 Normal;

void main()
{
  gl_Position = camera.projection * camera.view * model *  vec4(aPos * vec3(1.05), 1.0);
}