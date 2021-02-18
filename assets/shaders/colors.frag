in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

uniform Material material;
uniform PointLight light;
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