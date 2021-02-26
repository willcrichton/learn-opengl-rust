in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

uniform DirLight dir_lights[4];
uniform int dir_lights_len;
uniform SpotLight spot_lights[16];
uniform int spot_lights_len;
uniform PointLight point_lights[16];
uniform int point_lights_len;

uniform Material material;

out vec4 FragColor;


vec4 compute_light(vec3 lightVec, vec3 light_ambient, vec3 light_diffuse, vec3 light_specular) {
  vec4 diffuse_tex = texture(material.diffuse, TexCoords);
  vec4 specular_tex = texture(material.specular, TexCoords);
    
  // Ambient
  vec4 ambient = vec4(light_ambient, 1.0) * diffuse_tex;

  // Diffuse
  vec3 norm = normalize(Normal);
  vec3 lightDir = normalize(lightVec);
  float diff = max(dot(norm, lightDir), 0.);
  vec4 diffuse = vec4(light_diffuse, 1.0) * diff * diffuse_tex;

  // Specular
  vec3 viewDir = normalize(view_pos - FragPos);
  vec3 reflectDir = reflect(-lightDir, norm);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  vec4 specular = vec4(light_specular, 1.0) * spec * specular_tex;

  return ambient + diffuse + specular;
}

vec4 compute_dir_light(DirLight light) {
  return compute_light(-light.direction, light.ambient, light.diffuse, light.specular);
}

vec4 compute_point_light(PointLight light) {
  vec3 lightVec = light.position - FragPos;

  // Attenuation
  float d = length(lightVec);
  float attenuation = 1.0 / (light.constant + light.linear * d + light.quadratic * d * d);

  return compute_light(lightVec, light.ambient, light.diffuse, light.specular) * attenuation;
}

vec4 compute_spot_light(SpotLight light) {
  vec3 lightVec = light.position - FragPos;
  vec3 lightDir = normalize(lightVec);

  // Cut off
  float theta = dot(lightDir, normalize(-light.direction));
  float epsilon = light.inner_cut_off - light.outer_cut_off;
  float intensity = clamp((theta - light.outer_cut_off) / epsilon, 0., 1.);

  if (theta > light.outer_cut_off) {

    // Attenuation
    float d = length(lightVec);
    float attenuation = 1.0 / (light.constant + light.linear * d + light.quadratic * d * d);  

    return compute_light(lightVec, light.ambient, light.diffuse, light.specular) * attenuation * intensity;
  } else {
    return vec4(0.);
  }
}

void main()
{
  vec4 result = vec4(0.);
    
  for (int i = 0; i < dir_lights_len; ++i) {
    result += compute_dir_light(dir_lights[i]);
  }

  for (int i = 0; i < point_lights_len; ++i) {
    result += compute_point_light(point_lights[i]);
  }

  for (int i = 0; i < spot_lights_len; ++i) {
    result += compute_spot_light(spot_lights[i]);
  }

  FragColor = result;
}