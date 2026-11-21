use super::*;

fn shadow_dot(a: [f64; 3], b: [f64; 3]) -> f64
{
	a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn shadow_basis(lat: f64, lon: f64) -> [[f64; 3]; 3]
{
	let (s, c) = lat.to_radians().sin_cos();
	let (sl, cl) = lon.to_radians().sin_cos();
	[[c * sl, s, c * cl], [cl, 0.0, -sl], [-s * sl, c, -s * cl]]
}

fn shadow_window_weight(p: [f64; 3], basis: [[f64; 3]; 3]) -> f64
{
	let x = MOON_RADIUS_M * shadow_dot(p, basis[1]);
	let y = MOON_RADIUS_M * shadow_dot(p, basis[2]);
	let edge = ((12800.0 - x.abs().max(y.abs())) / 1536.0).clamp(0.0, 1.0);
	edge * edge * (3.0 - 2.0 * edge)
}

fn shadow_dem(p: [f64; 3], basis: [[f64; 3]; 3], rim: f64) -> TerrainSample
{
	let x = MOON_RADIUS_M * shadow_dot(p, basis[1]);
	let y = MOON_RADIUS_M * shadow_dot(p, basis[2]);
	let weight = shadow_window_weight(p, basis);
	let height = |spacing: f64| {
		let ix = (x / spacing).floor();
		let iy = (y / spacing).floor();
		let field = |x: f64, y: f64| {
			let local = rim * (-((x.hypot(y) - 1800.0) / 90.0).powi(2)).exp();
			let distant =
				4000.0 * (-((x - 90000.0) / 10000.0).powi(2) - (y / 10000.0).powi(2)).exp();
			((-1000.0 + local + distant) * 2.0).round() * 0.5
		};
		let tx = x / spacing - ix;
		let ty = y / spacing - iy;
		let row = |j| field(ix * spacing, j) * (1.0 - tx) + field((ix + 1.0) * spacing, j) * tx;
		row(iy * spacing) * (1.0 - ty) + row((iy + 1.0) * spacing) * ty
	};
	TerrainSample {
		z_m: height(1000.0) * (1.0 - weight) + height(50.0) * weight,
		resolution_m: if weight > 0.0 { 50.0 } else { 1000.0 },
	}
}

fn shadow_march(basis: [[f64; 3]; 3], sun: [f64; 3], rim: f64) -> f64
{
	let bias = 3.0 - 2.5 * shadow_window_weight(basis[0], basis);
	let origin = basis[0].map(|x| x * (MOON_RADIUS_M + shadow_dem(basis[0], basis, rim).z_m + bias));
	let mut distance = 25.0;
	let mut clearance_angle = 1.0_f64;
	let mut visit = |distance: f64| {
		let ray = std::array::from_fn(|i| origin[i] + distance * sun[i]);
		let radius = shadow_dot(ray, ray).sqrt();
		let p = ray.map(|x| x / radius);
		let height = shadow_dem(p, basis, rim).z_m;
		let angle = (radius - MOON_RADIUS_M - height) / distance;
		let improved = angle < clearance_angle;
		clearance_angle = clearance_angle.min(angle);
		(p, radius, improved)
	};
	for _ in 0..768
	{
		let (p, radius, _) = visit(distance);
		let edge = MOON_RADIUS_M
			* shadow_dot(p, basis[1])
				.abs()
				.max(shadow_dot(p, basis[2]).abs());
		if edge >= 12800.0 || distance >= 600000.0
		{
			break;
		}
		if radius > MOON_RADIUS_M + 15000.0 && shadow_dot(p, sun) > 0.0
		{
			break;
		}
		distance += 50.0;
	}
	let growth = (600000.0 / distance).max(1.0).powf(1.0 / 47.0);
	let mut best_distance = 0.0;
	for _ in 0..48
	{
		let (_, _, improved) = visit(distance);
		if improved
		{
			best_distance = distance;
		}
		distance *= growth;
	}
	if best_distance > 0.0
	{
		let start = best_distance / growth;
		let stride = best_distance * (growth - 1.0 / growth) / 16.0;
		for i in 0..16
		{
			visit((start + (f64::from(i) + 0.5) * stride).min(600000.0));
		}
	}
	let x = (clearance_angle / f64::from(SUN_RADIUS_DEG).to_radians()).clamp(-1.0, 1.0);
	(x.acos().mul_add(-1.0, std::f64::consts::PI) + x * (1.0 - x * x).max(0.0).sqrt())
		/ std::f64::consts::PI
}

#[test]
fn detail_shadow_march_matches_core_horizons()
{
	// Moon-fixed ephemeris directions at 2027-01-01 and 2027-01-02 00:00 UTC.
	let dated_suns = [
		[
			-0.9926345348358154,
			-0.01770699955523014,
			-0.11984651535749435,
		],
		[
			-0.9450840353965759,
			-0.017429450526833534,
			-0.32636234164237976,
		],
	];
	for (lat, lon) in [(-5.9, 179.4), (-89.45, 222.69)]
	{
		let basis = shadow_basis(lat, lon);
		for rim in [0.0, 300.0]
		{
			let horizon = horizon_from_terrain(lat, lon, 360, 0.5, Some(600000.0), |lat, lon| {
				Ok(Some(shadow_dem(shadow_basis(lat, lon)[0], basis, rim)))
			})
			.unwrap();
			for az in [0.0_f64, 90.0, 180.0, 270.0]
			{
				let h = horizon.at(az as f32);
				for offset in [-1.0_f32, -0.4, 0.0, 0.4, 1.0]
				{
					let el = f64::from(h + offset).to_radians();
					let az = az.to_radians();
					let sun = std::array::from_fn(|i| {
						basis[0][i] * el.sin()
							+ (basis[1][i] * az.sin() + basis[2][i] * az.cos()) * el.cos()
					});
					let actual = shadow_march(basis, sun, rim);
					let expected = f64::from(sun_fraction(h + offset, h));
					assert!(
						(actual - expected).abs() < 0.16,
						"{lat},{lon}: {az} {h} {offset}: {actual} != {expected}"
					);
					if offset.abs() > SUN_RADIUS_DEG
					{
						assert_eq!(actual > 0.5, offset > 0.0);
					}
				}
			}
			for sun in dated_suns
			{
				let sun = sun.map(|x| x / shadow_dot(sun, sun).sqrt());
				let az = shadow_dot(sun, basis[1])
					.atan2(shadow_dot(sun, basis[2]))
					.to_degrees()
					.rem_euclid(360.0);
				let el = shadow_dot(sun, basis[0]).asin().to_degrees() as f32;
				let actual = shadow_march(basis, sun, rim);
				let expected = f64::from(sun_fraction(el, horizon.at(az as f32)));
				assert!(
					(actual - expected).abs() < 0.16,
					"dated Sun {lat},{lon}: {actual} != {expected}"
				);
			}
		}
	}
}

#[test]
fn detail_shadow_origin_is_continuous_at_window_border()
{
	let basis = shadow_basis(0.0, 0.0);
	let origin_height = |x: f64| {
		let p = [x / MOON_RADIUS_M, 0.0, (1.0 - (x / MOON_RADIUS_M).powi(2)).sqrt()];
		shadow_dem(p, basis, 300.0).z_m + 3.0 - 2.5 * shadow_window_weight(p, basis)
	};
	assert!((origin_height(12799.99) - origin_height(12800.01)).abs() < 1e-6);
	assert_eq!(shadow_window_weight(basis[0], basis), 1.0);
	assert!(origin_height(12032.0).is_finite());
}

#[test]
fn detail_shadow_geometry_is_curved_and_falls_back_to_distant_heights()
{
	let basis = shadow_basis(0.0, 0.0);
	let sun = |el: f64| [el.to_radians().cos(), 0.0, el.to_radians().sin()];
	assert_eq!(shadow_march(basis, sun(0.0), 0.0), 0.0);
	assert_eq!(shadow_march(basis, sun(3.0), 0.0), 1.0);
	assert_eq!(shadow_march(basis, sun(6.0), 300.0), 0.0);
	assert_eq!(shadow_march(basis, sun(20.0), 300.0), 1.0);
	assert_eq!(shadow_march(basis, sun(-5.0), 0.0), 0.0);
}

#[test]
fn pgda_projection_matches_labeled_axes()
{
	// PGDA's spherical pole has no false offsets.
	assert_eq!(to_xy(-90.0, 0.0), (0.0, 0.0));
	// At the labeled 60S latitude, tan(15 degrees)=2-sqrt(3).
	let northing_60s = 2.0 * MOON_RADIUS_M * (2.0 - 3.0_f64.sqrt());
	assert!((to_xy(-60.0, 0.0).1 - northing_60s).abs() < 1e-8);
	// The Site01 label gives the upper-left corner (-19000,-4000) meters; invert its radius.
	let (x, y) = (-19000.0_f64, -4000.0_f64);
	let lat = -90.0 + (2.0 * (x.hypot(y) / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
	let lon = x.atan2(y).to_degrees();
	// A known map coordinate must round-trip, including both axis signs.
	let projected = to_xy(lat, lon);
	assert!((projected.0 - x).abs() < 1e-8 && (projected.1 - y).abs() < 1e-8);
	// Longitude 90E maps toward positive x, and longitude zero toward positive y.
	assert!(to_xy(-89.0, 90.0).0 > 30000.0);
	assert!(to_xy(-89.0, 0.0).1 > 30000.0);
}

#[test]
fn exact_elevation_and_great_circle()
{
	// Equal surface radii give the exact half-central-angle depression.
	for beta in [1e-10, 1e-5, 0.1, 1.0]
	{
		assert!((spherical_elevation(500.0, 500.0, beta) + beta.to_degrees() / 2.0).abs() < 1e-10);
	}
	// Compare against the requested unrearranged formula at a non-cancelling distance.
	let beta = 0.01_f64;
	let expected = ((MOON_RADIUS_M + 1200.0) * beta.cos() - (MOON_RADIUS_M + 802.0))
		.atan2((MOON_RADIUS_M + 1200.0) * beta.sin())
		.to_degrees();
	assert!((spherical_elevation(1200.0, 802.0, beta) - expected).abs() < 1e-10);
	// A northward meridian keeps longitude and increases latitude by the central angle.
	let d = MOON_RADIUS_M * 0.25_f64.to_radians();
	let (lat, lon) = destination(-89.45, 222.69, 0.0, d);
	assert!((lat + 89.2).abs() < 1e-10);
	assert!((lon.rem_euclid(360.0) - 222.69).abs() < 1e-10);
	// At the south pole the longitude convention still fixes the local north direction.
	let (lat, lon) = destination(-90.0, 90.0, 90.0, d);
	assert!((lat + 89.75).abs() < 1e-10 && (lon - 180.0).abs() < 1e-10);
	// At 90E, geographic east points toward negative projected y.
	assert!(
		to_xy(
			destination(-89.0, 90.0, 90.0, 100.0).0,
			destination(-89.0, 90.0, 90.0, 100.0).1
		)
		.1 < 0.0
	);
}

#[test]
fn terrain_sampler_height_edges_and_errors()
{
	// A flat sphere with a raised observer has a known tangent depression.
	let h = horizon_from_terrain(-90.0, 0.0, 8, 2.0, Some(20000.0), |_, _| {
		Ok(Some(TerrainSample {
			z_m: 0.0,
			resolution_m: 10.0,
		}))
	})
	.unwrap();
	let expected = -(MOON_RADIUS_M / (MOON_RADIUS_M + 2.0)).acos().to_degrees() as f32;
	assert!(h.h_deg.iter().all(|v| (*v - expected).abs() < 1e-5));
	// No observer data cannot produce a scientifically usable profile.
	assert!(horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Ok(None)).is_err());
	// Decoding errors must propagate instead of truncating a ray.
	assert_eq!(
		horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Err("bad TIFF".into())).unwrap_err(),
		"bad TIFF"
	);
	for (lat, n, height) in [(91.0, 8, 2.0), (-90.0, 0, 2.0), (-90.0, 8, -1.0)]
	{
		assert!(
			horizon_from_terrain(lat, 0.0, n, height, None, |_, _| panic!("must not sample"))
				.is_err()
		);
	}
	// Corrupt elevation or resolution is not a missing-data edge.
	for (z, res) in [(f64::NAN, 5.0), (0.0, 0.0), (-MOON_RADIUS_M, 5.0)]
	{
		assert!(
			horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Ok(Some(TerrainSample {
				z_m: z,
				resolution_m: res
			})))
			.is_err()
		);
	}
	// A finite coverage disk ends each ray without imposing an arbitrary production distance.
	let edge = horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |lat, lon| {
		let (x, y) = to_xy(lat, lon);
		Ok((x.hypot(y) < 1000.0).then_some(TerrainSample {
			z_m: 0.0,
			resolution_m: 10.0,
		}))
	})
	.unwrap();
	assert_eq!(edge.h_deg.len(), 8);
	assert!(edge.h_deg.iter().all(|v| v.is_finite() && *v < 0.0));
}
