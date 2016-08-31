#version 150
in vec2 Texcoord;
out vec4 oColor;

uniform sampler2D tex;

void main()
{
  oColor = vec4(Texcoord, 0.5, 1.0);
}
