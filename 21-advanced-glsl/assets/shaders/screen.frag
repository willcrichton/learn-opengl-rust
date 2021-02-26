out vec4 FragColor;
  
in vec2 TexCoords;

uniform sampler2D screenTexture;
uniform int effect;

vec4 invert() {
  vec3 pixel = vec3(texture(screenTexture, TexCoords));
  return vec4(1.0 - pixel, 1.0);
}

vec4 grayscale() {
  vec3 pixel = vec3(texture(screenTexture, TexCoords));
  float average;
  if (gl_FragCoord.x < 512.) {
    average = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
  } else {
    average = 0.33 * pixel.r + 0.33 * pixel.g + 0.33 * pixel.b;
  }
  return vec4(average, average, average, 1.0);
}

vec4 convolve(float kernel[9]) {
  float offset = 1.0 / 300.0;  

  vec2 offsets[9] = vec2[](
    vec2(-offset,  offset), // top-left
    vec2( 0.0f,    offset), // top-center
    vec2( offset,  offset), // top-right
    vec2(-offset,  0.0f),   // center-left
    vec2( 0.0f,    0.0f),   // center-center
    vec2( offset,  0.0f),   // center-right
    vec2(-offset, -offset), // bottom-left
    vec2( 0.0f,   -offset), // bottom-center
    vec2( offset, -offset)  // bottom-right    
  );

  vec3 sampleTex[9];
  for(int i = 0; i < 9; i++) {
    sampleTex[i] = vec3(texture(screenTexture, TexCoords.st + offsets[i]));
  }

  vec3 col = vec3(0.0);
  for(int i = 0; i < 9; i++) {
    col += sampleTex[i] * kernel[i];
  }
  
  return vec4(col, 1.0);
}

void main() { 
  if (effect == 0) {
    vec3 pixel = vec3(texture(screenTexture, TexCoords));
    FragColor = vec4(pixel, 1.0);
  } else if (effect == 1) {
    FragColor = invert();
  } else if (effect == 2) {
    FragColor = grayscale();
  } else if (effect == 3) {
    // Sharpen
    float kernel[9] = float[](
      -1., -1., -1.,
      -1.,  9., -1.,
      -1., -1., -1.
    );
    FragColor = convolve(kernel);
  } else if (effect == 4) {
    // Blur
    float kernel[9] = float[](
      1.0 / 16., 2.0 / 16., 1.0 / 16.,
      2.0 / 16., 4.0 / 16., 2.0 / 16.,
      1.0 / 16., 2.0 / 16., 1.0 / 16. 
    );
    FragColor = convolve(kernel);
  } else if (effect == 5) {
    // Edge detection
    float kernel[9] = float[](
      1., 1., 1.,
      1., -8., 1.,
      1., 1., 1.
    );
    FragColor = convolve(kernel);
  }
}