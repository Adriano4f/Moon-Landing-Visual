precision highp float;
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Signed cross-strip or circular coordinates for analytic ray coverage.
in vec2 a_coverage;
out vec2 v_coverage;
// Receive the column-major view-to-clip perspective matrix from Rust.
uniform mat4 u_projection;
// Receive camera rotation times Moon-fixed-to-J2000 rotation, shared with lights and CPU overlays.
uniform mat3 u_model;
uniform highp float u_camera_distance;
// Explicit integer precision must match the fragment stage for WebGL linking.
uniform highp int u_ray_mode;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
// Keep default smooth interpolation: derivatives of coverage-dependent centroid values worsened the filtering artifact.
out vec3 v_position;
out vec2 v_uv;
out vec3 v_object;
out vec2 v_screen;

void main()
{
	if (u_ray_mode == 0)
	{
		v_screen = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2)) * 2.0 - 1.0;
		v_position = vec3(0.0);
		v_object = vec3(0.0, 0.0, 1.0);
		v_uv = vec2(0.0);
		v_coverage = vec2(0.0);
		gl_Position = vec4(v_screen, 0.0, 1.0);
		return;
	}
	v_screen = vec2(0.0);
	vec3 p = u_model * a_position;
	v_position = p;
	v_uv = a_uv;
	v_coverage = a_coverage;
	v_object = a_position;
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, u_camera_distance), 1.0);
	if (u_ray_mode != 0) gl_Position.xy += a_uv * gl_Position.w;
}
