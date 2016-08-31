#version 150
in vec2 Texcoord;
out vec4 oColor;

uniform sampler2DRect tex;

void main()
{
  vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
  vec4 texture_sample = texture(tex, Texcoord);
  //oColor = texture_sample;

  //oColor = texture_sample;
  // Blend with a white background because the png sometimes has no alpha
  // and in those cases need white.
  oColor = mix(white, texture_sample, texture_sample.w);
}
