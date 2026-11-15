// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// Keep CPU projection and shader view directions at the same illustrative distance; C++ constant.
pub(crate) const CAMERA_DISTANCE: f32 = 5.0;

// Estimate reflected visible irradiance relative to the Sun using NASA geometric albedo and mean separation; C++ scalar helper.
pub(crate) fn earthshine_ratio(sun: [f32; 3], earth: [f32; 3]) -> f32
{
	// Normalize the phase-angle dot product independently of small trigonometric roundoff; C++ norm product.
	let norm =
		(sun.iter().map(|x| x * x).sum::<f32>() * earth.iter().map(|x| x * x).sum::<f32>()).sqrt();
	// Invalid or empty directions must not create spurious Earth light; C++ finite/range guard.
	if !norm.is_finite() || norm < 1e-6
	{
		// Invalid input contributes no reflected irradiance; C++ early return.
		return 0.0;
	}
	// Earth's phase is approximately the supplement of the lunar phase at the Moon; C++ acos after clamp.
	let alpha = PI
		- ((sun[0] * earth[0] + sun[1] * earth[1] + sun[2] * earth[2]) / norm)
			.clamp(-1.0, 1.0)
			.acos();
	// Integrate a Lambert sphere's illuminated visible disk, normalized to one at full Earth; C++ scalar expression.
	let phase = ((alpha.sin() + (PI - alpha) * alpha.cos()) / PI).max(0.0);
	// NASA Earth geometric albedo 0.434, mean radius 6371 km, lunar semimajor axis 384400 km; C++ inverse-square scaling.
	0.434 * (6371.0_f32 / 384400.0).powi(2) * phase
}

// Keep all CPU/GPU camera parameters in one copyable value; C++ plain value struct.
#[derive(Clone, Copy)]
pub(crate) struct View
{
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub yaw: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub tilt: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub zoom: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub aspect: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub pixels: [f32; 2],
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub scale: f32,
}

// Reuse identical rigid/perspective transforms for markers and triangle ribbons; C++ member definitions.
impl View
{
	// Latitude tilt after longitude rotation; C++ matrix composition.
	pub fn model(self) -> [f32; 9]
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (s, c) = self.yaw.sin_cos();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (st, ct) = self.tilt.sin_cos();
		[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c]
	}
	// Zoom narrows the field of view without entering the sphere; C++ matrix scale.
	pub fn projection(self) -> [f32; 16]
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut p = perspective(self.aspect);
		p[0] *= self.zoom;
		p[5] *= self.zoom;
		p
	}
	// Optionally reject sphere-occluded surface markers, but project off-sphere ray tips; C++ checked projection.
	pub fn project(self, point: [f32; 3], surface: bool) -> Option<[f32; 2]>
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let m = self.model();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let p: [f32; 3] =
			std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
		if (surface && CAMERA_DISTANCE * p[2] <= 1.0) || p[2] >= CAMERA_DISTANCE - 0.1
		{
			return None;
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let projection = self.projection();
		Some([
			projection[0] * p[0] / (CAMERA_DISTANCE - p[2]),
			projection[5] * p[1] / (CAMERA_DISTANCE - p[2]),
		])
	}
	// Build two camera-facing triangles and a 20-triangle endpoint disk in clip-offset coordinates; C++ vector<float>.
	pub fn ribbon(self, origin: [f32; 3], end: [f32; 3]) -> Vec<f32>
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (Some(a), Some(b)) = (self.project(origin, false), self.project(end, false))
		else
		{
			return vec![];
		};
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dx = (b[0] - a[0]) * self.pixels[0];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dy = (b[1] - a[1]) * self.pixels[1];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let length = dx.hypot(dy).max(1e-6);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let offset = [
			-dy / length * 2.5 * self.scale / self.pixels[0],
			dx / length * 2.5 * self.scale / self.pixels[1],
		];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut vertices = Vec::with_capacity(462);
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for (p, sign) in [
			(origin, -1.0),
			(end, -1.0),
			(end, 1.0),
			(origin, -1.0),
			(end, 1.0),
			(origin, 1.0),
		]
		{
			vertices.extend_from_slice(&[
				p[0],
				p[1],
				p[2],
				offset[0] * sign,
				offset[1] * sign,
				sign,
				0.0,
			]);
		}
		self.endpoint(&mut vertices, end);

		vertices
	}
	fn endpoint(self, vertices: &mut Vec<f32>, end: [f32; 3])
	{
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for i in 0..20
		{
			// Visit each value without transferring the owner's resources; C++ range-for loop.
			for angle in [
				None,
				Some(i as f32 * TAU / 20.0),
				Some((i + 1) as f32 * TAU / 20.0),
			]
			{
				// Bind this intermediate value for the current operation; C++ local initialization.
				let uv = angle.map_or([0.0, 0.0], |a| {
					[
						a.cos() * 4.0 * self.scale / self.pixels[0],
						a.sin() * 4.0 * self.scale / self.pixels[1],
					]
				});
				vertices.extend_from_slice(&[
					end[0],
					end[1],
					end[2],
					uv[0],
					uv[1],
					angle.map_or(0.0, f32::cos),
					angle.map_or(0.0, f32::sin),
				]);
			}
		}
	}
}

// Convert latitude/longitude in radians to a unit object-space direction; C++ returns array<float,3>.
pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	// Return x=cos(lat)sin(lon), y=sin(lat), z=cos(lat)cos(lon); C++ returns an aggregate array.
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

// Convert local north-clockwise azimuth/elevation to a Moon-fixed unit vector, all angles in radians; C++ optional<array<float,3>>.
pub(crate) fn local_direction(lat: f32, lon: f32, az: f32, el: f32) -> Option<[f32; 3]>
{
	// Invalid external angles must clear the rays rather than contaminate GPU vertices; C++ finite/range guards.
	if [lat, lon, az, el].iter().any(|v| !v.is_finite())
		|| lat.abs() > FRAC_PI_2
		|| el.abs() > FRAC_PI_2
	{
		// No valid direction exists for this input; C++ nullopt.
		return None;
	}
	// Up points away from the center in the renderer's y-north, z-zero-longitude convention; C++ radial vector.
	let up = direction(lat, lon);
	// Differentiate the radial vector with respect to latitude; C++ normalized north tangent.
	let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
	// Differentiate with respect to longitude and normalize analytically, including at the pole; C++ east tangent.
	let east = [lon.cos(), 0.0, -lon.sin()];
	// Resolve the local unit vector onto east, north and up; C++ component-wise linear combination.
	Some(std::array::from_fn(|i| {
		east[i] * el.cos() * az.sin() + north[i] * el.cos() * az.cos() + up[i] * el.sin()
	}))
}

/// Interleaved object position and UV, followed by counterclockwise triangle indices.
// Generate a UV sphere with duplicate seam vertices and indexed triangles; C++ returns two vectors.
pub(crate) fn sphere(slices: u32, stacks: u32) -> (Vec<f32>, Vec<u32>)
{
	// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
	assert!(slices >= 3 && stacks >= 2);
	// Own interleaved position/UV values on the CPU; C++ vector<float>.
	let mut vertices = Vec::new();
	// Own counterclockwise triangle index triples on the CPU; C++ vector<uint32_t>.
	let mut indices = Vec::new();
	// Visit every latitude ring, including both polar endpoints; C++ uses an inclusive integer for loop.
	for row in 0..=stacks
	{
		// Compute the north-to-south normalized row coordinate; C++ float division after explicit casts.
		let v = row as f32 / stacks as f32;
		// Visit every longitude endpoint, including the duplicate seam column; C++ uses an inclusive integer for loop.
		for col in 0..=slices
		{
			// Compute the west-to-east normalized column coordinate; C++ float division after explicit casts.
			let u = col as f32 / slices as f32;
			// Compute this mesh corner's unit object-space position; C++ std::array<float,3>.
			// Collapse polar rings to exact points and make both seam copies bit-identical.
			// C++ likewise must avoid tiny position differences between duplicated boundary vertices.
			let p = if row == 0
			{
				[0.0, 1.0, 0.0]
			}
			else if row == stacks
			{
				[0.0, -1.0, 0.0]
			}
			else
			{
				direction(
					FRAC_PI_2 - PI * v,
					TAU * (if col == slices { 0.0 } else { u } - 0.5),
				)
			};
			// Append position xyz and UV to one interleaved vertex; C++ vector::insert appends these five floats.
			vertices.extend_from_slice(&[p[0], p[1], p[2], u, v]);
		}
	}
	// Visit each latitude cell band, excluding the final endpoint ring; C++ uses row < stacks.
	for row in 0..stacks
	{
		// Visit each longitude cell between neighboring columns; C++ uses col < slices.
		for col in 0..slices
		{
			// Index this cell's upper-left corner; C++ uint32_t indexes the vertex array.
			let a = row * (slices + 1) + col;
			// Index the lower-left corner one complete ring later; C++ integer addition.
			let b = a + slices + 1;
			// Append two counterclockwise triangles for the current grid cell; C++ vector::insert appends six indices.
			indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
		}
	}
	// Move both vectors into the returned tuple; C++ returns a pair with move semantics.
	(vertices, indices)
}

/// Column-major, right-handed projection: 45-degree field on the shorter axis, near 0.1, far 10.
// Return a column-major right-handed projection; C++ returns array<float,16>.
pub(crate) fn perspective(aspect: f32) -> [f32; 16]
{
	// Compute cotangent of half the field of view, with portrait compensation; C++ tan from cmath.
	let f = aspect.min(1.0) / (PI / 8.0).tan();
	// Set the near clipping distance to 0.1 sphere radii; C++ const float.
	let near = 0.1;
	// Set the far clipping distance to 10 sphere radii; C++ const float.
	let far = 10.0;
	[
		f / aspect,
		0.0,
		0.0,
		0.0,
		0.0,
		f,
		0.0,
		0.0,
		0.0,
		0.0,
		(far + near) / (near - far),
		-1.0,
		0.0,
		0.0,
		2.0 * far * near / (near - far),
		0.0,
	]
}

// Compose a 30-degree south presentation tilt after geographic-north rotation; C++ returns a column-major array<float,9>.
#[cfg(test)]
pub(crate) fn rotation(angle: f32) -> [f32; 9]
{
	// Compute the animated longitude rotation; C++ separate std::sin/std::cos calls.
	let (s, c) = angle.sin_cos();
	// Negative x rotation brings the south pole toward the camera; C++ radians and trigonometric calls.
	let (st, ct) = (-PI / 6.0).sin_cos();
	// Return Rx(-30 deg)*Ry(angle), with each consecutive triple a column; C++ aggregate initialization.
	[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c]
}

// Project a unit object-space site only when visible on the sphere; C++ optional<array<float,2>>.
#[cfg(test)]
pub(crate) fn marker(point: [f32; 3], angle: f32, aspect: f32) -> Option<[f32; 2]>
{
	// Do not let invalid external coordinates or an empty viewport create NaN UI positions; C++ finite-value guard.
	if !aspect.is_finite()
		|| aspect <= 0.0
		|| !angle.is_finite()
		|| point.iter().any(|p| !p.is_finite())
	{
		// Absence means no marker rather than an invented position; C++ nullopt.
		return None;
	}
	// Use exactly the shader's rigid transform; C++ array<float,9>.
	let m = rotation(angle);
	// Multiply column-major matrix by the object direction; C++ array<float,3> computed by a loop.
	let p: [f32; 3] =
		std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
	// A surface point faces the camera iff p dot (eye-p)>0; C++ sphere occlusion test.
	if CAMERA_DISTANCE * p[2] <= 1.0
	{
		// Hide far-side and tangent markers instead of drawing through the Moon; C++ nullopt.
		return None;
	}
	// Reuse the GPU perspective scales and camera distance; C++ projection matrix lookup.
	let projection = perspective(aspect);
	// Divide clip x/y by the camera-relative depth; C++ returns an optional projected point.
	Some([
		projection[0] * p[0] / (CAMERA_DISTANCE - p[2]),
		projection[5] * p[1] / (CAMERA_DISTANCE - p[2]),
	])
}

// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
#[cfg(test)]
// Group tests in a nested module; C++ puts test helpers in a namespace.
mod tests
{
	// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
	use super::*;

	#[test]
	fn camera_margin_zoom_out_and_shared_uniform()
	{
		for zoom in [super::super::MIN_ZOOM, 1.0]
		{
			let view = View {
				yaw: 0.0,
				tilt: 0.0,
				zoom,
				aspect: 0.6,
				pixels: [600.0, 1000.0],
				scale: 1.0,
			};
			for lat in -90..=90
			{
				for lon in 0..360
				{
					if let Some(p) = view.project(
						direction((lat as f32).to_radians(), (lon as f32).to_radians()),
						true,
					)
					{
						assert!(p.iter().all(|v| v.abs() < 0.9));
					}
				}
			}
		}
		for shader in [include_str!("moon.vert"), include_str!("moon.frag")]
		{
			assert!(shader.contains("uniform highp float u_camera_distance;"));
		}
	}

	// Check full, new, quarter Earth and malformed vectors independently of shader display exposure; C++ test registration.
	#[test]
	fn earthshine_phase_and_scale()
	{
		// Opposite Sun/Earth directions at the Moon imply full Earth; C++ physical reference value.
		let full = earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
		// Full Earth is about one part in 8388 of sunlight; C++ near-equality assertion.
		assert!((full - 0.000119214).abs() < 1e-8);
		// Coincident Sun and Earth directions imply new Earth; C++ zero assertion.
		assert_eq!(earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), 0.0);
		// Quarter phase is 1/pi, not one half, for a Lambert sphere; C++ ratio assertion.
		assert!(
			(earthshine_ratio([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]) / full - 1.0 / PI).abs() < 1e-6
		);
		// Empty direction vectors must not create light; C++ invalid-input assertion.
		assert_eq!(earthshine_ratio([0.0; 3], [0.0; 3]), 0.0);
		// Nonfinite ephemerides must not enter shader uniforms; C++ finite-input assertion.
		assert_eq!(earthshine_ratio([f32::NAN; 3], [1.0; 3]), 0.0);
	}

	// Camera centering and billboards must agree with the GPU at high zoom; C++ regression registration.
	#[test]
	fn centered_site_and_billboard_width()
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let lat = (-89.4398_f32).to_radians();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let lon = 222.83_f32.to_radians();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let view = View {
			yaw: -lon,
			tilt: lat,
			zoom: 128.0,
			aspect: 1.0,
			pixels: [800.0, 800.0],
			scale: 2.0,
		};
		// Bind this intermediate value for the current operation; C++ local initialization.
		let point = direction(lat, lon);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let center = view.project(point, true).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(center[0].abs() < 1e-4 && center[1].abs() < 1e-4);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let vertices = view.ribbon(point, [point[0] + 0.6, point[1], point[2]]);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(vertices.len(), 462);
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(vertices.iter().all(|v| v.is_finite()));
		// Bind this intermediate value for the current operation; C++ local initialization.
		let width = (vertices[3].powi(2) + vertices[4].powi(2)).sqrt() * 800.0;
		// Enforce the independent regression expectation; C++ test assertion.
		assert!((width - 5.0).abs() < 1e-4);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let aligned = view.ribbon(point, point);
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(aligned.iter().all(|v| v.is_finite()));
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(view.project([0.0, 0.0, -4.0], true).is_none());
		// Reject points behind the camera instead of emitting invalid ribbon vertices; C++ empty-vector error case.
		assert!(
			view.ribbon(point.map(|v| v * CAMERA_DISTANCE), point)
				.is_empty()
		);
		// A camera-aligned direction retains its finite endpoint disk; C++ vertex-layout assertion.
		assert_eq!(aligned.len(), 462);
		// The first outer disk vertex is 2 logical points from its center; C++ screen-space radius check.
		assert!((aligned[52].hypot(aligned[53]) * 400.0 - 4.0).abs() < 1e-4);
	}

	// Register an independent local-frame regression; C++ test registration.
	#[test]
	fn local_sky_frame()
	{
		// Exercise the actual selected south-pole site; C++ scalar setup.
		let lat = (-89.4398_f32).to_radians();
		// Longitude remains east-positive despite the renderer's unusual axes; C++ scalar setup.
		let lon = 222.83_f32.to_radians();
		// Straight up must equal the geographic radial vector; C++ optional extraction.
		let up = local_direction(lat, lon, 0.0, FRAC_PI_2).unwrap();
		// Geographic north is the derivative of the radial vector with latitude; C++ expected components.
		let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
		// Check both basis directions component by component; C++ counted loop.
		for i in 0..3
		{
			// Single-precision trigonometry warrants a small absolute tolerance; C++ numerical assertion.
			assert!((up[i] - direction(lat, lon)[i]).abs() < 1e-6);
			// Zero azimuth/elevation must point north, not along longitude or toward the center; C++ numerical assertion.
			assert!((local_direction(lat, lon, 0.0, 0.0).unwrap()[i] - north[i]).abs() < 1e-6);
		}
		// A nontrivial closed-form example detects swapped east/north or renderer axes; C++ fixture vector.
		let expected = [0.41602117, 0.7803301, -0.4669168];
		// At latitude 30, longitude 60, azimuth 45, elevation 30 degrees; C++ direction calculation.
		let actual = local_direction(PI / 6.0, PI / 3.0, PI / 4.0, PI / 6.0).unwrap();
		// All components must agree independently of the pole fixture; C++ counted assertions.
		for i in 0..3
		{
			// Enforce the independent regression expectation; C++ test assertion.
			assert!((actual[i] - expected[i]).abs() < 1e-6);
		}
		// Invalid data must not become a GPU coordinate; C++ error-case assertion.
		assert!(local_direction(f32::NAN, 0.0, 0.0, 0.0).is_none());
		// Elevation beyond the physical sky range is rejected; C++ range-error assertion.
		assert!(local_direction(0.0, 0.0, 0.0, PI).is_none());
	}

	// Register a marker projection regression; C++ testing macro.
	#[test]
	// Verify orientation, occlusion, all rotation phases and invalid input; C++ unit test.
	fn site_projection_and_south_tilt()
	{
		// A near-side equatorial point must be visible; C++ assert optional engagement.
		assert!(marker(direction(0.0, 0.0), 0.0, 1.0).is_some());
		// Its antipode must be hidden; C++ assert disengagement.
		assert!(marker(direction(0.0, PI), 0.0, 1.0).is_none());
		// No marker should be emitted for invalid projection inputs; C++ finite-value error cases.
		assert!(marker([f32::NAN, 0.0, 1.0], 0.0, 1.0).is_none());
		// Zero width cannot be projected; C++ assert error/absence.
		assert!(marker(direction(0.0, 0.0), 0.0, 0.0).is_none());
		// Check the actual example site throughout a complete turn; C++ counted loop.
		for deg in 0..360
		{
			// Convert geographic degrees to the same unit direction used by set_site; C++ trig helper call.
			let point = direction((-89.45_f32).to_radians(), 222.69_f32.to_radians());
			// South tilt must expose this near-pole site at every phase; C++ checked optional extraction.
			let projected = marker(point, (deg as f32).to_radians(), 0.6).unwrap();
			// The marker stays in the lower hemisphere and inside the viewport; C++ tolerance bounds.
			assert!(projected[0].abs() < 1.0 && (-1.0..0.0).contains(&projected[1]));
		}
		// Guard the shared shader contract, so geometry and normals cannot silently diverge; C++ source contract test.
		assert!(include_str!("moon.vert").contains("uniform mat3 u_model;"));
		// Both stages must use the same matrix; C++ source contract assertion.
		assert!(include_str!("moon.frag").contains("uniform mat3 u_model;"));
		// WebGL rejects shared uniforms whose integer precisions differ between stages; C++ shader-contract assertion.
		for shader in [include_str!("moon.vert"), include_str!("moon.frag")]
		{
			// Require the explicit matching qualifier instead of different stage defaults; C++ substring assertion.
			assert!(shader.contains("uniform highp int u_ray_mode;"));
		}
	}

	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Check the production mesh's sizes, seam, orientation and winding; C++ would use test assertions.
	fn mesh_orientation_and_seam()
	{
		// Generate the production mesh for regression checks; C++ structured binding receives two vectors.
		let (v, i) = sphere(256, 128);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(v.len(), 257 * 129 * 5);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(i.len(), 256 * 128 * 6);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(i.iter().all(|&index| (index as usize) < v.len() / 5));
		// Check every duplicated seam row, including the polar rows; C++ loops over rows.
		for row in 0..=128
		{
			// Check xyz separately so every seam component is bit-identical; C++ loops over array indices.
			for axis in 0..3
			{
				// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
				assert_eq!(v[row * 257 * 5 + axis], v[(row * 257 + 256) * 5 + axis]);
			}
		}
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(0.0, 0.0)[2] > 0.99);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(0.0, FRAC_PI_2)[0] > 0.99);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(-0.75, -0.2)[1] < 0.0);
		// A middle-row triangle must face outwards, towards the camera at longitude zero.
		// Select the first cell vertex or a borrowed triangle corner; C++ integer or span.
		let a = &v[(63 * 257 + 128) * 5..];
		// Select the next-row vertex or a borrowed triangle corner; C++ integer or span.
		let b = &v[(64 * 257 + 128) * 5..];
		// Select the third triangle corner for the winding check; C++ span.
		let c = &v[(63 * 257 + 129) * 5..];
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
	}

	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Check portrait scaling and near/far depth endpoints; C++ would use test assertions.
	fn projection_depth_and_aspect()
	{
		// Construct the landscape matrix for endpoint checks; C++ std::array<float,16>.
		let p = perspective(2.0);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((p[5] / p[0] - 2.0).abs() < 1e-6);
		// Build the portrait projection to check that the short-axis field stays fixed; C++ array<float,16>.
		let portrait = perspective(0.5);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((portrait[0] - p[5]).abs() < 1e-6);
		// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
		for (z, expected) in [(-0.1, -1.0), (-10.0, 1.0)]
		{
			// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
			assert!(((p[10] * z + p[14]) / -z - expected).abs() < 1e-5);
		}
	}
}
