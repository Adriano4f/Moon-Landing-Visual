// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Read one unit object-space sphere vertex from attribute location zero.
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Signed cross-strip or circular coordinates for analytic ray coverage; C++ vertex attribute.
in vec2 a_coverage;
out vec2 v_coverage;
// Receive the column-major view-to-clip perspective matrix from Rust.
uniform mat4 u_projection;
// Receive camera rotation times Moon-fixed-to-J2000 rotation, shared with lights and CPU overlays.
uniform mat3 u_model;
// Share the CPU camera distance with fragment viewing geometry.
uniform highp float u_camera_distance;
// Match integer precision in both stages, required by WebGL's uniform linker; C++ glUniform1i.
uniform highp int u_ray_mode;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
// Keep default smooth interpolation: derivatives of coverage-dependent centroid values worsened the filtering artifact.
out vec3 v_position;
// Pass mesh UV for the vertex interface; direction-based fragment sampling does not depend on it.
out vec2 v_uv;
// Pass unrotated object position so texture coordinates remain attached to the Moon.
out vec3 v_object;

// Run once per vertex or fragment respectively; GLSL entry points have no C++ object ownership.
void main()
{
	// Apply dated lunar orientation followed by the inertial camera rotation.
	vec3 p = u_model * a_position;
	// Export the rotated world position for perspective view direction and geometric day/night masking.
	v_position = p;
	// Export the mesh UV, while the fragment stage uses analytic direction-based sampling.
	v_uv = a_uv;
	v_coverage = a_coverage;
	// Export the original direction for texture lookup independently of rotation.
	v_object = a_position;
	// Translate world position to view space and project to homogeneous clip coordinates; rasterization divides by w.
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, u_camera_distance), 1.0);
	// Multiplication by homogeneous w preserves constant screen width after perspective division.
	if (u_ray_mode != 0) gl_Position.xy += a_uv * gl_Position.w;
}
