use super::*;

#[test]
fn axes_parallax_and_numeric_direction()
{
	// A body on the prime-meridian axis is exactly overhead at zero latitude/longitude.
	let zenith = BodyPosition {
		direction: [0.0, 0.0, 1.0],
		distance_m: 384400000.0,
	};
	assert_eq!(topocentric(zenith, 0.0, 0.0, 2.0).unwrap(), (0.0, 90.0));
	// A body on the east axis has the analytic finite-distance horizon depression.
	let east = BodyPosition {
		direction: [1.0, 0.0, 0.0],
		..zenith
	};
	let (az, el) = topocentric(east, 0.0, 0.0, 2.0).unwrap();
	assert!((az - 90.0).abs() < 1e-10);
	assert!((el + ((MOON_RADIUS_M + 2.0) / 384400000.0).atan().to_degrees()).abs() < 1e-10);
	// Independent ENU components for latitude 30 and longitude 45.
	let (az, el) = topocentric(east, 30.0, 45.0, 10.0).unwrap();
	let expected_el = (384400000.0 * 0.375_f64.sqrt() - MOON_RADIUS_M - 10.0)
		.atan2(384400000.0 * 0.625_f64.sqrt())
		.to_degrees();
	assert!((az - 116.565051177078).abs() < 1e-10 && (el - expected_el).abs() < 1e-10);
	// Geographic north and longitude wrapping use the same basis.
	let north = BodyPosition {
		direction: [0.0, 1.0, 0.0],
		..zenith
	};
	assert_eq!(topocentric(north, 0.0, 360.0, 0.0).unwrap().0, 0.0);
	// Include the observer position to construct an exactly northward tangent.
	let tangent = BodyPosition::from_cartesian([0.0, 3e8, MOON_RADIUS_M]).unwrap();
	let (az, el) = topocentric(tangent, 0.0, 0.0, 0.0).unwrap();
	assert_eq!(az, 0.0);
	assert!(el.abs() < 1e-6);
	assert!(topocentric(zenith, -90.0, 222.83, -1000.0).unwrap().1 < 0.0);
}

#[test]
fn invalid_positions_and_observers()
{
	for p in [
		[0.0; 3],
		[f64::NAN, 1.0, 0.0],
		[f64::MAX; 3],
		[0.0, 0.0, MOON_RADIUS_M + 0.0001],
	]
	{
		assert!(BodyPosition::from_cartesian(p).is_err());
	}
	let body = BodyPosition::from_cartesian([3e8, 4e8, 0.0]).unwrap();
	// Preserve direction while quantizing only its compact representation.
	assert!((body.cartesian().unwrap()[0] - 3e8).abs() < 20.0);
	for (lat, lon, h) in [
		(91.0, 0.0, 0.0),
		(0.0, f64::NAN, 0.0),
		(0.0, 0.0, -MOON_RADIUS_M),
		(0.0, 0.0, f64::INFINITY),
	]
	{
		assert!(topocentric(body, lat, lon, h).is_err());
	}
	for invalid in [
		BodyPosition {
			distance_m: 0.0,
			..body
		},
		BodyPosition {
			direction: [0.0; 3],
			..body
		},
		BodyPosition {
			direction: [f32::NAN; 3],
			..body
		},
	]
	{
		assert!(invalid.cartesian().is_err());
	}
	// A body at the observer cannot be projected.
	let same = BodyPosition {
		direction: [0.0, 0.0, 1.0],
		distance_m: 2000000.0,
	};
	assert!(topocentric(same, 0.0, 0.0, 2000000.0 - MOON_RADIUS_M).is_err());
}

#[test]
fn binary_round_trip_empty_and_corrupt()
{
	let body = BodyPosition::from_cartesian([3e8, 4e8, 0.0]).unwrap();
	let data = Ephemeris {
		start_utc: 1798761600,
		step_s: 3600,
		samples: vec![[body, body]; 2],
	};
	let bytes = data.encode().unwrap();
	assert_eq!(bytes.len(), 24 + 64);
	assert_eq!(Ephemeris::decode(&bytes).unwrap(), data);
	assert_eq!(data.epoch(1), Some(1798765200));
	assert_eq!(data.epoch(2), None);
	for n in [0, 7, 23, 24, bytes.len() - 1]
	{
		assert!(Ephemeris::decode(&bytes[..n]).is_err());
	}
	for (offset, value) in [(0, 0), (16, 0), (20, 0)]
	{
		let mut bad = bytes.clone();
		if offset == 0
		{
			bad[0] = value;
		}
		else
		{
			bad[offset..offset + 4].fill(value);
		}
		assert!(Ephemeris::decode(&bad).is_err());
	}
	// Nonfinite payloads and integer-time overflow must fail before use.
	let mut bad = bytes.clone();
	bad[24..28].copy_from_slice(&f32::NAN.to_le_bytes());
	assert!(Ephemeris::decode(&bad).is_err());
	assert!(
		Ephemeris {
			samples: vec![],
			..data
		}
		.encode()
		.is_err()
	);
	assert!(
		Ephemeris {
			start_utc: i64::MAX,
			..data
		}
		.encode()
		.is_err()
	);
}
