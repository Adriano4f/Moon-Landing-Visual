// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Read one unit object-space sphere vertex from attribute location zero.
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Receive the column-major view-to-clip perspective matrix from Rust.
uniform mat4 u_projection;
// Receive Rx(-30 degrees)*Ry(animation), shared with normals and the CPU site marker; C++ glUniformMatrix3fv.
uniform mat3 u_model;
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
	// Rotate about geographic north, then tilt the south pole toward the camera; C++ matrix-vector multiplication.
	vec3 p = u_model * a_position;
	// Export the rotated world position for perspective view direction and geometric day/night masking.
	v_position = p;
	// Export the mesh UV, while the fragment stage uses analytic direction-based sampling.
	v_uv = a_uv;
	// Export the original direction for texture lookup independently of rotation.
	v_object = a_position;
	// Translate world position to view space and project to homogeneous clip coordinates; rasterization divides by w.
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, 3.3), 1.0);
}
