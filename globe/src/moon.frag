precision highp float;
in vec3 v_position;
in vec2 v_uv;
uniform sampler2D u_albedo;
uniform sampler2D u_normal;
uniform float u_angle;
out vec4 out_color;

vec3 srgb_to_linear(vec3 x)
{
	return mix(x / 12.92, pow((x + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), x));
}

vec3 linear_to_srgb(vec3 x)
{
	return mix(x * 12.92, 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), x));
}

void main()
{
	vec3 n = normalize(texture(u_normal, v_uv).rgb * 2.0 - 1.0);
	float c = cos(u_angle);
	float s = sin(u_angle);
	n = normalize(vec3(c * n.x + s * n.z, n.y, -s * n.x + c * n.z));
	vec3 light = normalize(vec3(1.0, 0.2, 0.6));
	vec3 view = normalize(vec3(0.0, 0.0, 3.3) - v_position);
	float mu0 = max(dot(n, light), 0.0);
	float mu = max(dot(n, view), 0.0);
	// Constant L(alpha)=0.5 is an explicit illustrative mixture, not a fitted lunar phase law.
	float lunar_weight = 0.5;
	float reflectance = 0.0;
	if (mu0 > 0.0 && mu > 0.0)
	{
		reflectance = 2.0 * lunar_weight * mu0 / max(mu0 + mu, 0.00001)
			+ (1.0 - lunar_weight) * mu0;
	}
	vec3 albedo = srgb_to_linear(texture(u_albedo, v_uv).rgb);
	vec3 radiance = 2.0 * albedo * reflectance;
	// Reinhard tone mapping; no ambient or earthshine contribution.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
