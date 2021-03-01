layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
  vec3 Normal;
  vec3 FragPos;
  vec2 TexCoords;
} gs_in[];


out vec3 Normal;
out vec2 TexCoords;
out vec3 FragPos;

uniform float time;
uniform bool should_explode;

vec3 GetNormal() {
  vec3 a = vec3(gl_in[0].gl_Position) - vec3(gl_in[1].gl_Position);
  vec3 b = vec3(gl_in[2].gl_Position) - vec3(gl_in[1].gl_Position);
  return normalize(cross(a, b));
}  

vec4 explode(vec4 position, vec3 normal) {
  float magnitude = 2.0;
  vec3 direction = normal * ((sin(time) + 1.0) / 2.0) * magnitude; 
  return position + vec4(direction, 0.0);
} 

void main() {
  vec3 normal = GetNormal();
  for (int i = 0; i < 3; ++i) {
    TexCoords = gs_in[i].TexCoords;
    Normal = gs_in[i].Normal;
    FragPos = gs_in[i].FragPos;    
    gl_Position = should_explode ? explode(gl_in[i].gl_Position, normal) : gl_in[i].gl_Position;
    EmitVertex(); 
  }    
 
  EndPrimitive();
}  