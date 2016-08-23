#version 150
in vec2 Texcoord;
out vec4 oColor;

uniform sampler2D tex;

void main()
{
  //oColor = vec4(Texcoord.y, 0.0, 0.0, 1.0);
  oColor = texture(tex, Texcoord);
  //oColor = texture(tex, Texcoord) * vec4(1.0, 1.0, 1.0, 1.0);
  //oColor = vec4(1.0, 1.0, 0, 0);
}
