layout (location = 0) in vec3 aPos;

out vec3 TexCoords;

uniform Camera camera;

void main()
{
  TexCoords = aPos;
  gl_Position = camera.projection * mat4(mat3(camera.view)) * vec4(aPos, 1.0);
  gl_Position = gl_Position.xyww;
}  