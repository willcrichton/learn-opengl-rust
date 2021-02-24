out vec4 FragColor;
  
in vec2 TexCoords;

uniform sampler2D screenTexture;
uniform int effect;

vec4 invert(vec3 pixel) {
  return vec4(1.0 - pixel, 1.0);
}

vec4 grayscale(vec3 pixel) {
  float average = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
  return vec4(average, average, average, 1.0);
}

void main() { 
  vec3 pixel = vec3(texture(screenTexture, TexCoords));
  if (effect == 0) {
    FragColor = vec4(pixel, 1.0);
  } else if (effect == 1) {
    FragColor = invert(pixel);
  } else if (effect == 2) {
    FragColor = grayscale(pixel);
  }
}