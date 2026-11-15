use super::*;
use std::io::Cursor;
use tiff::encoder::{TiffEncoder, colortype};

fn keys() -> Vec<u16>
{
	// Radius keys reference the one DOUBLE value; all other values are inline SHORTs.
	vec![
		1, 1, 0, 13, 1024, 0, 1, 1, 1025, 0, 1, 1, 2054, 0, 1, 9102, 2057, 34736, 1, 0, 2058,
		34736, 1, 0, 2061, 0, 1, 0, 3075, 0, 1, 15, 3076, 0, 1, 9001, 3081, 34736, 1, 1, 3082, 0,
		1, 0, 3083, 0, 1, 0, 3092, 0, 1, 1, 3095, 0, 1, 0,
	]
}

/// Construct a two-by-two float32 raster with north-to-south rows.
fn fixture(values: &[f32], res: f64, invalid_projection: bool) -> Cursor<Vec<u8>>
{
	// A cursor implements both Read and Seek without filesystem I/O.
	let mut bytes = Cursor::new(Vec::new());
	{
		let mut encoder = TiffEncoder::new(&mut bytes).unwrap();
		let mut image = encoder.new_image::<colortype::Gray32Float>(2, 2).unwrap();
		// Split rows into distinct strips to exercise cache indexing across chunks.
		image.rows_per_strip(1).unwrap();
		// Pixel-is-area corner gives sample centers (-res/2,+res/2) and (+res/2,-res/2).
		image
			.encoder()
			.write_tag(Tag::ModelPixelScaleTag, &[res, res, 0.0][..])
			.unwrap();
		image
			.encoder()
			.write_tag(Tag::ModelTiepointTag, &[0.0, 0.0, 0.0, -res, res, 0.0][..])
			.unwrap();
		image
			.encoder()
			.write_tag(Tag::GeoKeyDirectoryTag, &keys()[..])
			.unwrap();
		image
			.encoder()
			.write_tag(
				Tag::GeoDoubleParamsTag,
				&[
					if invalid_projection
					{
						6378137.0
					}
					else
					{
						MOON_RADIUS_M
					},
					-90.0,
				][..],
			)
			.unwrap();
		image.encoder().write_tag(Tag::GdalNodata, "nan").unwrap();
		// Float32 preserves NaN as the actual published nodata representation.
		image.write_data(values).unwrap();
	}
	bytes.set_position(0);
	bytes
}

#[test]
fn decoded_strips_centers_interpolation_and_nodata()
{
	// These are the downloaded Site01's actual dimensions, scale and tie point.
	let real = grid(
		3200,
		3200,
		&[5.0, 5.0, 0.0],
		&[0.0, 0.0, 0.0, -19000.0, -4000.0, 0.0],
		&keys(),
		&[MOON_RADIUS_M, -90.0],
	)
	.unwrap();
	assert_eq!((real.x, real.y), (-18997.5, -4002.5));
	let mut r = Raster::new(fixture(&[0.0, 2.0, 4.0, 6.0], 5.0, false)).unwrap();
	// The upper-left sample is half a pixel inside the corner tie point.
	assert_eq!(r.height(-2.5, 2.5).unwrap(), Some(0.0));
	assert_eq!(r.height(2.5, -2.5).unwrap(), Some(6.0));
	// Equal bilinear weights at the pole average all four elevations.
	assert_eq!(r.height(0.0, 0.0).unwrap(), Some(3.0));
	assert_eq!(r.cache.len(), 2);
	assert_eq!(r.height(0.0, 0.0).unwrap(), Some(3.0));
	assert_eq!(r.order.len(), 2);
	// Do not extrapolate beyond the last sample center or accept NaN coordinates.
	assert_eq!(r.height(2.5001, 0.0).unwrap(), None);
	assert_eq!(r.height(f64::NAN, 0.0).unwrap(), None);
	// Any invalid interpolation corner causes a fallback, not a fabricated zero height.
	let mut nodata = Raster::new(fixture(&[0.0, f32::NAN, 4.0, 6.0], 5.0, false)).unwrap();
	assert_eq!(nodata.height(0.0, 0.0).unwrap(), None);
	// At an exact valid center the NaN neighbor has zero weight.
	assert_eq!(nodata.height(-2.5, 2.5).unwrap(), Some(0.0));
}

#[test]
fn finest_valid_surface_wins_and_nodata_falls_back()
{
	// Synthetic tiers intentionally disagree, proving that finest-first selection executes.
	let mut terrain = Terrain {
		rasters: vec![
			Raster::new(fixture(&[10.0; 4], 5.0, false)).unwrap(),
			Raster::new(fixture(&[20.0; 4], 20.0, false)).unwrap(),
		],
	};
	assert_eq!(terrain.sample(-90.0, 0.0).unwrap().unwrap().z_m, 10.0);
	// A location outside the fine raster but inside the coarse raster must use the latter.
	let lat = -90.0 + (2.0 * (5.0 / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
	assert_eq!(terrain.sample(lat, 90.0).unwrap().unwrap().z_m, 20.0);
	// Source NaN also triggers a valid coarse fallback.
	terrain.rasters[0] = Raster::new(fixture(&[f32::NAN; 4], 5.0, false)).unwrap();
	assert_eq!(terrain.sample(-90.0, 0.0).unwrap().unwrap().z_m, 20.0);
	assert!(terrain.sample(-80.0, 0.0).unwrap().is_none());
}

#[test]
fn rejects_wrong_sphere_bad_keys_and_corrupt_tiff()
{
	// An Earth radius may not enter lunar ray geometry.
	assert!(Raster::new(fixture(&[0.0; 4], 5.0, true)).is_err());
	// A NaN radius must not pass through floating-point comparison semantics.
	assert!(
		grid(
			2,
			2,
			&[5.0, 5.0, 0.0],
			&[0.0; 6],
			&keys(),
			&[f64::NAN, -90.0]
		)
		.is_err()
	);
	// Invalid bytes and missing key-directory headers are errors, not empty images.
	assert!(Raster::new(Cursor::new(vec![])).is_err());
	assert!(key(&[], &[], 1024).is_err());
	assert!(key(&[1, 1, 0, 1, 2057, 34736, 1, 2], &[MOON_RADIUS_M], 2057).is_err());
	// Duplicated key definitions must not depend on entry order.
	assert!(key(&[1, 1, 0, 2, 1024, 0, 1, 1, 1024, 0, 1, 2], &[], 1024).is_err());
	let inventory = assets().unwrap();
	assert_eq!(inventory.len(), 5);
	assert!(inventory.iter().map(|a| a.bytes).sum::<u64>() < 20_000_000_000);
}
