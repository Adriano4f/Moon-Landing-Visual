use super::*;

fn flat() -> Dem
{
	Dem {
		w: 21,
		h: 21,
		res_m: 100.0,
		x_min_m: -1000.0,
		y_max_m: 1000.0,
		z_m: vec![0.0; 441],
	}
}

#[test]
fn pole_and_bilinear_grid()
{
	assert_eq!(to_xy(-90.0, 123.0), (0.0, 0.0));
	let dem = Dem {
		w: 2,
		h: 2,
		res_m: 1.0,
		x_min_m: 0.0,
		y_max_m: 1.0,
		z_m: vec![0.0, 2.0, 4.0, 6.0],
	};
	assert_eq!(dem.height_at(0.5, 0.5), Some(3.0));
	assert_eq!(dem.height_at(1.0, 0.0), Some(6.0));
	assert_eq!(dem.height_at(-0.1, 0.0), None);
	assert_eq!(dem.height_at(f64::NAN, 0.0), None);
}

#[test]
fn flat_horizon_includes_curvature()
{
	let dem = flat();
	let horizon = horizon_from_dem(&dem, -90.0, 0.0, 8, 800.0);
	let expected = (-50.0 / (2.0 * MOON_RADIUS_M)).atan().to_degrees() as f32;
	for h in horizon.h_deg
	{
		assert!(h < 0.0 && (h - expected).abs() < 1e-6);
	}
	assert_eq!(
		horizon_from_dem(&dem, -80.0, 0.0, 1, 800.0).h_deg,
		vec![-90.0]
	);
	assert!(
		horizon_from_dem(&dem, -90.0, 0.0, 0, 800.0)
			.h_deg
			.is_empty()
	);
}

#[test]
fn wall_angle_and_local_azimuth()
{
	let mut dem = flat();
	for c in 0..dem.w
	{
		dem.z_m[15 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -90.0, 90.0, 4, 800.0);
	let expected = ((100.0 - 500.0_f64.powi(2) / (2.0 * MOON_RADIUS_M)) / 500.0)
		.atan()
		.to_degrees() as f32;
	assert!((horizon.h_deg[1] - expected).abs() < 1e-4);
	assert!(horizon.h_deg[3] < 0.0);
}

#[test]
fn north_ridge_away_from_pole_singularity()
{
	let (_, y) = to_xy(-89.0, 0.0);
	let mut dem = Dem {
		w: 21,
		h: 41,
		res_m: 100.0,
		x_min_m: -1000.0,
		y_max_m: y + 1000.0,
		z_m: vec![0.0; 21 * 41],
	};
	for c in 0..dem.w
	{
		dem.z_m[15 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -89.0, 0.0, 4, 800.0);
	assert!(horizon.h_deg[0] > 10.0);
	assert!(horizon.h_deg[2] < 0.0);
}

#[test]
fn east_at_lon_90_points_toward_negative_y()
{
	let (x, y) = to_xy(-89.0, 90.0);
	let mut dem = Dem {
		w: 21,
		h: 41,
		res_m: 100.0,
		x_min_m: x - 1000.0,
		y_max_m: y + 1000.0,
		z_m: vec![0.0; 21 * 41],
	};
	for c in 0..dem.w
	{
		dem.z_m[15 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -89.0, 90.0, 4, 800.0);
	assert!(horizon.h_deg[1] > 10.0);
	assert!(horizon.h_deg[3] < 0.0);
}

#[test]
fn interpolation_wraps()
{
	let h = Horizon {
		h_deg: vec![0.0, 10.0, 20.0, 30.0],
	};
	assert_eq!(h.at(360.0), 0.0);
	assert_eq!(h.at(-45.0), 15.0);
	assert_eq!(h.at(405.0), 5.0);
	assert_eq!(Horizon { h_deg: vec![] }.at(0.0), -90.0);
}

#[test]
fn visibility_and_solar_disc()
{
	assert_eq!(sun_fraction(0.0, 0.0), 0.5);
	assert_eq!(sun_fraction(-1.0, 0.0), 0.0);
	assert_eq!(sun_fraction(1.0, 0.0), 1.0);
	let track = Track {
		t: vec![0, 1],
		az_deg: vec![0.0, 360.0],
		el_deg: vec![0.0, 1.0],
	};
	assert_eq!(
		visible(&track, &Horizon { h_deg: vec![0.0] }, 0.5),
		vec![false, true]
	);
	assert!(visible(&Track::default(), &Horizon { h_deg: vec![] }, 0.0).is_empty());
}

#[test]
fn weighted_stats_include_gap()
{
	let s = stats(
		&[0, 3600, 7200, 21600, 25200],
		&[true, false, false, true, true],
		&[true, true, false, false, true],
	);
	assert!((s.pct_sun - 200.0 / 7.0).abs() < 1e-9);
	assert!((s.pct_earth - 200.0 / 7.0).abs() < 1e-9);
	assert!((s.pct_both - 100.0 / 7.0).abs() < 1e-9);
	assert_eq!(s.longest_night_h, 5.0);
	assert_eq!(stats(&[], &[], &[]).pct_sun, 0.0);
	assert_eq!(stats(&[0], &[false], &[false]).longest_night_h, 0.0);
}

#[test]
#[should_panic]
fn stats_rejects_unordered_times()
{
	stats(&[1, 0], &[true; 2], &[true; 2]);
}

#[test]
fn horizons_sample_and_errors()
{
	let track = parse_horizons("header\n$$SOE\n 2026-Dec-01 00:00 *    57.386895   1.766619\n 2026-Dec-01 06:00 C m 58.0 2.0\n$$EOE\nfooter").unwrap();
	assert_eq!(track.t, vec![1796083200, 1796104800]);
	assert_eq!(track.az_deg[0], 57.386895);
	assert_eq!(track.el_deg[0], 1.766619);
	for text in [
		"",
		"$$SOE",
		"$$SOE\n$$EOE",
		"$$SOE\nbad row\n$$EOE",
		"$$SOE\n2026-Dec-01 00:00 n.a. n.a.\n$$EOE",
	]
	{
		assert!(parse_horizons(text).is_err());
	}
}

#[test]
fn horizons_combined_optional_markers_and_last_numeric_pair()
{
	for marker in ["*i", " i", "*", "", "CNAnim"]
	{
		let text = format!("$$SOE\n 2027-Jan-01 05:00 {marker} 123.45 1.23\n$$EOE");
		let track = parse_horizons(&text).unwrap();
		assert_eq!(track.az_deg, vec![123.45]);
		assert_eq!(track.el_deg, vec![1.23]);
	}
	let track = parse_horizons("$$SOE\n2027-Jan-01 05:00 999 *i 123.45 flag -1.23 trailing\n$$EOE")
		.unwrap();
	assert_eq!(track.az_deg, vec![123.45]);
	assert_eq!(track.el_deg, vec![-1.23]);
	for values in ["*i", "i 123.45", "*i NaN 1.23", "* 123.45 inf"]
	{
		let text = format!("$$SOE\n2027-Jan-01 05:00 {values}\n$$EOE");
		assert!(parse_horizons(&text).is_err());
	}
}
