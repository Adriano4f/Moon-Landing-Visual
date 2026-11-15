// Import only scientific values and geometry; C++ test includes the core header.
use super::*;

// Register independent projection metadata and axis checks; C++ TEST macro.
#[test]
fn pgda_projection_matches_labeled_axes()
{
	// PGDA's spherical pole has no false offsets; C++ exact origin assertion.
	assert_eq!(to_xy(-90.0, 0.0), (0.0, 0.0));
	// At the labeled 60S latitude, tan(15 degrees)=2-sqrt(3); C++ independent exact-angle reference.
	let northing_60s = 2.0 * MOON_RADIUS_M * (2.0 - 3.0_f64.sqrt());
	assert!((to_xy(-60.0, 0.0).1 - northing_60s).abs() < 1e-8);
	// The Site01 label gives the upper-left corner (-19000,-4000) meters; invert its radius; C++ independent inverse formula.
	let (x, y) = (-19000.0_f64, -4000.0_f64);
	let lat = -90.0 + (2.0 * (x.hypot(y) / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
	let lon = x.atan2(y).to_degrees();
	// A known map coordinate must round-trip, including both axis signs; C++ tolerance assertions.
	let projected = to_xy(lat, lon);
	assert!((projected.0 - x).abs() < 1e-8 && (projected.1 - y).abs() < 1e-8);
	// Longitude 90E maps toward positive x, and longitude zero toward positive y; C++ coordinate convention checks.
	assert!(to_xy(-89.0, 90.0).0 > 30000.0);
	assert!(to_xy(-89.0, 0.0).1 > 30000.0);
}

// Register exact geometry checks; C++ TEST macro.
#[test]
fn exact_elevation_and_great_circle()
{
	// Equal surface radii give the exact half-central-angle depression; C++ closed-form identity.
	for beta in [1e-10, 1e-5, 0.1, 1.0]
	{
		assert!((spherical_elevation(500.0, 500.0, beta) + beta.to_degrees() / 2.0).abs() < 1e-10);
	}
	// Compare against the requested unrearranged formula at a non-cancelling distance; C++ direct independent equation.
	let beta = 0.01_f64;
	let expected = ((MOON_RADIUS_M + 1200.0) * beta.cos() - (MOON_RADIUS_M + 802.0))
		.atan2((MOON_RADIUS_M + 1200.0) * beta.sin())
		.to_degrees();
	assert!((spherical_elevation(1200.0, 802.0, beta) - expected).abs() < 1e-10);
	// A northward meridian keeps longitude and increases latitude by the central angle; C++ expected destination.
	let d = MOON_RADIUS_M * 0.25_f64.to_radians();
	let (lat, lon) = destination(-89.45, 222.69, 0.0, d);
	assert!((lat + 89.2).abs() < 1e-10);
	assert!((lon.rem_euclid(360.0) - 222.69).abs() < 1e-10);
	// At the south pole the longitude convention still fixes the local north direction; C++ pole regression.
	let (lat, lon) = destination(-90.0, 90.0, 90.0, d);
	assert!((lat + 89.75).abs() < 1e-10 && (lon - 180.0).abs() < 1e-10);
	// At 90E, geographic east points toward negative projected y; C++ short-ray axis test.
	assert!(
		to_xy(
			destination(-89.0, 90.0, 90.0, 100.0).0,
			destination(-89.0, 90.0, 90.0, 100.0).1
		)
		.1 < 0.0
	);
}

// Register adaptive integration, height, and error checks; C++ TEST macro.
#[test]
fn terrain_sampler_height_edges_and_errors()
{
	// A flat sphere with a raised observer has a known tangent depression; C++ test lambda.
	let h = horizon_from_terrain(-90.0, 0.0, 8, 2.0, Some(20000.0), |_, _| {
		Ok(Some(TerrainSample {
			z_m: 0.0,
			resolution_m: 10.0,
		}))
	})
	.unwrap();
	let expected = -(MOON_RADIUS_M / (MOON_RADIUS_M + 2.0)).acos().to_degrees() as f32;
	assert!(h.h_deg.iter().all(|v| (*v - expected).abs() < 1e-5));
	// No observer data cannot produce a scientifically usable profile; C++ optional-empty callback.
	assert!(horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Ok(None)).is_err());
	// Decoding errors must propagate instead of truncating a ray; C++ expected-error callback.
	assert_eq!(
		horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Err("bad TIFF".into())).unwrap_err(),
		"bad TIFF"
	);
	// Reject nonsensical inputs before using the callback; C++ validation table.
	for (lat, n, height) in [(91.0, 8, 2.0), (-90.0, 0, 2.0), (-90.0, 8, -1.0)]
	{
		assert!(
			horizon_from_terrain(lat, 0.0, n, height, None, |_, _| panic!("must not sample"))
				.is_err()
		);
	}
	// Corrupt elevation or resolution is not a missing-data edge; C++ invalid-sample table.
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
	// A finite coverage disk ends each ray without imposing an arbitrary production distance; C++ closure with a geographic coverage predicate.
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
