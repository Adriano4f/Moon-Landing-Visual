precision highp float;
in vec3 a_position;
in vec2 a_uv;
uniform mat4 u_projection;
uniform float u_angle;
out vec3 v_position;
out vec2 v_uv;

void main()
{
	float c = cos(u_angle);
	float s = sin(u_angle);
	vec3 p = vec3(c * a_position.x + s * a_position.z, a_position.y,
		-s * a_position.x + c * a_position.z);
	v_position = p;
	v_uv = a_uv;
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, 3.3), 1.0);
}
