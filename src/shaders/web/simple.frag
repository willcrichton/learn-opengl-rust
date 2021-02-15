#version 300 es
precision highp float;
in vec3 ourColorVarying;
uniform vec4 ourColorUniform;
out vec4 FragColor;

void main()
{
  FragColor = mix(vec4(ourColorVarying, 1.0), ourColorUniform, 0.5);
} 