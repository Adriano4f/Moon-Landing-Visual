//! IAU lunar orientation for display; UTC approximates TDB.

use crate::math::Mat3;

pub(crate) const INITIAL_UTC: i64 = 1_798_761_600;
const J2000_UTC: f64 = 946_728_000.0;
const RATE: f64 = 13.17635815;

fn angles(utc: i64) -> [f64; 3]
{
	let d = (utc as f64 - J2000_UTC) / 86400.0;
	let t = d / 36525.0;
	let terms = [
		(125.045, -0.0529921, -3.8787, 1.5419, 3.5610),
		(250.089, -0.1059842, -0.1204, 0.0239, 0.1208),
		(260.008, 13.0120009, 0.0700, -0.0278, -0.0642),
		(176.625, 13.3407154, -0.0172, 0.0068, 0.0158),
		(357.529, 0.9856003, 0.0, 0.0, 0.0252),
		(311.589, 26.4057084, 0.0072, -0.0029, -0.0066),
		(134.963, 13.0649930, 0.0, 0.0009, -0.0047),
		(276.617, 0.3287146, 0.0, 0.0, -0.0046),
		(34.226, 1.7484877, 0.0, 0.0, 0.0028),
		(15.134, -0.1589763, -0.0052, 0.0008, 0.0052),
		(119.743, 0.0036096, 0.0, 0.0, 0.0040),
		(239.961, 0.1643573, 0.0, 0.0, 0.0019),
		(25.053, 12.9590088, 0.0043, -0.0009, -0.0044),
	];
	let mut result = [
		269.9949 + 0.0031 * t,
		66.5392 + 0.0130 * t,
		38.3213 + RATE * d - 1.4e-12 * d * d,
	];
	for (phase, rate, ra, dec, w) in terms
	{
		let (s, c) = (phase + rate * d).rem_euclid(360.0).to_radians().sin_cos();
		result[0] += ra * s;
		result[1] += dec * c;
		result[2] += w * s;
	}
	result
}

/// Moon-fixed to J2000 equatorial, in renderer axes (Y, Z, X).
pub(crate) fn at_utc(utc: i64) -> Mat3
{
	let [ra, dec, w] = angles(utc).map(|a| a.rem_euclid(360.0).to_radians());
	let (sa, ca) = ra.sin_cos();
	let (sd, cd) = dec.sin_cos();
	let (sw, cw) = w.sin_cos();
	let node = [-sa, ca, 0.0];
	let across = [-sd * ca, -sd * sa, cd];
	let x: [f64; 3] = std::array::from_fn(|i| node[i] * cw + across[i] * sw);
	let y: [f64; 3] = std::array::from_fn(|i| -node[i] * sw + across[i] * cw);
	let z = [cd * ca, cd * sa, sd];
	let columns = [y, z, x];
	std::array::from_fn(|i| columns[i / 3][(i % 3 + 1) % 3] as f32)
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::{self, View};

	#[test]
	fn rotation_is_right_handed_and_orthonormal_across_the_timeline()
	{
		for utc in [946728000, INITIAL_UTC, 1893456000, 2051222400]
		{
			let r = at_utc(utc);
			for a in 0..3
			{
				for b in 0..3
				{
					let dot: f32 = (0..3).map(|i| r[3 * a + i] * r[3 * b + i]).sum();
					assert!((dot - if a == b { 1.0 } else { 0.0 }).abs() < 2e-6);
				}
			}
			let det = r[0] * (r[4] * r[8] - r[5] * r[7]) - r[3] * (r[1] * r[8] - r[2] * r[7])
				+ r[6] * (r[1] * r[5] - r[2] * r[4]);
			assert!((det - 1.0).abs() < 2e-6);
		}
	}

	#[test]
	fn sidereal_period_and_daily_spin_have_the_correct_direction_and_rate()
	{
		assert!((360.0 / RATE - 27.32166).abs() < 0.00001);
		for utc in [INITIAL_UTC, 1893456000, 2051136000]
		{
			let spin = angles(utc + 86400)[2] - angles(utc)[2];
			assert!((spin - RATE).abs() < 0.15, "{spin}");
		}
	}

	#[test]
	fn rigid_rotation_preserves_relative_lighting()
	{
		let sun = crate::math::direction(0.3, 1.9);
		let earth = crate::math::direction(-0.1, 0.05);
		let normal = crate::math::direction(0.6, -0.2);
		let dot = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
		for utc in [INITIAL_UTC, 1893456000, 2051222400]
		{
			let r = at_utc(utc);
			let [s, e, n] = [sun, earth, normal].map(|v| math::transform(r, v));
			assert!((dot(s, e) - dot(sun, earth)).abs() < 2e-6);
			assert!((dot(n, s) - dot(normal, sun)).abs() < 2e-6);
			assert!((dot(n, e) - dot(normal, earth)).abs() < 2e-6);
		}
	}

	#[test]
	fn earth_follow_site_centering_and_picking_compose_the_same_rotation()
	{
		for utc in [INITIAL_UTC, INITIAL_UTC + 86400, 2051222400]
		{
			let rotation = at_utc(utc);
			for point in [
				math::direction(-1.55, 2.0),
				math::direction(0.2, -0.5),
				[0.0, 0.0, 1.0],
			]
			{
				let world = math::transform(rotation, point);
				let [yaw, tilt] = math::camera_angles(world);
				let view = View {
					pan: [0.0; 2],
					yaw,
					tilt,
					zoom: 1.0,
					aspect: 1.3,
					pixels: [800.0, 600.0],
					scale: 1.0,
					rotation,
				};
				let p = view.project(point, true).unwrap();
				assert!(p.iter().all(|x| x.abs() < 2e-6));
				let back = view.unproject(p).unwrap();
				assert!(point.iter().zip(back).all(|(a, b)| (a - b).abs() < 2e-5));
			}
		}
	}
}
