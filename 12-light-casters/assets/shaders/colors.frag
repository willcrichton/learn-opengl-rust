in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

uniform Material material;
uniform SpotLight global_light;
uniform Camera camera;

out vec4 FragColor;

vec3 dir_light(DirLight light, vec3 diffuse_tex, vec3 specular_tex) {
  // Ambient
  vec3 ambient = light.ambient * diffuse_tex;

  // Diffuse
  vec3 norm = normalize(Normal);
  vec3 lightDir = normalize(-light.direction);
  float diff = max(dot(norm, lightDir), 0.);
  vec3 diffuse = light.diffuse * diff * diffuse_tex;

  // Specular
  vec3 viewDir = normalize(camera.view_pos - FragPos);
  vec3 reflectDir = reflect(-lightDir, norm);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  vec3 specular = light.specular * spec * specular_tex;

  return ambient + diffuse + specular;
}

vec3 point_light(PointLight light, vec3 diffuse_tex, vec3 specular_tex) {
  vec3 lightVec = light.position - FragPos;

  // Ambient
  vec3 ambient = light.ambient * diffuse_tex;

  // Diffuse
  vec3 norm = normalize(Normal);
  vec3 lightDir = normalize(lightVec);
  float diff = max(dot(norm, lightDir), 0.);
  vec3 diffuse = light.diffuse * diff * diffuse_tex;

  // Specular
  vec3 viewDir = normalize(camera.view_pos - FragPos);
  vec3 reflectDir = reflect(-lightDir, norm);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  vec3 specular = light.specular * spec * specular_tex;

  // Attenuation
  float d = length(lightVec);
  float attenuation = 1.0 / (light.constant + light.linear * d + light.quadratic * d * d);

  return (ambient + diffuse + specular) * attenuation;
}

vec3 spot_light(SpotLight light, vec3 diffuse_tex, vec3 specular_tex) {
  vec3 lightVec = light.position - FragPos;
  vec3 lightDir = normalize(lightVec);

  float theta = dot(lightDir, normalize(-light.direction));
  float epsilon = light.inner_cut_off - light.outer_cut_off;
  float intensity = clamp((theta - light.outer_cut_off) / epsilon, 0., 1.);

  if (theta > light.outer_cut_off) {
    // Diffuse
    vec3 norm = normalize(Normal);
    float diff = max(dot(norm, lightDir), 0.);
    vec3 diffuse = light.diffuse * diff * diffuse_tex;

    // Specular
    vec3 viewDir = normalize(camera.view_pos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * spec * specular_tex;

    // Attenuation
    float d = length(lightVec);
    float attenuation = 1.0 / (light.constant + light.linear * d + light.quadratic * d * d);  

    return (diffuse + specular) * attenuation * intensity;
  } else {
    return vec3(0., 0., 0.);
  }
}

void main()
{
  vec3 diffuse_tex = vec3(texture(material.diffuse, TexCoords));
  vec3 specular_tex = vec3(texture(material.specular, TexCoords));

  //vec3 result = dir_light(diffuse_tex, specular_tex);
  //vec3 result = point_light(global_light, diffuse_tex, specular_tex);
  vec3 result = spot_light(global_light, diffuse_tex, specular_tex);
  FragColor = vec4(result, 1.0);
}