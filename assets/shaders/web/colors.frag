#version 300 es
precision highp float;

struct Camera {  
  vec3 view_pos;
  mat4 view;
  mat4 projection;
};

struct Material {
  sampler2D diffuse;
  sampler2D specular;
  float shininess;
};

struct Light {
  vec3 position;
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

uniform Material material;
uniform Light light;
uniform Camera camera;

out vec4 FragColor;

void main()
{
  vec3 diffuse_tex = vec3(texture(material.diffuse, TexCoords));
  vec3 specular_tex = vec3(texture(material.specular, TexCoords));

  // Ambient
  vec3 ambient = light.ambient * diffuse_tex;

  // Diffuse
  vec3 norm = normalize(Normal);
  vec3 lightDir = normalize(light.position - FragPos);
  float diff = max(dot(norm, lightDir), 0.);
  vec3 diffuse = light.diffuse * diff * diffuse_tex;

  // Specular
  vec3 viewDir = normalize(camera.view_pos - FragPos);
  vec3 reflectDir = reflect(-lightDir, norm);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  vec3 specular = light.specular * spec * specular_tex;

  vec3 result = ambient + diffuse + specular;
  FragColor = vec4(result, 1.0);
}