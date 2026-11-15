//! Derive apparent MEAN_ME vectors from a surveyed lunar reference observer.

// Reuse time parsing, scientific geometry and checked binary representation; C++ library includes.
use chrono::{NaiveDate, NaiveDateTime};
use lunar_core::{
	MOON_RADIUS_M, Site, Track,
	ephemeris::{BodyPosition, Ephemeris, topocentric},
};
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	fs,
	path::Path,
	time::{Duration, Instant},
};

// Preserve precision until the final compact conversion; C++ exact AU definition in meters.
const AU_M: f64 = 149597870700.0;

// Report errors in Sun/Earth order, independently of the request order; C++ JSON aggregate.
#[derive(Serialize)]
struct Validation
{
	// Retain the published site name; C++ string.
	name: String,
	// Number of epochs compared for each body; C++ size_t.
	samples: usize,
	// Maximum wrapped azimuth errors, degrees; C++ array<double,2>.
	max_az_deg: [f64; 2],
	// Maximum elevation errors, degrees; C++ array<double,2>.
	max_el_deg: [f64; 2],
}

// Read only the existing independent reference quantities; C++ deserialized record.
#[derive(Deserialize)]
struct Reference
{
	// Published location of the independent observation; C++ Site value.
	site: Site,
	// Existing apparent directions with their original UTC grid; C++ Track members.
	sun: Track,
	earth: Track,
	// Reference altitude from the original full-resolution run; C++ value member.
	terrain: Altitude,
}

// Keep instrument and terrain height distinct; C++ provenance aggregate.
#[derive(Deserialize)]
struct Altitude
{
	// Radial DEM height above the reference sphere; C++ double.
	surface_height_m: f64,
	// Height above that sampled surface; C++ double.
	observer_height_m: f64,
}

// Execute the explicit fixed-interval preparation command; C++ fallible command handler.
pub fn run() -> Result<(), Box<dyn Error>>
{
	// Require an external raw cache and an independent reference-data directory; C++ argument vector.
	let args: Vec<_> = std::env::args().skip(2).collect();
	if args.len() != 3
	{
		return Err("Usage: prep --ephemeris OUTPUT.bin RAW_CACHE REFERENCE_DATA_DIRECTORY".into());
	}
	let clock = Instant::now();
	let cache = Path::new(&args[1]);
	fs::create_dir_all(cache)?;
	// Bound network waits; no unbounded retry or concurrent Horizons calls; C++ HTTP client.
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(180))
		.build()?;
	let sun = fetch_body(&client, "10", cache)?;
	let earth = fetch_body(&client, "399", cache)?;
	if sun.len() != earth.len() || sun.len() != 70129
	{
		return Err("Unexpected eight-year ephemeris count".into());
	}
	// Every annual chunk has already proven its exact epoch/cadence; C++ pairwise zip.
	let data = Ephemeris {
		start_utc: year_start(2027),
		step_s: 3600,
		samples: sun.into_iter().zip(earth).map(|(s, e)| [s, e]).collect(),
	};
	let bytes = data.encode()?;
	// Validate the quantized serialized representation, not a more precise precursor; C++ checked decode.
	let decoded = Ephemeris::decode(&bytes)?;
	let validation = validate_all(&decoded, Path::new(&args[2]))?;
	// Publish only after all seven sites meet the requested angular thresholds; C++ staged output.
	let output = Path::new(&args[0]);
	fs::write(output.with_extension("bin.part"), &bytes)?;
	fs::rename(output.with_extension("bin.part"), output)?;
	// Reuse exactly the validated first sample for the application's initial picture; C++ tiny derived artifact.
	fs::write(
		output.with_extension("initial.bin"),
		initial_sample(&decoded)?,
	)?;
	let metadata = serde_json::json!({
		"format": "MLVEPH01", "frame": "Moon MEAN_ME, X=90E Y=north Z=0E", "time_scale": "UTC",
		"start": "2027-01-01T00:00:00Z", "stop": "2035-01-01T00:00:00Z", "step_s": 3600,
		"samples": decoded.samples.len(), "bytes": bytes.len(), "distance_unit": "m",
		"source": "https://ssd.jpl.nasa.gov/api/horizons.api", "quantities": "4,20",
		"reference_observer": {"lat_deg":0,"lon_deg":0,"altitude_km":0},
		"method": "AIRLESS apparent reference-observer ENU direction times apparent range, plus observer radial vector; not inertial geometric state vectors",
		"chunks": "Eight inclusive calendar-year requests per body; duplicate boundary rows removed",
		"validation": validation
	});
	fs::write(
		output.with_extension("meta.json"),
		serde_json::to_vec_pretty(&metadata)?,
	)?;
	println!(
		"Ephemeris: {} samples/body, {} bytes, elapsed {:.3} s",
		decoded.samples.len(),
		bytes.len(),
		clock.elapsed().as_secs_f64()
	);
	Ok(())
}

// Encode a normal versioned file rather than maintaining a second ad-hoc parser; C++ checked subset serialization.
fn initial_sample(data: &Ephemeris) -> Result<Vec<u8>, String>
{
	Ephemeris {
		start_utc: data.start_utc,
		step_s: data.step_s,
		samples: vec![
			*data
				.samples
				.first()
				.ok_or("Missing initial ephemeris sample")?,
		],
	}
	.encode()
}

// Date construction is internal and restricted to 2027 through 2035; C++ calendar helper.
fn year_start(year: i32) -> i64
{
	NaiveDate::from_ymd_opt(year, 1, 1)
		.unwrap()
		.and_hms_opt(0, 0, 0)
		.unwrap()
		.and_utc()
		.timestamp()
}

// Fetch chronological annual chunks below the service row limit; C++ vector assembly.
fn fetch_body(
	client: &reqwest::blocking::Client,
	body: &str,
	cache: &Path,
) -> Result<Vec<BodyPosition>, Box<dyn Error>>
{
	let mut result = Vec::with_capacity(70129);
	for year in 2027..2035
	{
		let text = request(client, body, year, cache)?;
		let mut rows = parse_chunk(&text, body, year_start(year), year_start(year + 1))?;
		// Each new chunk repeats the preceding inclusive endpoint; C++ erase first element.
		if year != 2027
		{
			rows.remove(0);
		}
		println!("Horizons {body}, {year}: {} accepted rows", rows.len());
		result.extend(rows);
	}
	Ok(result)
}

// Persist original API responses outside deployed assets for reproducibility; C++ cached HTTP helper.
fn request(
	client: &reqwest::blocking::Client,
	body: &str,
	year: i32,
	cache: &Path,
) -> Result<String, Box<dyn Error>>
{
	let path = cache.join(format!("{body}-{year}.txt"));
	if path.exists()
	{
		return Ok(fs::read_to_string(path)?);
	}
	let text = client
		.get("https://ssd.jpl.nasa.gov/api/horizons.api")
		.query(&[
			("format", "text"),
			("COMMAND", body),
			("CENTER", "'coord@301'"),
			("COORD_TYPE", "'GEODETIC'"),
			("SITE_COORD", "'0,0,0'"),
			("MAKE_EPHEM", "'YES'"),
			("EPHEM_TYPE", "'OBSERVER'"),
			("QUANTITIES", "'4,20'"),
			("ANG_FORMAT", "'DEG'"),
			("APPARENT", "'AIRLESS'"),
			("EXTRA_PREC", "'YES'"),
			("CSV_FORMAT", "'YES'"),
			("START_TIME", &format!("'{year}-01-01'")),
			("STOP_TIME", &format!("'{}-01-01'", year + 1)),
			("STEP_SIZE", "'1h'"),
		])
		.send()?
		.error_for_status()?
		.text()?;
	// An error response must not become a valid-looking cached chunk; C++ validation before publish.
	if let Err(error) = parse_chunk(&text, body, year_start(year), year_start(year + 1))
	{
		// Retain an unexpected service response for diagnosis, without accepting it as a cache hit; C++ error artifact.
		let diagnostic = cache.join(format!("{body}-{year}.failed.txt"));
		fs::write(&diagnostic, &text)?;
		return Err(format!("{error}; raw response: {}", diagnostic.display()).into());
	}
	fs::write(&path, &text)?;
	Ok(text)
}

// Check actual frame/site diagnostics, not just the requested parameters; C++ header validation.
fn check_header(text: &str, body: &str) -> Result<(), String>
{
	let line = |prefix| text.lines().find(|s| s.starts_with(prefix)).unwrap_or("");
	if !line("Center pole/equ").contains("MEAN_ME (high precision)")
		|| !line("Target body name:").contains(&format!("({body})"))
		|| !line("Center body name:").contains("Moon (301)")
	{
		return Err("Unexpected Horizons body or lunar frame".into());
	}
	// Verify the reference observer used for vector reconstruction; C++ scalar metadata checks.
	let coordinates: Vec<_> = line("Center geodetic")
		.split(':')
		.nth(1)
		.unwrap_or("")
		.split(',')
		.take(3)
		.map(|v| v.split_whitespace().next().unwrap_or("").parse::<f64>())
		.collect();
	if coordinates.len() != 3 || coordinates.iter().any(|v| !matches!(v, Ok(x) if *x == 0.0))
	{
		return Err("Unexpected Horizons reference observer".into());
	}
	// The reconstruction assumes the same spherical lunar radius as the terrain; C++ metadata invariant.
	let radii: Vec<_> = line("Center radii")
		.split(':')
		.nth(1)
		.unwrap_or("")
		.split(',')
		.take(3)
		.map(|v| v.split_whitespace().next().unwrap_or("").parse::<f64>())
		.collect();
	if radii.len() != 3
		|| radii
			.iter()
			.any(|v| !matches!(v, Ok(x) if *x == MOON_RADIUS_M / 1000.0))
	{
		return Err("Unexpected Horizons lunar reference radius".into());
	}
	Ok(())
}

// Validate every timestamp and reconstruct apparent vectors in double precision; C++ fallible parser.
fn parse_chunk(text: &str, body: &str, start: i64, stop: i64) -> Result<Vec<BodyPosition>, String>
{
	check_header(text, body)?;
	let rows = text
		.split_once("$$SOE")
		.and_then(|(_, s)| s.split_once("$$EOE").map(|(r, _)| r))
		.ok_or("Missing Horizons table delimiters")?;
	let mut result = Vec::new();
	for line in rows.lines().filter(|s| !s.trim().is_empty())
	{
		let fields: Vec<_> = line.split(',').map(str::trim).collect();
		if fields.len() != 8
		{
			return Err("Expected Horizons CSV azimuth/elevation/range columns".into());
		}
		let time = NaiveDateTime::parse_from_str(fields[0], "%Y-%b-%d %H:%M")
			.map_err(|e| e.to_string())?
			.and_utc()
			.timestamp();
		if time != start + result.len() as i64 * 3600 || time > stop
		{
			return Err("Unexpected Horizons UTC cadence or bounds".into());
		}
		result.push(reference_position(fields[3], fields[4], fields[5])?);
	}
	if stop < start || result.len() as i64 != (stop - start) / 3600 + 1
	{
		return Err("Incomplete Horizons annual chunk".into());
	}
	Ok(result)
}

// At zero latitude/longitude, ENU maps exactly to X/Y/Z; C++ analytic inverse basis.
fn reference_position(az: &str, el: &str, range: &str) -> Result<BodyPosition, String>
{
	let number = |s: &str| s.parse::<f64>().map_err(|e| e.to_string());
	let (az, el, distance) = (number(az)?, number(el)?, number(range)? * AU_M);
	if !az.is_finite()
		|| !(0.0..=360.0).contains(&az)
		|| !el.is_finite()
		|| !(-90.0..=90.0).contains(&el)
		|| !distance.is_finite()
		|| distance <= 0.0
	{
		return Err("Invalid Horizons direction or range".into());
	}
	let (sa, ca) = az.to_radians().sin_cos();
	let (se, ce) = el.to_radians().sin_cos();
	BodyPosition::from_cartesian([
		distance * ce * sa,
		distance * ce * ca,
		distance * se + MOON_RADIUS_M,
	])
}

// Compare all catalog points, never the rounded baseline in place of C1; C++ validation pipeline.
fn validate_all(data: &Ephemeris, directory: &Path) -> Result<Vec<Validation>, Box<dyn Error>>
{
	let mut results = Vec::new();
	for site in super::read_sites("sites.json")?
	{
		let path = directory.join(format!("{}.json", super::slug(&site.name)));
		let reference: Reference = serde_json::from_slice(&fs::read(path)?)?;
		if reference.site.lat_deg != site.lat_deg
			|| reference.site.lon_deg != site.lon_deg
			|| reference.sun.t != reference.earth.t
		{
			return Err("Reference site/epochs disagree with catalog".into());
		}
		let result = validate_site(data, reference)?;
		println!(
			"{}: max az Sun {:.8} Earth {:.8} deg; max el Sun {:.8} Earth {:.8} deg; {} epochs",
			result.name,
			result.max_az_deg[0],
			result.max_az_deg[1],
			result.max_el_deg[0],
			result.max_el_deg[1],
			result.samples
		);
		results.push(result);
	}
	Ok(results)
}

// Compare the compact decoded representation against independent topocentric tracks; C++ max-error reduction.
fn validate_site(data: &Ephemeris, reference: Reference) -> Result<Validation, String>
{
	let height = reference.terrain.surface_height_m + reference.terrain.observer_height_m;
	let mut result = Validation {
		name: reference.site.name,
		samples: reference.sun.t.len(),
		max_az_deg: [0.0; 2],
		max_el_deg: [0.0; 2],
	};
	for (body, track) in [&reference.sun, &reference.earth].into_iter().enumerate()
	{
		if track.t.is_empty()
			|| track.t.len() != track.az_deg.len()
			|| track.t.len() != track.el_deg.len()
		{
			return Err("Invalid reference arrays".into());
		}
		for (i, &time) in track.t.iter().enumerate()
		{
			let delta = time
				.checked_sub(data.start_utc)
				.ok_or("Reference time overflows")?;
			if delta < 0 || delta % i64::from(data.step_s) != 0
			{
				return Err("Reference epoch is off the ephemeris grid".into());
			}
			let pair = data
				.samples
				.get((delta / i64::from(data.step_s)) as usize)
				.ok_or("Reference epoch outside ephemeris")?;
			let (az, el) = topocentric(
				pair[body],
				reference.site.lat_deg,
				reference.site.lon_deg,
				height,
			)?;
			let da = ((az - f64::from(track.az_deg[i]) + 180.0).rem_euclid(360.0) - 180.0).abs();
			let de = (el - f64::from(track.el_deg[i])).abs();
			if !da.is_finite() || !de.is_finite()
			{
				return Err("Nonfinite reference angle".into());
			}
			result.max_az_deg[body] = result.max_az_deg[body].max(da);
			result.max_el_deg[body] = result.max_el_deg[body].max(de);
		}
		let tolerance = [0.01, 0.02][body];
		if result.max_az_deg[body] >= tolerance || result.max_el_deg[body] >= tolerance
		{
			return Err(format!(
				"Ephemeris validation exceeds {tolerance} degrees for {}",
				result.name
			));
		}
	}
	Ok(result)
}

// Isolate synthetic API fixtures from network and deployed resources; C++ test module.
#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn initial_sample_normal_empty_and_invalid()
	{
		let body = BodyPosition {
			direction: [0.0, 0.0, 1.0],
			distance_m: 3e8,
		};
		let mut data = Ephemeris {
			start_utc: year_start(2027),
			step_s: 3600,
			samples: vec![[body; 2]; 2],
		};
		let bytes = initial_sample(&data).unwrap();
		assert_eq!(bytes.len(), 56);
		let decoded = Ephemeris::decode(&bytes).unwrap();
		assert_eq!(decoded.samples, data.samples[..1]);
		assert_eq!(decoded.start_utc, data.start_utc);
		assert_eq!(decoded.step_s, data.step_s);
		data.step_s = 0;
		assert!(initial_sample(&data).is_err());
		data.samples.clear();
		assert!(initial_sample(&data).is_err());
	}
	// Build an analytic zenith reference, independent of finite-distance reductions; C++ test aggregate.
	fn reference() -> Reference
	{
		let track = || Track {
			t: vec![year_start(2027)],
			az_deg: vec![0.0],
			el_deg: vec![90.0],
		};
		Reference {
			site: Site {
				name: "Synthetic zenith".into(),
				lat_deg: 0.0,
				lon_deg: 0.0,
			},
			sun: track(),
			earth: track(),
			terrain: Altitude {
				surface_height_m: 0.0,
				observer_height_m: 2.0,
			},
		}
	}
	#[test]
	fn reference_validation_normal_empty_and_error()
	{
		// A finite distant zenith body projects analytically to elevation 90 degrees; C++ fixture.
		let body = BodyPosition {
			direction: [0.0, 0.0, 1.0],
			distance_m: 3e8,
		};
		let data = Ephemeris {
			start_utc: year_start(2027),
			step_s: 3600,
			samples: vec![[body, body]],
		};
		assert_eq!(
			validate_site(&data, reference()).unwrap().max_el_deg,
			[0.0; 2]
		);
		// Reject empty data, off-grid epochs and errors beyond the design limit; C++ negative fixtures.
		let mut empty = reference();
		empty.sun.t.clear();
		assert!(validate_site(&data, empty).is_err());
		let mut shifted = reference();
		shifted.sun.t[0] += 1;
		assert!(validate_site(&data, shifted).is_err());
		let mut wrong = reference();
		wrong.earth.el_deg[0] = 80.0;
		assert!(validate_site(&data, wrong).is_err());
	}
	// Supply actual CSV column/diagnostic conventions without a live request; C++ fixture builder.
	fn fixture(rows: &str) -> String
	{
		format!(
			"Target body name: Earth (399)\nCenter body name: Moon (301)\nCenter pole/equ : MEAN_ME (high precision)\nCenter geodetic : 0.0, 0.0, 0.0 {{E-lon}}\nCenter radii : 1737.4, 1737.4, 1737.4 km {{Equator_a, b, pole_c}}\n$$SOE\n{rows}\n$$EOE"
		)
	}
	#[test]
	fn csv_frame_cadence_and_errors()
	{
		let row = "2027-Jan-01 00:00, ,i,49.981905018,80.610294391,0.00260580711725,0.0577444,";
		let text = fixture(row);
		let p = parse_chunk(&text, "399", year_start(2027), year_start(2027)).unwrap();
		assert_eq!(p.len(), 1);
		assert!(p[0].distance_m > 3e8);
		for bad in [
			fixture(""),
			text.replace("MEAN_ME", "ICRF"),
			text.replace("1737.4", "1738.4"),
			text.replace("0.0, 0.0, 0.0", "1.0, 0.0, 0.0"),
			text.replace("80.610294391", "NaN"),
			text.replace("00:00", "01:00"),
			text.replace("$$EOE", ""),
			text.replace("0.00260580711725", "-1"),
		]
		{
			assert!(parse_chunk(&bad, "399", year_start(2027), year_start(2027)).is_err());
		}
		assert!(parse_chunk(&text, "10", year_start(2027), year_start(2027)).is_err());
		assert!(parse_chunk(&text, "399", year_start(2027), year_start(2027) + 3600).is_err());
	}
}
