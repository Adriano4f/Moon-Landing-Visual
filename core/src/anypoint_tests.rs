use super::*;

fn word(v: u16, little: bool) -> Vec<u8>
{
	if little
	{
		v.to_le_bytes().to_vec()
	}
	else
	{
		v.to_be_bytes().to_vec()
	}
}

fn long(v: u32, little: bool) -> Vec<u8>
{
	if little
	{
		v.to_le_bytes().to_vec()
	}
	else
	{
		v.to_be_bytes().to_vec()
	}
}

fn fixture(little: bool, tiled: bool, heights: &[f32; 9], nodata: &str) -> Vec<u8>
{
	let mut tags: Vec<(u16, u16, Vec<u8>)> = vec![
		(256, 4, long(3, little)),
		(257, 4, long(3, little)),
		(258, 3, word(32, little)),
		(259, 3, word(1, little)),
		(262, 3, word(1, little)),
		(277, 3, word(1, little)),
		(339, 3, word(3, little)),
		(42113, 2, format!("{nodata}\0").into_bytes()),
	];
	let mut chunks = Vec::new();
	if tiled
	{
		tags.push((322, 4, long(2, little)));
		tags.push((323, 4, long(2, little)));
		for y in [0, 2]
		{
			for x in [0, 2]
			{
				let mut chunk = Vec::new();
				for r in 0..2
				{
					for c in 0..2
					{
						let value = if y + r < 3 && x + c < 3
						{
							heights[(y + r) * 3 + x + c]
						}
						else
						{
							f32::NAN
						};
						chunk.extend(long(value.to_bits(), little));
					}
				}
				chunks.push(chunk);
			}
		}
	}
	else
	{
		tags.push((278, 4, long(2, little)));
		chunks = heights
			.chunks(6)
			.map(|v| v.iter().flat_map(|f| long(f.to_bits(), little)).collect())
			.collect();
	}
	let offsets_tag = if tiled { 324 } else { 273 };
	tags.push((offsets_tag, 4, vec![0; chunks.len() * 4]));
	tags.push((
		if tiled { 325 } else { 279 },
		4,
		chunks
			.iter()
			.flat_map(|c| long(c.len() as u32, little))
			.collect(),
	));
	let extra = tags
		.iter()
		.filter(|(_, _, b)| b.len() > 4)
		.map(|(_, _, b)| b.len())
		.sum::<usize>();
	let mut offset = 8 + 2 + tags.len() * 12 + 4 + extra;
	let offsets = tags.iter_mut().find(|t| t.0 == offsets_tag).unwrap();
	offsets.2.clear();
	for chunk in &chunks
	{
		offsets.2.extend(long(offset as u32, little));
		offset += chunk.len();
	}
	let mut bytes = if little
	{
		b"II".to_vec()
	}
	else
	{
		b"MM".to_vec()
	};
	bytes.extend(word(42, little));
	bytes.extend(long(8, little));
	bytes.extend(word(tags.len() as u16, little));
	let mut extra_bytes: Vec<u8> = Vec::new();
	let start = 8 + 2 + tags.len() * 12 + 4;
	for (tag, kind, data) in &tags
	{
		bytes.extend(word(*tag, little));
		bytes.extend(word(*kind, little));
		let size = if *kind == 3
		{
			2
		}
		else if *kind == 4
		{
			4
		}
		else
		{
			1
		};
		bytes.extend(long((data.len() / size) as u32, little));
		if data.len() <= 4
		{
			bytes.extend(data);
			bytes.extend(vec![0; 4 - data.len()]);
		}
		else
		{
			bytes.extend(long((start + extra_bytes.len()) as u32, little));
			extra_bytes.extend(data);
		}
	}
	bytes.extend(long(0, little));
	bytes.extend(extra_bytes);
	bytes.extend(chunks.into_iter().flatten());
	bytes
}

fn test_window() -> Window
{
	Window {
		projection: Projection::Geographic,
		bbox: [-1.5, -1.5, 1.5, 1.5],
		service: "LRO_LOLA_DEM_Global_256ppd_v06",
	}
}

#[test]
fn tiles_cover_globe_with_unique_ids()
{
	assert!((10_000..11_000).contains(&tile_count()));
	let mut id = 0;
	for band in 0..90
	{
		for c in 0..cells(band)
		{
			let tile = tile_of(
				-89.0 + f64::from(band) * 2.0,
				(f64::from(c) + 0.5) * 360.0 / f64::from(cells(band)),
			);
			assert_eq!(tile.id, id);
			assert!(tile.lon_deg >= 0.0 && tile.lon_deg < 360.0);
			assert_eq!(tile.half_lat_deg, 1.0);
			id += 1;
		}
	}
	assert_eq!(id, tile_count());
	assert_eq!(tile_of(0.0, -1.0), tile_of(0.0, 359.0));
	assert_eq!(tile_of(90.0, 360.0), tile_of(100.0, 0.0));
	assert_eq!(tile_of(-90.0, 0.0).lat_deg, -89.0);
	assert_eq!(tile_of(f64::NAN, f64::INFINITY), tile_of(0.0, 0.0));
}

#[test]
fn requests_choose_projection_and_split_seams()
{
	for (lat, lon, count, name) in [
		(0.0, 23.5, 2, "Global_256"),
		(-89.4398, 222.83, 2, "SPole"),
		(90.0, 0.0, 2, "NPole"),
		(0.0, 180.0, 4, "Global_256"),
		(0.0, 0.0, 2, "Global_256"),
	]
	{
		let urls = terrain_requests(lat, lon);
		assert_eq!(urls.len(), count);
		for url in urls
		{
			assert!(url.starts_with("https://trek.nasa.gov/"));
			assert!(url.contains(name));
			assert!(url.contains("size=512%2C512") || url.contains("size=1024%2C1024"));
			assert!(url.contains("pixelType=F32&compression=none"));
			assert!(url.contains("adjustAspectRatio=false"));
		}
	}
	for lon in [-180.0, -179.9, 179.9, 180.0, 540.0]
	{
		for w in windows(79.0, lon)
			.into_iter()
			.filter(|w| matches!(w.projection, Projection::Geographic))
		{
			assert!(w.bbox[0] >= -180.0 && w.bbox[2] <= 180.0);
			assert!(w.bbox[0] < w.bbox[2]);
			assert!(w.bbox[1] >= -90.0 && w.bbox[3] <= 90.0);
		}
	}
	assert!(terrain_requests(f64::NAN, 0.0).is_empty());
	assert!(terrain_requests(91.0, 0.0).is_empty());
	assert!(terrain_requests(0.0, f64::INFINITY).is_empty());
}

#[test]
fn finest_polar_service_respects_extent_and_keeps_coarse_window()
{
	for (lat, lon, fine, coarse, spacing) in [
		(
			-89.4398,
			222.83,
			"LRO_LOLA_DEM_SPole875_5mp_v04",
			"LRO_LOLA_DEM_SPole75_30mp_v04",
			5.0,
		),
		(
			89.4398,
			222.83,
			"LRO_LOLA_DEM_NPole875_5mp_v04",
			"LRO_LOLA_DEM_NPole75_30mp_v04",
			5.0,
		),
		(
			-86.0,
			0.0,
			"ldem_85s_10m_Spole_float_SP",
			"LRO_LOLA_DEM_SPole75_30mp_v04",
			10.0,
		),
		(
			86.0,
			0.0,
			"LRO_LOLA_DEM_NPole75_30mp_v04",
			"LRO_LOLA_DEM_NPole75_30mp_v04",
			30.0,
		),
		(
			-76.0,
			45.0,
			"LRO_LOLA_DEM_SPole75_30mp_v04",
			"LRO_LOLA_DEM_Global_256ppd_v06",
			30.0,
		),
		(
			76.0,
			45.0,
			"LRO_LOLA_DEM_NPole75_30mp_v04",
			"LRO_LOLA_DEM_Global_256ppd_v06",
			30.0,
		),
	]
	{
		let w = windows(lat, lon);
		assert_eq!(w.len(), 2);
		assert_eq!(w[0].service, fine);
		assert_eq!(w.last().unwrap().service, coarse);
		assert!(((w[0].bbox[2] - w[0].bbox[0]) / POLAR_FINE_SIZE as f64 - spacing).abs() < 1e-9);
		let urls = terrain_requests(lat, lon);
		assert!(urls[0].contains(fine));
		assert!(urls.last().unwrap().contains(coarse));
		if lat.abs() >= 80.0
		{
			let coarse = w.last().unwrap();
			assert_eq!(coarse.bbox[2] - coarse.bbox[0], 600_000.0);
		}
	}
	for (northing, expected) in [(73_270.0, "875_5mp"), (73_290.0, "10m_Spole")]
	{
		let lat = -90.0 + (2.0 * (northing / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
		assert!(terrain_requests(lat, 0.0)[0].contains(expected));
	}
	let c1 = windows(-89.4398, 222.83)[0];
	assert!(c1.bbox.iter().all(|v| v.abs() < 75_840.0));
	assert!(terrain_requests(-74.0, 0.0)[0].contains("Global_256"));
	assert!(terrain_requests(74.0, 0.0)[0].contains("Global_256"));
}

#[test]
fn decodes_strips_tiles_and_both_byte_orders()
{
	let values = [-123.0, 1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0];
	for little in [true, false]
	{
		for tiled in [true, false]
		{
			let bytes = fixture(little, tiled, &values, "nan");
			let r = decode(&bytes, test_window()).unwrap();
			assert_eq!(r.values, values);
			assert_eq!(r.sample(0.0, 0.0).unwrap().z_m, 7.0);
			assert_eq!(r.sample(0.5, 0.5).unwrap().z_m, 5.0);
			assert!(r.sample(3.0, 0.0).is_none());
			assert!(decode(&bytes[..bytes.len() - 1], test_window()).is_err());
		}
	}
}

#[test]
fn nodata_and_invalid_tiffs()
{
	for value in [f32::NAN, -9999.0, f32::INFINITY]
	{
		let mut values = [7.0; 9];
		values[4] = value;
		let r = decode(&fixture(true, false, &values, "-9999"), test_window()).unwrap();
		assert!(r.sample(0.0, 0.0).is_none());
		assert_eq!(r.sample(1.0, -1.0).unwrap().z_m, 7.0);
	}
	assert!(decode(&[], test_window()).is_err());
	assert!(decode(b"{\"error\":500}", test_window()).is_err());
	let bytes = fixture(true, true, &[0.0; 9], "nan");
	for n in 0..bytes.len()
	{
		assert!(decode(&bytes[..n], test_window()).is_err());
	}
	let mut bytes = fixture(true, false, &[0.0; 9], "nan");
	bytes[2] = 43;
	assert!(decode(&bytes, test_window()).is_err());
	let mut bytes = fixture(true, false, &[0.0; 9], "nan");
	bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
	assert!(decode(&bytes, test_window()).is_err());
}

#[test]
fn incremental_job_and_errors()
{
	let bytes = fixture(true, false, &[100.0; 9], "nan");
	let tiffs = vec![bytes.clone(), bytes];
	let mut job = HorizonJob::new(0.0, 23.5, 16, 2.0, &tiffs).unwrap();
	assert_eq!(job.surface_m(), 100.0);
	assert_eq!(job.progress(), 0.0);
	assert!(job.step(0).unwrap().is_none());
	assert!(job.step(3).unwrap().is_none());
	assert_eq!(job.progress(), 3.0 / 16.0);
	let h = job.step(usize::MAX).unwrap().unwrap();
	assert_eq!(job.progress(), 1.0);
	assert_eq!(h.h_deg.len(), 16);
	assert!(h.h_deg.iter().all(|v| v.is_finite() && *v < 0.0));
	assert_eq!(job.step(0).unwrap().unwrap().h_deg, h.h_deg);
	let mut full = HorizonJob::new(0.0, 23.5, 16, 2.0, &tiffs).unwrap();
	assert_eq!(full.step(16).unwrap().unwrap().h_deg, h.h_deg);
	let mut missing_ray = HorizonJob::new(0.0, 23.5, 16, 2.0, &tiffs).unwrap();
	for raster in &mut missing_ray.rasters
	{
		raster.values.fill(f32::NAN);
	}
	assert!(missing_ray.step(1).is_err());
	assert_eq!(missing_ray.progress(), 0.0);
	for (lat, n, height, data) in [
		(91.0, 16, 2.0, tiffs.as_slice()),
		(0.0, 0, 2.0, tiffs.as_slice()),
		(0.0, 16, -1.0, tiffs.as_slice()),
		(0.0, 16, 2.0, &[][..]),
	]
	{
		assert!(HorizonJob::new(lat, 23.5, n, height, data).is_err());
	}
	let missing = fixture(true, false, &[f32::NAN; 9], "nan");
	assert!(HorizonJob::new(0.0, 23.5, 16, 2.0, &[missing.clone(), missing]).is_err());
	let mut fallback = decode(&fixture(true, false, &[f32::NAN; 9], "nan"), test_window()).unwrap();
	let coarse = decode(&fixture(true, false, &[123.0; 9], "nan"), test_window()).unwrap();
	assert_eq!(
		sample(&[fallback, coarse], 0.0, 0.0).unwrap().unwrap().z_m,
		123.0
	);
	// A valid origin with no off-origin coverage must fail, not yield a flat horizon.
	fallback = decode(&fixture(true, false, &[f32::NAN; 9], "nan"), test_window()).unwrap();
	fallback.values[4] = 0.0;
	assert!(
		terrain_ray(
			HorizonRay {
				lat: 0.0,
				lon: 0.0,
				z0: 2.0,
				near: 1.0,
				limit: 100.0,
				angular: 0.01
			},
			0.0,
			&mut |p, l| sample(std::slice::from_ref(&fallback), p, l)
		)
		.is_err()
	);
}

#[test]
fn polar_axes()
{
	let (x, y) = Projection::South.xy(-89.0, 0.0);
	assert!(x.abs() < 1e-9 && y > 0.0);
	let (x, y) = Projection::North.xy(89.0, 90.0);
	assert!(x > 0.0 && y.abs() < 1e-9);
	let (x, y) = Projection::North.xy(89.0, 0.0);
	assert!(x.abs() < 1e-9 && y < 0.0);
}

#[test]
fn polar_jobs_match_geographic_ray_sampler()
{
	let bytes = fixture(
		true,
		true,
		&[-10.0, 1000.0, 35.0, 100.0, 20.0, 3.0, 0.0, 800.0, 50.0],
		"nan",
	);
	for (lat, lon) in [
		(-89.4398, 222.83),
		(89.4398, 222.83),
		(-90.0, 0.0),
		(90.0, 180.0),
		(-80.0, 90.0),
		(80.0, 270.0),
		(-76.0, 45.0),
		(76.0, 45.0),
	]
	{
		let tiffs = vec![bytes.clone(); terrain_requests(lat, lon).len()];
		let rasters = windows(lat, lon)
			.into_iter()
			.zip(&tiffs)
			.map(|(w, b)| decode(b, w).unwrap())
			.collect::<Vec<_>>();
		let expected = crate::horizon_from_terrain(lat, lon, 16, 2.0, Some(LIMIT_M), |p, l| {
			sample(&rasters, p, l)
		})
		.unwrap();
		let mut job = HorizonJob::new(lat, lon, 16, 2.0, &tiffs).unwrap();
		let actual = job.step(16).unwrap().unwrap();
		for (a, e) in actual.h_deg.iter().zip(expected.h_deg)
		{
			assert!((a - e).abs() < 1e-4, "{lat},{lon}: {a} != {e}");
		}
	}
}

/// Run with: cargo test -p lunar_core --release network_check -- --ignored --nocapture
#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "Downloads Moon Trek TIFFs using curl"]
fn network_check()
{
	use std::{
		process::Command,
		time::{Duration, Instant},
	};
	let reference: serde_json::Value = serde_json::from_str(include_str!(
		"../../app/assets/data/connecting-ridge-c1.json"
	))
	.unwrap();
	for (label, lat, lon) in [("C1", -89.4398, 222.83), ("equator", 0.0, 23.5)]
	{
		let mut tiffs = Vec::new();
		for url in terrain_requests(lat, lon)
		{
			let result = Command::new("curl")
				.args([
					"--fail",
					"--silent",
					"--show-error",
					"--location",
					"--max-time",
					"120",
					&url,
				])
				.output()
				.unwrap();
			assert!(
				result.status.success(),
				"{}",
				String::from_utf8_lossy(&result.stderr)
			);
			println!("{label}: downloaded {} bytes", result.stdout.len());
			tiffs.push(result.stdout);
		}
		let start = Instant::now();
		let mut job = HorizonJob::new(lat, lon, N_AZ, 2.0, &tiffs).unwrap();
		let surface = job.surface_m();
		let mut worst = Duration::ZERO;
		let horizon = loop
		{
			let frame = Instant::now();
			let result = job.step(20).unwrap();
			worst = worst.max(frame.elapsed());
			if let Some(horizon) = result
			{
				break horizon;
			}
		};
		let min = horizon.h_deg.iter().copied().fold(f32::INFINITY, f32::min);
		let max = horizon
			.h_deg
			.iter()
			.copied()
			.fold(f32::NEG_INFINITY, f32::max);
		println!(
			"{label}: surface={surface:.3} m, horizon=[{min:.6}, {max:.6}] deg, job={:.3} s, worst 20 bins={:.3} ms (native release)",
			start.elapsed().as_secs_f64(),
			worst.as_secs_f64() * 1000.0
		);
		assert!(min.is_finite() && max.is_finite() && min > -20.0 && max < 45.0);
		if label == "C1"
		{
			let site = &reference["site"];
			assert_eq!(site["lat_deg"].as_f64().unwrap(), lat);
			assert_eq!(site["lon_deg"].as_f64().unwrap(), lon);
			assert_eq!(
				reference["terrain"]["observer_height_m"].as_f64().unwrap(),
				2.0
			);
			assert_eq!(
				reference["terrain"]["reference_radius_m"].as_f64().unwrap(),
				MOON_RADIUS_M
			);
			let h = reference["horizon"]["h_deg"].as_array().unwrap();
			let reference_h = Horizon {
				h_deg: h.iter().map(|v| v.as_f64().unwrap() as f32).collect(),
			};
			let errors: Vec<f64> = horizon
				.h_deg
				.iter()
				.enumerate()
				.map(|(i, &v)| f64::from(v - reference_h.at(i as f32 * 360.0 / N_AZ as f32)))
				.collect();
			let max_error = errors.iter().copied().map(f64::abs).fold(0.0, f64::max);
			let rms = (errors.iter().map(|v| v * v).sum::<f64>() / errors.len() as f64).sqrt();
			println!(
				"C1 vs reference ({} bins): max={max_error:.6} deg, RMS={rms:.6} deg; reference surface={} m, observer=2 m",
				reference_h.h_deg.len(),
				reference["terrain"]["surface_height_m"]
			);
		}
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "Downloads C1 source DEMs with curl; C1_DIAGNOSTIC_DIR caches TIFFs and writes a profile"]
fn network_c1_sources()
{
	use std::{fmt::Write, process::Command};
	let directory = std::env::var_os("C1_DIAGNOSTIC_DIR").map(std::path::PathBuf::from);
	let fetch = |name: &str, url: &str| {
		println!("{name}: {url}");
		let path = directory.as_ref().map(|d| d.join(format!("{name}.tif")));
		if let Some(bytes) = path.as_ref().and_then(|p| std::fs::read(p).ok())
		{
			return bytes;
		}
		let result = Command::new("curl")
			.args([
				"--fail",
				"--silent",
				"--show-error",
				"--location",
				"--max-time",
				"120",
				url,
			])
			.output()
			.unwrap();
		assert!(
			result.status.success(),
			"{}",
			String::from_utf8_lossy(&result.stderr)
		);
		if let Some(path) = path
		{
			std::fs::write(path, &result.stdout).unwrap();
		}
		result.stdout
	};
	let (lat, lon) = (-89.4398, 222.83);
	let urls = terrain_requests(lat, lon);
	let requested = windows(lat, lon);
	let fine_bytes = fetch("trek5", &urls[0]);
	let coarse_bytes = fetch("coarse", &urls[1]);
	let fine = decode(&fine_bytes, requested[0]).unwrap();
	let coarse = decode(&coarse_bytes, requested[1]).unwrap();
	let (x, y) = to_xy(lat, lon);
	let old_window = Window {
		projection: Projection::South,
		bbox: [x - 15_360.0, y - 15_360.0, x + 15_360.0, y + 15_360.0],
		service: "LRO_LOLA_DEM_SPole75_30mp_v04",
	};
	let old_bbox = old_window.bbox.map(|v| v.to_string()).join("%2C");
	let new_bbox = requested[0].bbox.map(|v| v.to_string()).join("%2C");
	let old_url = urls[0]
		.replace(requested[0].service, old_window.service)
		.replace(&new_bbox, &old_bbox);
	let old = decode(&fetch("trek30", &old_url), old_window).unwrap();
	let pgda_url =
		"https://pgda.gsfc.nasa.gov/data/LOLA_5mpp/Site01/Site01_final_adj_5mpp_surf.tif";
	let pgda = decode(
		&fetch("pgda5", pgda_url),
		Window {
			projection: Projection::South,
			bbox: [-19_000.0, -20_000.0, -3_000.0, -4_000.0],
			service: "Site01_final_adj_5mpp_surf",
		},
	)
	.unwrap();
	assert_eq!((pgda.w, pgda.h), (3200, 3200));
	let reference: serde_json::Value = serde_json::from_str(include_str!(
		"../../app/assets/data/connecting-ridge-c1.json"
	))
	.unwrap();
	let reference_h = Horizon {
		h_deg: reference["horizon"]["h_deg"]
			.as_array()
			.unwrap()
			.iter()
			.map(|v| v.as_f64().unwrap() as f32)
			.collect(),
	};
	assert!(
		(pgda.sample(lat, lon).unwrap().z_m
			- reference["terrain"]["surface_height_m"].as_f64().unwrap())
		.abs()
			< 1e-6
	);
	for (label, p, l) in [
		("C1", lat, lon),
		(
			"north2km",
			crate::destination(lat, lon, 0.0, 2000.0).0,
			crate::destination(lat, lon, 0.0, 2000.0).1,
		),
		(
			"east2km",
			crate::destination(lat, lon, 90.0, 2000.0).0,
			crate::destination(lat, lon, 90.0, 2000.0).1,
		),
	]
	{
		println!(
			"{label}: lat={p:.9}, lon={l:.9}, xy={:?}, Trek5={:.6}, Trek30={:.6}, PGDA5={:.6}",
			to_xy(p, l),
			fine.sample(p, l).unwrap().z_m,
			old.sample(p, l).unwrap().z_m,
			pgda.sample(p, l).unwrap().z_m
		);
	}
	let trace = |rasters: &[&Raster], az: f64, limit: f64, origin: Option<f64>| {
		let (mut z0, near) = horizon_origin(lat, lon, 2.0, &mut |p, l| {
			Ok(rasters.iter().find_map(|r| r.sample(p, l)))
		})
		.unwrap();
		if let Some(z) = origin
		{
			z0 = z;
		}
		let ray = HorizonRay {
			lat,
			lon,
			z0,
			near,
			limit,
			angular: std::f64::consts::TAU / N_AZ as f64 / 4.0,
		};
		let mut peak = (-90.0_f64, 0.0, 0.0);
		let h = crate::terrain_ray_samples(ray, az, &mut |s| {
			let (p, l) = crate::destination(lat, lon, az, s);
			let point = rasters.iter().find_map(|r| r.sample(p, l));
			if let Some(ref point) = point
			{
				let e = crate::spherical_elevation(point.z_m, z0, s / MOON_RADIUS_M);
				if e > peak.0
				{
					peak = (e, s, point.z_m);
				}
			}
			Ok(point)
		})
		.unwrap();
		(f64::from(h), peak.1, peak.2)
	};
	let mut job = HorizonJob::new(lat, lon, N_AZ, 2.0, &[fine_bytes, coarse_bytes]).unwrap();
	let actual = job.step(N_AZ).unwrap().unwrap();
	let mut csv = String::from(
		"az,reference,trek5,trek30,pgda5_coarse,trek5_peak_m,trek30_peak_m,pgda5_peak_m\n",
	);
	let mut rows = Vec::new();
	let mut bridge_change = 0.0_f64;
	let mut limit_change = 0.0_f64;
	for i in 0..N_AZ
	{
		let az = i as f64 * 360.0 / N_AZ as f64;
		let new = trace(&[&fine, &coarse], az, LIMIT_M, None);
		let old_h = trace(&[&old, &coarse], az, LIMIT_M, None);
		let pgda_h = trace(&[&pgda, &coarse], az, LIMIT_M, None);
		bridge_change =
			bridge_change.max((trace(&[&fine, &old, &coarse], az, LIMIT_M, None).0 - new.0).abs());
		limit_change = limit_change.max(
			(trace(
				&[&fine, &coarse],
				az,
				std::f64::consts::PI * MOON_RADIUS_M,
				None,
			)
			.0 - new.0)
				.abs(),
		);
		assert!((new.0 - f64::from(actual.h_deg[i])).abs() < 1e-4);
		let reference = f64::from(reference_h.at(az as f32));
		writeln!(
			&mut csv,
			"{az:.1},{reference:.6},{:.6},{:.6},{:.6},{:.3},{:.3},{:.3}",
			new.0, old_h.0, pgda_h.0, new.1, old_h.1, pgda_h.1
		)
		.unwrap();
		rows.push((az, reference, new, old_h, pgda_h));
	}
	if let Some(directory) = directory
	{
		std::fs::write(directory.join("profile.csv"), csv).unwrap();
	}
	println!(
		"all-azimuth controls: bridge max change={bridge_change:.9} deg, no-distance-cap max change={limit_change:.9} deg"
	);
	let reference_resolution =
		crate::horizon_from_terrain(lat, lon, 1440, 2.0, Some(LIMIT_M), |p, l| {
			Ok([&pgda, &coarse].iter().find_map(|r| r.sample(p, l)))
		})
		.unwrap();
	let sampling_change = rows
		.iter()
		.map(|r| (r.4.0 - f64::from(reference_resolution.at(r.0 as f32))).abs())
		.fold(0.0, f64::max);
	println!(
		"PGDA5+coarse: 720 vs 1440 bins, max change={sampling_change:.9} deg at shared bearings"
	);
	for sector in 0..=8
	{
		let (start, stop) = if sector == 8
		{
			(0, 720)
		}
		else
		{
			(sector * 90, (sector + 1) * 90)
		};
		let slice = &rows[start..stop];
		let error = |which: usize| {
			let values: Vec<f64> = slice
				.iter()
				.map(|r| [r.2.0, r.3.0, r.4.0][which] - r.1)
				.collect();
			(
				values.iter().copied().map(f64::abs).fold(0.0, f64::max),
				(values.iter().map(|v| v * v).sum::<f64>() / values.len() as f64).sqrt(),
			)
		};
		println!(
			"sector {}..{}: Trek5 max/RMS={:?}, Trek30={:?}, PGDA5+coarse={:?}",
			start as f64 / 2.0,
			stop as f64 / 2.0,
			error(0),
			error(1),
			error(2)
		);
	}
	let worst = rows
		.iter()
		.max_by(|a, b| (a.2.0 - a.1).abs().total_cmp(&(b.2.0 - b.1).abs()))
		.unwrap();
	println!(
		"worst: az={}, ref={}, Trek5(h,d,z)={:?}, Trek30={:?}, PGDA5={:?}",
		worst.0, worst.1, worst.2, worst.3, worst.4
	);
	let (p, l) = crate::destination(lat, lon, worst.0, worst.2.1);
	println!(
		"worst sample heights: Trek5={:.9}, Trek30={:.9}, PGDA5={:.9}",
		fine.sample(p, l).unwrap().z_m,
		old.sample(p, l).unwrap().z_m,
		pgda.sample(p, l).unwrap().z_m
	);
	for az in [0.0, 90.0, 180.0, 270.0, worst.0]
	{
		println!(
			"controls az={az}: fine-only={:?}, with30m={:?}, reference-origin={:?}, unlimited={:?}",
			trace(&[&fine], az, LIMIT_M, None),
			trace(&[&fine, &old, &coarse], az, LIMIT_M, None),
			trace(
				&[&fine, &coarse],
				az,
				LIMIT_M,
				Some(reference["terrain"]["surface_height_m"].as_f64().unwrap() + 2.0)
			),
			trace(
				&[&fine, &coarse],
				az,
				std::f64::consts::PI * MOON_RADIUS_M,
				None
			)
		);
		for s in [2500.0, 2550.0, 2560.0, 2570.0, 2600.0, 3000.0, 4000.0]
		{
			let (p, l) = crate::destination(lat, lon, az, s);
			println!(
				"edge az={az}, d={s}: fine={:?}, coarse={:?}, old={:?}, pgda={:?}",
				fine.sample(p, l).map(|p| p.z_m),
				coarse.sample(p, l).map(|p| p.z_m),
				old.sample(p, l).map(|p| p.z_m),
				pgda.sample(p, l).map(|p| p.z_m)
			);
		}
	}
}
