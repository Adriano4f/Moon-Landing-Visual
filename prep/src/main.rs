//! Offline Horizons and PGDA terrain preparation for fixed landing sites.

// Keep native TIFF decoding out of core and the web app; C++ implementation translation unit.
mod dem;
// Parse date-only UTC endpoints; C++ chrono::year_month_day.
use chrono::NaiveDate;
// Reuse geometry and statistics, not duplicated formulas; C++ scientific header.
use lunar_core::{
	EARTH_MARGIN_DEG, Horizon, Site, Track, horizon_from_terrain, parse_horizons, stats,
	sun_fraction, visible,
};
// Generate JSON encoders; C++ struct-to-JSON adapters.
use serde::Serialize;
// Native I/O and elapsed clocks belong in preparation; C++ standard headers.
use std::{
	error::Error,
	fs,
	path::Path,
	time::{Duration, Instant},
};

// Serialize provenance beside terrain bins; C++ JSON-adapted aggregate.
#[derive(Serialize)]
struct TerrainInfo
{
	// Instrument height above the DEM surface; C++ double.
	observer_height_m: f64,
	// Radial surface height above the reference sphere; C++ double.
	surface_height_m: f64,
	// Radius validated against the GeoKeys; C++ double.
	reference_radius_m: f64,
	// Exact files, URLs, sizes and digests; C++ vector<Asset>.
	assets: Vec<dem::Asset>,
}

// Keep the legacy track/profile schema with additional provenance; C++ JSON aggregate.
#[derive(Serialize)]
struct SiteData
{
	// Geographic site definition; C++ Site value.
	site: Site,
	// Aligned directions for each body; C++ Track members.
	sun: Track,
	earth: Track,
	// None explicitly means legacy flat preparation; C++ optional<Horizon>.
	horizon: Option<Horizon>,
	// Older app versions may ignore this new field; C++ optional<TerrainInfo>.
	terrain: Option<TerrainInfo>,
}

// Query one body at the instrument's radial altitude; C++ expected-returning HTTP helper.
fn download(
	client: &reqwest::blocking::Client,
	site: &Site,
	observer_z_m: f64,
	body: &str,
	start: &str,
	stop: &str,
	step: &str,
) -> Result<Track, Box<dyn Error>>
{
	// Horizons altitude is kilometers, unlike DEM meters; C++ formatted coordinate string.
	let coords = format!(
		"'{},{},{}'",
		site.lon_deg,
		site.lat_deg,
		observer_z_m / 1000.0
	);
	// Request airless apparent azimuth/elevation at a fixed lunar site; C++ HTTP builder.
	let text = client
		.get("https://ssd.jpl.nasa.gov/api/horizons.api")
		.query(&[
			("format", "text"),
			("COMMAND", body),
			("CENTER", "'coord@301'"),
			("COORD_TYPE", "'GEODETIC'"),
			("SITE_COORD", &coords),
			("MAKE_EPHEM", "'YES'"),
			("EPHEM_TYPE", "'OBSERVER'"),
			("QUANTITIES", "'4'"),
			("ANG_FORMAT", "'DEG'"),
			("APPARENT", "'AIRLESS'"),
			("START_TIME", &format!("'{start}'")),
			("STOP_TIME", &format!("'{stop}'")),
			("STEP_SIZE", &format!("'{step}'")),
		])
		.send()?
		.error_for_status()?
		.text()?;
	// Retain reference radii and frame diagnostics in the preparation log; C++ filtered stream output.
	for line in text.lines().filter(|line| {
		line.contains("Center radii")
			|| line.contains("Center-site")
			|| line.contains("Center pole")
	})
	{
		println!("Horizons {body}: {line}");
	}
	// Parsing errors include the actual remote response; C++ expected error mapping.
	parse_horizons(&text)
		.map_err(|e| format!("Horizons for {} ({body}): {e}\n{text}", site.name).into())
}

// Native command entry point; C++ main with an error-returning implementation.
fn main() -> Result<(), Box<dyn Error>>
{
	let job = configuration()?;
	let args = &job.args;
	let height = job.height;
	let expected = job.expected;
	let sites = read_sites(&args[0])?;
	// Reuse caches across sites; transpose converts optional failure into operation failure; C++ optional/expected conversion.
	let mut terrain = args
		.get(5)
		.map(|p| dem::Terrain::open(Path::new(p)))
		.transpose()?;
	// Explicitly identify legacy runs without a DEM; C++ formatted diagnostic.
	println!(
		"Preparation: {expected} rows/body, observer height {height} m, terrain={}",
		terrain.is_some()
	);
	// Preserve the previous output default; C++ path from optional argument.
	let output = Path::new(args.get(4).map(String::as_str).unwrap_or("app/assets/data"));
	// Bound remote request time; C++ HTTP client timeout.
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(180))
		.build()?;
	// Create only the selected output directory; C++ filesystem::create_directories.
	fs::create_dir_all(output)?;
	// Process fixed sites rather than interactive map positions; C++ range-for over owned values.
	for site in sites
	{
		prepare_site(&client, site, &mut terrain, &job, output)?;
	}
	// Signal success to the shell; C++ return success.
	Ok(())
}

// Convert a label to the existing filename convention; C++ string utility.
fn slug(name: &str) -> String
{
	// Split punctuation runs, lowercase nonempty ASCII words, then join; C++ ranges pipeline.
	name.split(|c: char| !c.is_ascii_alphanumeric())
		.filter(|s| !s.is_empty())
		.map(str::to_ascii_lowercase)
		.collect::<Vec<_>>()
		.join("-")
}

struct Job
{
	args: Vec<String>,
	start: NaiveDate,
	seconds: i64,
	expected: usize,
	height: f64,
}

fn configuration() -> Result<Job, Box<dyn Error>>
{
	// Own arguments after the executable name; C++ vector<string> from argv.
	let args: Vec<_> = std::env::args().skip(1).collect();
	// Append DEM directory and height without breaking the prior CLI; C++ argument-count guard.
	if !(4..=7).contains(&args.len())
	{
		return Err("Usage: prep <sites.json> <start YYYY-MM-DD> <stop YYYY-MM-DD> <step, e.g. 1h> [output-dir] [dem-dir] [observer-height-m, default 2]".into());
	}
	let (start, seconds, expected) = timing(&args)?;
	// Instrument height is a nonnegative physical input; C++ optional argument parsing.
	let height = args.get(6).map_or(Ok(2.0), |s| s.parse::<f64>())?;
	if !height.is_finite() || height < 0.0
	{
		return Err("Observer height must be finite and nonnegative".into());
	}

	Ok(Job {
		args,
		start,
		seconds,
		expected,
		height,
	})
}

fn timing(args: &[String]) -> Result<(NaiveDate, i64, usize), Box<dyn Error>>
{
	// Parse inclusive UTC dates before making requests; C++ checked calendar parsing.
	let start = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d")?;
	let stop = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
	// Empty/reversed intervals are errors; C++ early return.
	if stop <= start
	{
		return Err("Stop date must be after start date".into());
	}
	// Validate the supported Horizons cadence syntax; C++ suffix check and integer parsing.
	let step = &args[3];
	if !step.ends_with(['m', 'h', 'd']) || step[..step.len() - 1].parse::<u32>().unwrap_or(0) == 0
	{
		return Err("Step must be a positive integer followed by m, h, or d".into());
	}
	// Convert cadence to elapsed seconds; C++ switch expression.
	let seconds = i64::from(step[..step.len() - 1].parse::<u32>()?)
		* match step.as_bytes()[step.len() - 1]
		{
			b'm' => 60,
			b'h' => 3600,
			_ => 86400,
		};
	// Count inclusive endpoints against the per-query service limit; C++ integer arithmetic.
	let expected = ((stop - start).num_seconds() / seconds + 1) as usize;
	if expected > 90000
	{
		return Err(format!("{expected} rows exceeds Horizons' 90,000-row limit").into());
	}
	Ok((start, seconds, expected))
}

fn read_sites(path: &str) -> Result<Vec<Site>, Box<dyn Error>>
{
	// Decode fixed sites before opening large rasters; C++ JSON parser.
	let sites: Vec<Site> = serde_json::from_str(&fs::read_to_string(path)?)?;
	if sites.is_empty()
	{
		return Err("At least one fixed site is required".into());
	}
	// Reject filename collisions as well as invalid coordinates; C++ unordered_set.
	let mut slugs = std::collections::HashSet::new();
	for site in &sites
	{
		if !site.lat_deg.is_finite()
			|| !(-90.0..=90.0).contains(&site.lat_deg)
			|| !site.lon_deg.is_finite()
			|| slug(&site.name).is_empty()
			|| !slugs.insert(slug(&site.name))
		{
			return Err(format!(
				"Invalid coordinates or duplicate/empty slug for {}",
				site.name
			)
			.into());
		}
	}
	Ok(sites)
}

fn prepare_site(
	client: &reqwest::blocking::Client,
	site: Site,
	terrain: &mut Option<dem::Terrain>,
	job: &Job,
	output: &Path,
) -> Result<(), Box<dyn Error>>
{
	let height = job.height;
	let (surface_height, horizon) = site_horizon(&site, terrain, height)?;
	let (sun, earth) = ephemerides(client, &site, surface_height + height, job)?;
	report_stats(&sun, &earth, horizon.as_ref())?;
	// Add provenance only for real terrain; C++ optional factory with error propagation.
	let provenance = horizon
		.as_ref()
		.map(|_| {
			Ok::<_, String>(TerrainInfo {
				observer_height_m: height,
				surface_height_m: surface_height,
				reference_radius_m: lunar_core::MOON_RADIUS_M,
				assets: dem::assets()?,
			})
		})
		.transpose()?;
	// Derive the existing stable ASCII basename; C++ path concatenation.
	let path = output.join(format!("{}.json", slug(&site.name)));
	// Transfer owned arrays into the serialized record; C++ aggregate moves.
	let data = SiteData {
		site,
		sun,
		earth,
		horizon,
		terrain: provenance,
	};
	// Pretty JSON is shared unchanged by native and web; C++ encoded file write.
	fs::write(&path, serde_json::to_string_pretty(&data)?)?;
	println!(
		"Wrote {} ({} samples per track, {} bytes)",
		path.display(),
		data.sun.t.len(),
		fs::metadata(&path)?.len()
	);
	Ok(())
}

fn site_horizon(
	site: &Site,
	terrain: &mut Option<dem::Terrain>,
	height: f64,
) -> Result<(f64, Option<Horizon>), Box<dyn Error>>
{
	// Legacy no-DEM preparation uses the reference sphere; C++ initialized local.
	let mut surface_height = 0.0;
	// Optional mutable borrowing gives the sampler exclusive cache access; C++ optional reference.
	let horizon = if let Some(dem) = terrain
	{
		// Missing observer data must not silently become flat terrain; C++ checked optional extraction.
		surface_height = dem
			.sample(site.lat_deg, site.lon_deg)?
			.ok_or("Site outside DEM coverage")?
			.z_m;
		// Include tile decoding and cold-cache work in wall time; C++ steady_clock.
		let timer = Instant::now();
		// Trace 1,440 bearings to available coverage edges, with no fixed production distance; C++ mutable lambda.
		let h = horizon_from_terrain(
			site.lat_deg,
			site.lon_deg,
			1440,
			height,
			None,
			|lat, lon| dem.sample(lat, lon),
		)?;
		// Report exact extrema and a one-site elapsed benchmark; C++ reductions and stream output.
		println!(
			"Horizon {}: {} azimuths, surface {:.6} m, min {:.6} deg, max {:.6} deg, benchmark {:.3} s",
			site.name,
			h.h_deg.len(),
			surface_height,
			h.h_deg.iter().copied().fold(f32::INFINITY, f32::min),
			h.h_deg.iter().copied().fold(f32::NEG_INFINITY, f32::max),
			timer.elapsed().as_secs_f64()
		);
		Some(h)
	}
	else
	{
		None
	};
	Ok((surface_height, horizon))
}

fn ephemerides(
	client: &reqwest::blocking::Client,
	site: &Site,
	observer_z: f64,
	job: &Job,
) -> Result<(Track, Track), Box<dyn Error>>
{
	let args = &job.args;
	let start = job.start;
	let seconds = job.seconds;
	let expected = job.expected;
	let step = &args[3];
	// Query both bodies at the actual surface plus instrument altitude; C++ common scalar parameter.
	let sun = download(client, site, observer_z, "'10'", &args[1], &args[2], step)?;
	let earth = download(client, site, observer_z, "'399'", &args[1], &args[2], step)?;
	// A UTC date endpoint is midnight in the service's sample grid; C++ chrono conversion.
	let first = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
	// Verify count, endpoints, cadence and exact inter-body UTC alignment; C++ vector predicates.
	if sun.t != earth.t
		|| sun.t.len() != expected
		|| sun.t[0] != first
		|| *sun.t.last().unwrap() != first + (expected as i64 - 1) * seconds
		|| sun.t.windows(2).any(|t| t[1] - t[0] != seconds)
	{
		return Err("Unexpected Horizons row counts or mismatched timestamps".into());
	}
	Ok((sun, earth))
}

fn report_stats(sun: &Track, earth: &Track, horizon: Option<&Horizon>)
-> Result<(), Box<dyn Error>>
{
	// Only the legacy no-DEM path has a flat fallback; C++ local profile.
	let flat = Horizon { h_deg: vec![0.0] };
	let h = horizon.unwrap_or(&flat);
	// Evaluate visible solar area at every sample; C++ zipped transform.
	let fractions: Vec<_> = sun
		.el_deg
		.iter()
		.zip(&sun.az_deg)
		.map(|(&el, &az)| sun_fraction(el, h.at(az)))
		.collect();
	// stats requires booleans: report any visible area, not average power; C++ predicate transform.
	let any_sun: Vec<_> = fractions.iter().map(|&f| f > 0.0).collect();
	// Report both allowed communication-margin endpoints; C++ range-for.
	for margin in [EARTH_MARGIN_DEG, 1.0]
	{
		println!(
			"Stats (Sun area > 0, Earth margin {margin} deg): {}",
			serde_json::to_string(&stats(&sun.t, &any_sun, &visible(earth, h, margin)))?
		);
	}
	// Match stats' left-held weighting and durationless final sample; C++ weighted sum.
	let mean_area: f64 = if sun.t.len() < 2
	{
		0.0
	}
	else
	{
		sun.t
			.windows(2)
			.zip(&fractions)
			.map(|(t, &f)| (t[1] - t[0]) as f64 * f64::from(f))
			.sum::<f64>()
			/ (sun.t.last().unwrap() - sun.t[0]) as f64
	};
	println!("Mean visible solar area: {:.6}%", mean_area * 100.0);
	Ok(())
}
