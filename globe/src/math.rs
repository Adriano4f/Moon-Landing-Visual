use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub(crate) const RADIUS_M: f32 = 1_737_400.0;
pub(crate) const EXAGGERATION: f32 = 2.0;

pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

pub(crate) fn normalize(v: [f32; 3]) -> [f32; 3]
{
	let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
	v.map(|x| x / length)
}

/// Interleaved object position and UV, followed by counterclockwise triangle indices.
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
			let p = direction(FRAC_PI_2 - PI * v, TAU * (u - 0.5));
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

/// NASA unsigned heights are half-meters above a 1,727,400 m reference radius.
/// Differentiate metric relief on the sphere, then encode object-space normals as RGB8.
pub(crate) fn normal_map(heights: &[u16], w: usize, h: usize) -> Result<Vec<u8>, String>
{
	if w < 4 || h < 2 || w.checked_mul(h) != Some(heights.len())
	{
		return Err("Invalid height-map dimensions".into());
	}
	let mut normals = Vec::new();
	let d_lon = TAU / w as f32;
	let d_lat = PI / h as f32;
	for y in 0..h
	{
		let lat = FRAC_PI_2 - (y as f32 + 0.5) * d_lat;
		let north = y.saturating_sub(1);
		let south = (y + 1).min(h - 1);
		for x in 0..w
		{
			let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * TAU;
			let radial = direction(lat, lon);
			let height = |x, y| (f32::from(heights[y * w + x]) * 0.5 - 10_000.0) * EXAGGERATION;
			let radius = RADIUS_M + height(x, y);
			let east_slope = (height((x + 1) % w, y) - height((x + w - 1) % w, y))
				/ (2.0 * d_lon * radius * lat.cos());
			let north_slope =
				(height(x, north) - height(x, south)) / ((south - north) as f32 * d_lat * radius);
			let east = [lon.cos(), 0.0, -lon.sin()];
			let north_dir = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
			// The longitude basis is singular at a pole. Use a radial cap for one pixel row.
			let n = if y == 0 || y == h - 1
			{
				radial
			}
			else
			{
				normalize(std::array::from_fn(|i| {
					radial[i] - east_slope * east[i] - north_slope * north_dir[i]
				}))
			};
			normals.extend(n.map(|v| ((v * 0.5 + 0.5) * 255.0).round() as u8));
		}
	}
	Ok(normals)
}

/// Column-major, right-handed projection: 45-degree field on the shorter axis, near 0.1, far 10.
pub(crate) fn perspective(aspect: f32) -> [f32; 16]
{
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

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn mesh_orientation_and_seam()
	{
		let (v, i) = sphere(128, 64);
		assert_eq!(v.len(), 129 * 65 * 5);
		assert_eq!(i.len(), 128 * 64 * 6);
		assert!(i.iter().all(|&index| (index as usize) < v.len() / 5));
		for row in 0..=64
		{
			for axis in 0..3
			{
				assert!((v[row * 129 * 5 + axis] - v[(row * 129 + 128) * 5 + axis]).abs() < 1e-6);
			}
		}
		assert!(direction(0.0, 0.0)[2] > 0.99);
		assert!(direction(0.0, FRAC_PI_2)[0] > 0.99);
		assert!(direction(-0.75, -0.2)[1] < 0.0);
		// A middle-row triangle must face outwards, towards the camera at longitude zero.
		let a = &v[(31 * 129 + 64) * 5..];
		let b = &v[(32 * 129 + 64) * 5..];
		let c = &v[(31 * 129 + 65) * 5..];
		assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
	}

	#[test]
	fn normals_metric_slopes_caps_and_invalid_input()
	{
		assert!(normal_map(&[], 0, 0).is_err());
		assert!(normal_map(&[0; 8], 4, 3).is_err());
		assert!(normal_map(&[], usize::MAX, 2).is_err());
		let w = 16;
		let h = 8;
		let mut heights = vec![20_000; w * h];
		let flat = normal_map(&heights, w, h).unwrap();
		for y in 0..h
		{
			for x in 0..w
			{
				let expected = direction(
					FRAC_PI_2 - (y as f32 + 0.5) * PI / h as f32,
					((x as f32 + 0.5) / w as f32 - 0.5) * TAU,
				);
				for axis in 0..3
				{
					let decoded = f32::from(flat[(y * w + x) * 3 + axis]) / 255.0 * 2.0 - 1.0;
					assert!((decoded - expected[axis]).abs() < 0.004);
				}
			}
		}
		heights[4 * w + 9] += 10_000;
		let slopes = normal_map(&heights, w, h).unwrap();
		assert_ne!(
			&flat[(4 * w + 8) * 3..(4 * w + 9) * 3],
			&slopes[(4 * w + 8) * 3..(4 * w + 9) * 3]
		);
		assert_eq!(&flat[..w * 3], &slopes[..w * 3]);
		let lat = -PI / 16.0;
		let lon = TAU / 32.0;
		let radial = direction(lat, lon);
		let east = [lon.cos(), 0.0, -lon.sin()];
		let slope = 10_000.0 / (2.0 * TAU / w as f32 * RADIUS_M * lat.cos());
		let expected = normalize(std::array::from_fn(|i| radial[i] - slope * east[i]));
		for axis in 0..3
		{
			let decoded = f32::from(slopes[(4 * w + 8) * 3 + axis]) / 255.0 * 2.0 - 1.0;
			assert!((decoded - expected[axis]).abs() < 0.004);
		}
		heights[4 * w] += 10_000;
		let wrapped = normal_map(&heights, w, h).unwrap();
		assert_ne!(
			&flat[(5 * w - 1) * 3..5 * w * 3],
			&wrapped[(5 * w - 1) * 3..5 * w * 3]
		);
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
