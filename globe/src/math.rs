use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub(crate) const CAMERA_DISTANCE: f32 = 5.0;

/// Column-major 3×3 matrix.
pub(crate) type Mat3 = [f32; 9];
#[cfg(test)]
pub(crate) const IDENTITY: Mat3 = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

pub(crate) fn transform(m: Mat3, v: [f32; 3]) -> [f32; 3]
{
	std::array::from_fn(|i| m[i] * v[0] + m[i + 3] * v[1] + m[i + 6] * v[2])
}

fn product(a: Mat3, b: Mat3) -> Mat3
{
	std::array::from_fn(|i| (0..3).map(|k| a[3 * k + i % 3] * b[3 * (i / 3) + k]).sum())
}

pub(crate) fn camera_angles(direction: [f32; 3]) -> [f32; 2]
{
	[
		-direction[0].atan2(direction[2]),
		direction[1].clamp(-1.0, 1.0).asin(),
	]
}

/// Estimate reflected visible irradiance relative to the Sun using NASA geometric albedo and mean separation.
pub(crate) fn earthshine_ratio(sun: [f32; 3], earth: [f32; 3]) -> f32
{
	let norm =
		(sun.iter().map(|x| x * x).sum::<f32>() * earth.iter().map(|x| x * x).sum::<f32>()).sqrt();
	// Invalid or empty directions must not create spurious Earth light.
	if !norm.is_finite() || norm < 1e-6
	{
		return 0.0;
	}
	// Earth's phase is approximately the supplement of the lunar phase at the Moon.
	let alpha = PI
		- ((sun[0] * earth[0] + sun[1] * earth[1] + sun[2] * earth[2]) / norm)
			.clamp(-1.0, 1.0)
			.acos();
	// Integrate a Lambert sphere's illuminated visible disk, normalized to one at full Earth.
	let phase = ((alpha.sin() + (PI - alpha) * alpha.cos()) / PI).max(0.0);
	// NASA Earth geometric albedo 0.434, mean radius 6371 km, lunar semimajor axis 384400 km.
	0.434 * (6371.0_f32 / 384400.0).powi(2) * phase
}

/// Shared CPU/GPU camera transforms; orbit angles are radians.
#[derive(Clone, Copy)]
pub(crate) struct View
{
	pub rotation: Mat3,
	pub yaw: f32,
	pub tilt: f32,
	pub zoom: f32,
	/// Additive screen-space offset in normalized device coordinates.
	pub pan: [f32; 2],
	pub aspect: f32,
	/// Physical viewport dimensions in pixels.
	pub pixels: [f32; 2],
	/// Physical pixels per logical UI point.
	pub scale: f32,
}

impl View
{
	/// Latitude tilt after longitude rotation.
	pub fn model(self) -> [f32; 9]
	{
		let (s, c) = self.yaw.sin_cos();
		let (st, ct) = self.tilt.sin_cos();
		product(
			[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c],
			self.rotation,
		)
	}
	pub fn camera_distance(self) -> f32
	{
		1.0 + (CAMERA_DISTANCE - 1.0) / self.zoom.max(1.0)
	}

	pub fn near(self) -> f32
	{
		((self.camera_distance() - 1.0) * 0.25).clamp(0.00001, 0.1)
	}

	/// Perspective projection with zoom-dependent clipping and an NDC lens shift.
	pub fn projection(self) -> [f32; 16]
	{
		let mut p = perspective(self.aspect);
		p[0] *= self.zoom.min(1.0);
		p[5] *= self.zoom.min(1.0);
		p[8] = -self.pan[0];
		p[9] = -self.pan[1];
		let near = self.near();
		let far = self.camera_distance() + 5.0;
		p[10] = (far + near) / (near - far);
		p[14] = 2.0 * far * near / (near - far);
		p
	}
	/// Optionally reject sphere-occluded surface markers, but project off-sphere ray tips.
	pub fn project(self, point: [f32; 3], surface: bool) -> Option<[f32; 2]>
	{
		let m = self.model();
		let p: [f32; 3] =
			std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
		let distance = self.camera_distance();
		if (surface && distance * p[2] <= 1.0) || p[2] >= distance - self.near()
		{
			return None;
		}
		let projection = self.projection();
		Some([
			projection[0] * p[0] / (distance - p[2]) + self.pan[0],
			projection[5] * p[1] / (distance - p[2]) + self.pan[1],
		])
	}
	/// Invert project for the unit sphere: nearest ray hit in Moon-fixed coordinates.
	pub fn unproject(self, ndc: [f32; 2]) -> Option<[f32; 3]>
	{
		let projection = self.projection();
		let d = [
			(ndc[0] - self.pan[0]) / projection[0],
			(ndc[1] - self.pan[1]) / projection[5],
			-1.0,
		];
		let distance = self.camera_distance();
		// Stable nearest root of |o + t d|^2 = 1, matching the surface shader.
		let a = d[0] * d[0] + d[1] * d[1] + 1.0;
		let b = -2.0 * distance;
		let c = (distance - 1.0) * (distance + 1.0);
		let disc = b * b - 4.0 * a * c;
		if disc < 0.0
		{
			return None;
		}
		let t = 2.0 * c / (-b + disc.sqrt());
		let p = [t * d[0], t * d[1], distance - t];
		let m = self.model();
		Some(std::array::from_fn(|j| {
			m[3 * j] * p[0] + m[3 * j + 1] * p[1] + m[3 * j + 2] * p[2]
		}))
	}
	/// Build two camera-facing triangles and a 20-triangle endpoint disk in clip-offset coordinates.
	pub fn ribbon(self, origin: [f32; 3], end: [f32; 3]) -> Vec<f32>
	{
		let (Some(a), Some(b)) = (self.project(origin, false), self.project(end, false))
		else
		{
			return vec![];
		};
		let dx = (b[0] - a[0]) * self.pixels[0];
		let dy = (b[1] - a[1]) * self.pixels[1];
		let length = dx.hypot(dy).max(1e-6);
		let offset = [
			-dy / length * 2.5 * self.scale / self.pixels[0],
			dx / length * 2.5 * self.scale / self.pixels[1],
		];
		let mut vertices = Vec::with_capacity(462);
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
		for i in 0..20
		{
			for angle in [
				None,
				Some(i as f32 * TAU / 20.0),
				Some((i + 1) as f32 * TAU / 20.0),
			]
			{
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

	pub fn arrowhead(self, origin: [f32; 3], end: [f32; 3], clearance: f32) -> Vec<f32>
	{
		let (Some(a), Some(b)) = (self.project(origin, false), self.project(end, false))
		else
		{
			return vec![];
		};
		let delta = [
			(b[0] - a[0]) * self.pixels[0],
			(b[1] - a[1]) * self.pixels[1],
		];
		let length = delta[0].hypot(delta[1]);
		if !length.is_finite() || length < 1e-5 || !clearance.is_finite() || clearance < 0.0
		{
			return vec![];
		}
		let direction = delta.map(|v| v / length);
		let mut vertices = Vec::with_capacity(84);
		for side in [-1.0, 1.0]
		{
			let wing = [
				-direction[0] * 16.0 - direction[1] * side * 7.0,
				-direction[1] * 16.0 + direction[0] * side * 7.0,
			];
			let norm = wing[0].hypot(wing[1]);
			let normal = [-wing[1] / norm * 1.5, wing[0] / norm * 1.5];
			for (offset, edge) in [
				(wing, -1.0),
				([0.0; 2], -1.0),
				([0.0; 2], 1.0),
				(wing, -1.0),
				([0.0; 2], 1.0),
				(wing, 1.0),
			]
			{
				vertices.extend_from_slice(&[
					end[0],
					end[1],
					end[2],
					(offset[0] + normal[0] * edge - direction[0]*2.0*clearance) * self.scale / self.pixels[0],
					(offset[1] + normal[1] * edge - direction[1]*2.0*clearance) * self.scale / self.pixels[1],
					edge,
					0.0,
				]);
			}
		}
		vertices
	}
}

/// Convert latitude/longitude in radians to a unit object-space direction.
pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	// Return x=cos(lat)sin(lon), y=sin(lat), z=cos(lat)cos(lon).
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

/// Convert local north-clockwise azimuth/elevation to a Moon-fixed unit vector, all angles in radians.
pub(crate) fn local_direction(lat: f32, lon: f32, az: f32, el: f32) -> Option<[f32; 3]>
{
	// Invalid external angles must clear the rays rather than contaminate GPU vertices.
	if [lat, lon, az, el].iter().any(|v| !v.is_finite())
		|| lat.abs() > FRAC_PI_2
		|| el.abs() > FRAC_PI_2
	{
		return None;
	}
	// Up points away from the center in the renderer's y-north, z-zero-longitude convention.
	let up = direction(lat, lon);
	// Differentiate the radial vector with respect to latitude.
	let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
	// Differentiate with respect to longitude and normalize analytically, including at the pole.
	let east = [lon.cos(), 0.0, -lon.sin()];
	// Resolve the local unit vector onto east, north and up.
	Some(std::array::from_fn(|i| {
		east[i] * el.cos() * az.sin() + north[i] * el.cos() * az.cos() + up[i] * el.sin()
	}))
}

/// Interleaved object position and UV, followed by counterclockwise triangle indices.
/// Generate a UV sphere with duplicate seam vertices and indexed triangles.
#[cfg(test)]
pub(crate) fn sphere(slices: u32, stacks: u32) -> (Vec<f32>, Vec<u32>)
{
	assert!(slices >= 3 && stacks >= 2);
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	for row in 0..=stacks
	{
		let v = row as f32 / stacks as f32;
		for col in 0..=slices
		{
			let u = col as f32 / slices as f32;
			// Collapse polar rings to exact points and make both seam copies bit-identical.
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
			vertices.extend_from_slice(&[p[0], p[1], p[2], u, v]);
		}
	}
	for row in 0..stacks
	{
		for col in 0..slices
		{
			let a = row * (slices + 1) + col;
			let b = a + slices + 1;
			indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
		}
	}
	(vertices, indices)
}

/// Column-major, right-handed projection: 45-degree field on the shorter axis, near 0.1, far 10.
pub(crate) fn perspective(aspect: f32) -> [f32; 16]
{
	// Compute cotangent of half the field of view, with portrait compensation.
	let f = aspect.min(1.0) / (PI / 8.0).tan();
	let near = 0.1;
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

/// Compose a 30-degree south presentation tilt after geographic-north rotation.
#[cfg(test)]
pub(crate) fn rotation(angle: f32) -> [f32; 9]
{
	let (s, c) = angle.sin_cos();
	// Negative x rotation brings the south pole toward the camera.
	let (st, ct) = (-PI / 6.0).sin_cos();
	// Return Rx(-30 deg)*Ry(angle), with each consecutive triple a column.
	[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c]
}

/// Project a unit object-space site only when visible on the sphere.
#[cfg(test)]
pub(crate) fn marker(point: [f32; 3], angle: f32, aspect: f32) -> Option<[f32; 2]>
{
	// Do not let invalid external coordinates or an empty viewport create NaN UI positions.
	if !aspect.is_finite()
		|| aspect <= 0.0
		|| !angle.is_finite()
		|| point.iter().any(|p| !p.is_finite())
	{
		// Absence means no marker rather than an invented position.
		return None;
	}
	let m = rotation(angle);
	// Multiply column-major matrix by the object direction.
	let p: [f32; 3] =
		std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
	// A surface point faces the camera iff p dot (eye-p)>0.
	if CAMERA_DISTANCE * p[2] <= 1.0
	{
		// Hide far-side and tangent markers instead of drawing through the Moon.
		return None;
	}
	let projection = perspective(aspect);
	// Divide clip x/y by the camera-relative depth.
	Some([
		projection[0] * p[0] / (CAMERA_DISTANCE - p[2]),
		projection[5] * p[1] / (CAMERA_DISTANCE - p[2]),
	])
}

#[cfg(test)]
mod tests
{
	use super::*;

	fn panned_view(zoom: f32) -> View
	{
		View {
			rotation: IDENTITY,
			yaw: -0.4,
			tilt: 0.2,
			zoom,
			pan: [0.35, -0.27],
			aspect: 1.4,
			pixels: [1400.0, 1000.0],
			scale: 1.0,
		}
	}

	#[test]
	fn panned_surface_projection_round_trip()
	{
		for zoom in [0.5, 1.0, 8.0, 128.0, 1024.0]
		{
			let view = panned_view(zoom);
			let point = direction(0.2001, 0.4001);
			let ndc = view.project(point, true).unwrap();
			let back = view.unproject(ndc).unwrap();
			assert!(point.iter().zip(back).all(|(a, b)| (a - b).abs() < 2e-4));
			assert!(view.project(direction(-0.2, 0.4 + PI), true).is_none());
		}
		assert!(panned_view(1.0).unproject([10.0, 10.0]).is_none());
	}

	#[test]
	fn pan_adds_ndc_offset_to_projection()
	{
		for zoom in [0.5, 1.0, 8.0, 128.0]
		{
			let view = panned_view(zoom);
			let centered = View { pan: [0.0; 2], ..view };
			let point = direction(0.201, 0.402);
			let a = centered.project(point, true).unwrap();
			let b = view.project(point, true).unwrap();
			for i in 0..2
			{
				assert!((b[i] - a[i] - view.pan[i]).abs() < 1e-5);
			}
		}
	}

	#[test]
	fn panned_cpu_projection_matches_gpu_matrix_and_surface_ray()
	{
		for zoom in [0.5, 1.0, 8.0, 128.0]
		{
			let view = panned_view(zoom);
			let point = direction(0.201, 0.402);
			let p = transform(view.model(), point);
			let eye = [p[0], p[1], p[2] - view.camera_distance(), 1.0];
			let projection = view.projection();
			let clip: [f32; 4] = std::array::from_fn(|i| {
				(0..4).map(|j| projection[4 * j + i] * eye[j]).sum()
			});
			let ndc = view.project(point, true).unwrap();
			for i in 0..2
			{
				assert!((clip[i] / clip[3] - ndc[i]).abs() < 1e-5);
			}
			let ray = [
				(ndc[0] + projection[8]) / projection[0],
				(ndc[1] + projection[9]) / projection[5],
				-1.0,
			];
			for i in 0..2
			{
				assert!((ray[i] - eye[i] / -eye[2]).abs() < 1e-5);
			}
		}
		assert!(include_str!("moon.frag").contains("v_screen + u_projection[2].xy"));
	}

	#[test]
	fn camera_margin_zoom_out_and_shared_uniform()
	{
		for zoom in [super::super::MIN_ZOOM, 1.0]
		{
			let view = View {
				rotation: IDENTITY,
				pan: [0.0; 2],
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

	#[test]
	fn deep_camera_stays_above_surface_with_useful_depth_precision()
	{
		for zoom in [0.5, 1.0, 8.0, 64.0, 256.0, 1024.0]
		{
			let view = View {
				rotation: IDENTITY,
				pan: [0.0; 2],
				yaw: 0.0,
				tilt: 0.0,
				zoom,
				aspect: 0.6,
				pixels: [600.0, 1000.0],
				scale: 1.0,
			};
			let altitude = view.camera_distance() - 1.0;
			assert!(altitude > view.near() && view.near() > 0.0);
			assert_eq!(view.project([0.0, 0.0, 1.0], true), Some([0.0, 0.0]));
			assert!(view.project([0.0, 0.0, -1.0], true).is_none());
			let p = view.projection();
			let depth = (p[10] * -altitude + p[14]) / altitude;
			assert!(depth > -1.0 && depth < 1.0);
			// A metre of separation remains distinguishable in a 24-bit depth buffer.
			let offset = altitude + 1.0 / 1_737_400.0;
			let next_depth = (p[10] * -offset + p[14]) / offset;
			if zoom >= 64.0
			{
				assert!((next_depth - depth).abs() > 2.0 / 16_777_216.0);
			}
		}
	}

	#[test]
	fn earthshine_phase_and_scale()
	{
		let full = earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
		// Full Earth is about one part in 8388 of sunlight.
		assert!((full - 0.000119214).abs() < 1e-8);
		assert_eq!(earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), 0.0);
		// Quarter phase is 1/pi, not one half, for a Lambert sphere.
		assert!(
			(earthshine_ratio([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]) / full - 1.0 / PI).abs() < 1e-6
		);
		// Empty direction vectors must not create light.
		assert_eq!(earthshine_ratio([0.0; 3], [0.0; 3]), 0.0);
		// Nonfinite ephemerides must not enter shader uniforms.
		assert_eq!(earthshine_ratio([f32::NAN; 3], [1.0; 3]), 0.0);
	}

	/// Camera centering and billboards must agree with the GPU at high zoom.
	#[test]
	fn centered_site_and_billboard_width()
	{
		let lat = (-89.4398_f32).to_radians();
		let lon = 222.83_f32.to_radians();
		let view = View {
			rotation: IDENTITY,
			pan: [0.0; 2],
			yaw: -lon,
			tilt: lat,
			zoom: 128.0,
			aspect: 1.0,
			pixels: [800.0, 800.0],
			scale: 2.0,
		};
		let point = direction(lat, lon);
		let center = view.project(point, true).unwrap();
		assert!(center[0].abs() < 1e-4 && center[1].abs() < 1e-4);
		let vertices = view.ribbon(point, [point[0] + 0.6, point[1], point[2]]);
		assert_eq!(vertices.len(), 462);
		for (at_origin, tip, sign) in [
			(true, point, 1.0),
			(false, [point[0] + 0.6, point[1], point[2]], -1.0),
		]
		{
			let start=if at_origin {[point[0]+0.6,point[1],point[2]]} else {point};
			let head = view.arrowhead(start,tip,if at_origin {11.0} else {0.0});
			assert_eq!(head.len(), 84);
			assert!(head.iter().all(|v| v.is_finite()));
			assert!(head.chunks_exact(7).all(|v| v[..3] == tip));
			if at_origin {assert!(head.chunks_exact(7).all(|v|(v[3]*view.pixels[0]).hypot(v[4]*view.pixels[1])/(2.0*view.scale)>10.0));}
			let a=view.project(point,false).unwrap();
			let b=view.project([point[0]+0.6,point[1],point[2]],false).unwrap();
			assert!(head.chunks_exact(7).map(|v|v[3]*(b[0]-a[0])+v[4]*(b[1]-a[1])).sum::<f32>()*sign>0.0);
		}
		assert!(view.arrowhead(point, point, 0.0).is_empty());
		assert!(
			view.arrowhead(point.map(|v| v * CAMERA_DISTANCE), point, 0.0)
				.is_empty()
		);
		assert!(vertices.iter().all(|v| v.is_finite()));
		let width = (vertices[3].powi(2) + vertices[4].powi(2)).sqrt() * 800.0;
		assert!((width - 5.0).abs() < 1e-4);
		let aligned = view.ribbon(point, point);
		assert!(aligned.iter().all(|v| v.is_finite()));
		assert!(view.project([0.0, 0.0, -4.0], true).is_none());
		// Reject points behind the camera instead of emitting invalid ribbon vertices.
		assert!(
			view.ribbon(point.map(|v| v * CAMERA_DISTANCE), point)
				.is_empty()
		);
		// A camera-aligned direction retains its finite endpoint disk.
		assert_eq!(aligned.len(), 462);
		// The first outer disk vertex is 2 logical points from its center.
		assert!((aligned[52].hypot(aligned[53]) * 400.0 - 4.0).abs() < 1e-4);
	}

	#[test]
	fn local_sky_frame()
	{
		// Exercise the actual selected south-pole site.
		let lat = (-89.4398_f32).to_radians();
		// Longitude remains east-positive despite the renderer's unusual axes.
		let lon = 222.83_f32.to_radians();
		// Straight up must equal the geographic radial vector.
		let up = local_direction(lat, lon, 0.0, FRAC_PI_2).unwrap();
		// Geographic north is the derivative of the radial vector with latitude.
		let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
		for i in 0..3
		{
			// Single-precision trigonometry warrants a small absolute tolerance.
			assert!((up[i] - direction(lat, lon)[i]).abs() < 1e-6);
			// Zero azimuth/elevation must point north, not along longitude or toward the center.
			assert!((local_direction(lat, lon, 0.0, 0.0).unwrap()[i] - north[i]).abs() < 1e-6);
		}
		// A nontrivial closed-form example detects swapped east/north or renderer axes.
		let expected = [0.41602117, 0.7803301, -0.4669168];
		// At latitude 30, longitude 60, azimuth 45, elevation 30 degrees.
		let actual = local_direction(PI / 6.0, PI / 3.0, PI / 4.0, PI / 6.0).unwrap();
		// All components must agree independently of the pole fixture.
		for i in 0..3
		{
			assert!((actual[i] - expected[i]).abs() < 1e-6);
		}
		// Invalid data must not become a GPU coordinate.
		assert!(local_direction(f32::NAN, 0.0, 0.0, 0.0).is_none());
		// Elevation beyond the physical sky range is rejected.
		assert!(local_direction(0.0, 0.0, 0.0, PI).is_none());
	}

	#[test]
	fn site_projection_and_south_tilt()
	{
		// A near-side equatorial point must be visible.
		assert!(marker(direction(0.0, 0.0), 0.0, 1.0).is_some());
		// Its antipode must be hidden.
		assert!(marker(direction(0.0, PI), 0.0, 1.0).is_none());
		assert!(marker([f32::NAN, 0.0, 1.0], 0.0, 1.0).is_none());
		assert!(marker(direction(0.0, 0.0), 0.0, 0.0).is_none());
		for deg in 0..360
		{
			let point = direction((-89.45_f32).to_radians(), 222.69_f32.to_radians());
			// South tilt must expose this near-pole site at every phase.
			let projected = marker(point, (deg as f32).to_radians(), 0.6).unwrap();
			assert!(projected[0].abs() < 1.0 && (-1.0..0.0).contains(&projected[1]));
		}
		// Guard the shared shader contract, so geometry and normals cannot silently diverge.
		assert!(include_str!("moon.vert").contains("uniform mat3 u_model;"));
		// Both stages must use the same matrix.
		assert!(include_str!("moon.frag").contains("uniform mat3 u_model;"));
		for shader in [include_str!("moon.vert"), include_str!("moon.frag")]
		{
			// Require the explicit matching qualifier instead of different stage defaults.
			assert!(shader.contains("uniform highp int u_ray_mode;"));
		}
	}

	#[test]
	fn mesh_orientation_and_seam()
	{
		let (v, i) = sphere(256, 128);
		assert_eq!(v.len(), 257 * 129 * 5);
		assert_eq!(i.len(), 256 * 128 * 6);
		assert!(i.iter().all(|&index| (index as usize) < v.len() / 5));
		// Check every duplicated seam row, including the polar rows.
		for row in 0..=128
		{
			// Check xyz separately so every seam component is bit-identical.
			for axis in 0..3
			{
				assert_eq!(v[row * 257 * 5 + axis], v[(row * 257 + 256) * 5 + axis]);
			}
		}
		assert!(direction(0.0, 0.0)[2] > 0.99);
		assert!(direction(0.0, FRAC_PI_2)[0] > 0.99);
		assert!(direction(-0.75, -0.2)[1] < 0.0);
		// A middle-row triangle must face outwards, towards the camera at longitude zero.
		let a = &v[(63 * 257 + 128) * 5..];
		let b = &v[(64 * 257 + 128) * 5..];
		let c = &v[(63 * 257 + 129) * 5..];
		assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
	}

	#[test]
	fn projection_depth_and_aspect()
	{
		let p = perspective(2.0);
		assert!((p[5] / p[0] - 2.0).abs() < 1e-6);
		let portrait = perspective(0.5);
		assert!((portrait[0] - p[5]).abs() < 1e-6);
		for (z, expected) in [(-0.1, -1.0), (-10.0, 1.0)]
		{
			assert!(((p[10] * z + p[14]) / -z - expected).abs() < 1e-5);
		}
	}
}
