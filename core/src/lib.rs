//! I/O-free lunar geometry, terrain horizons, visibility and ephemeris conversion.

pub mod ephemeris;

pub mod anypoint;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// PGDA's spherical reference radius in meters.
pub const MOON_RADIUS_M: f64 = 1_737_400.0;
/// Fixed apparent solar radius used for the inexpensive disk-area approximation.
pub const SUN_RADIUS_DEG: f32 = 0.267;
/// Default angular clearance for direct-to-Earth links.
pub const EARTH_MARGIN_DEG: f32 = 0.5;

#[derive(Debug, Serialize, Deserialize)]
pub struct Site
{
	pub name: String,
	/// Planetocentric latitude, north positive.
	pub lat_deg: f64,
	/// East-positive longitude.
	pub lon_deg: f64,
}

/// UTC Unix seconds and angles in degrees. Azimuth is north, clockwise to east.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track
{
	/// UTC Unix seconds, monotonically increasing.
	pub t: Vec<i64>,
	/// Geographic azimuth north clockwise.
	pub az_deg: Vec<f32>,
	/// Local tangent-plane elevation.
	pub el_deg: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Horizon
{
	/// Uniform cyclic bearing bins, beginning at north.
	pub h_deg: Vec<f32>,
}

impl Horizon
{
	/// An empty profile represents an unobstructed horizon below all elevations.
	pub fn at(&self, az_deg: f32) -> f32
	{
		// Preserve the unobstructed sentinel for legacy scientific callers.
		if self.h_deg.is_empty()
		{
			// No surface reaches this elevation.
			return -90.0;
		}
		// Euclidean remainder wraps negative and greater-than-360 bearings.
		let p = az_deg.rem_euclid(360.0) / 360.0 * self.h_deg.len() as f32;
		let i = p.floor() as usize % self.h_deg.len();
		let a = self.h_deg[i];
		let b = self.h_deg[(i + 1) % self.h_deg.len()];
		// Piecewise-linear interpolation includes the last-to-first wrap.
		a + (b - a) * p.fract()
	}
}

pub struct Dem
{
	pub w: usize,
	pub h: usize,
	/// Projected spacing in meters.
	pub res_m: f64,
	/// First sample center, not the corner of a GeoTIFF pixel.
	pub x_min_m: f64,
	pub y_max_m: f64,
	/// Row-major radial heights above the reference sphere.
	pub z_m: Vec<f32>,
}

/// PGDA south polar stereographic: spherical radius R, true scale at -90,
/// longitude origin zero, no false offsets; +x toward 90E, +y toward 0E.
pub fn to_xy(lat_deg: f64, lon_deg: f64) -> (f64, f64)
{
	// South-pole true scale gives rho=2R tan(pi/4+latitude/2).
	let rho =
		2.0 * MOON_RADIUS_M * (std::f64::consts::FRAC_PI_4 + lat_deg.to_radians() / 2.0).tan();
	let lon = lon_deg.to_radians();
	(rho * lon.sin(), rho * lon.cos())
}

impl Dem
{
	/// Grid coordinates refer to sample centers, with rows running toward decreasing y.
	pub fn height_at(&self, x: f64, y: f64) -> Option<f32>
	{
		if self.w == 0
			|| self.h == 0
			|| !self.res_m.is_finite()
			|| self.res_m <= 0.0
			|| self.w.checked_mul(self.h)? != self.z_m.len()
		{
			return None;
		}
		let col = (x - self.x_min_m) / self.res_m;
		let row = (self.y_max_m - y) / self.res_m;
		// Do not extrapolate or cast invalid coordinates into indices.
		if !col.is_finite()
			|| !row.is_finite()
			|| col < 0.0
			|| row < 0.0
			|| col > (self.w - 1) as f64
			|| row > (self.h - 1) as f64
		{
			return None;
		}
		let c = col.floor() as usize;
		let r = row.floor() as usize;
		let c1 = (c + 1).min(self.w - 1);
		let r1 = (r + 1).min(self.h - 1);
		let fx = col.fract() as f32;
		let fy = row.fract() as f32;
		let top = self.z_m[r * self.w + c] * (1.0 - fx) + self.z_m[r * self.w + c1] * fx;
		let bottom = self.z_m[r1 * self.w + c] * (1.0 - fx) + self.z_m[r1 * self.w + c1] * fx;
		let z = top * (1.0 - fy) + bottom * fy;
		z.is_finite().then_some(z)
	}
}

/// Compatibility wrapper: zero observer height and an explicit distance limit.
pub fn horizon_from_dem(dem: &Dem, lat: f64, lon: f64, n_az: usize, max_dist_m: f64) -> Horizon
{
	// Adapt a memory grid to the fallible sampler without introducing I/O.
	horizon_from_terrain(lat, lon, n_az, 0.0, Some(max_dist_m), |lat, lon| {
		// Project each spherical destination, not a straight map-grid ray.
		let (x, y) = to_xy(lat, lon);
		Ok(dem.height_at(x, y).map(|z| TerrainSample {
			z_m: f64::from(z),
			resolution_m: dem.res_m,
		}))
	})
	// Preserve the old empty/missing-data convention only in this compatibility API.
	.unwrap_or_else(|_| Horizon {
		h_deg: vec![-90.0; n_az],
	})
}

/// A sampled surface height and local ground pixel spacing.
pub struct TerrainSample
{
	/// Radial elevation above the reference sphere, not absolute radius.
	pub z_m: f64,
	/// Ground spacing allows the integrator to retain half-pixel radial sampling.
	pub resolution_m: f64,
}

/// Great-circle destination with a geographic north/east tangent basis, also at the pole.
pub fn destination(lat: f64, lon: f64, az: f64, distance_m: f64) -> (f64, f64)
{
	GreatCircle::new(lat, lon, az).destination(distance_m)
}

struct GreatCircle
{
	origin: [f64; 3],
	tangent: [f64; 3],
}

impl GreatCircle
{
	fn new(lat: f64, lon: f64, az: f64) -> Self
	{
		let (sp, cp) = lat.to_radians().sin_cos();
		let (sl, cl) = lon.to_radians().sin_cos();
		let (sa, ca) = az.to_radians().sin_cos();
		Self {
			origin: [cp * cl, cp * sl, sp],
			tangent: [-sp * cl * ca - sl * sa, -sp * sl * ca + cl * sa, cp * ca],
		}
	}

	fn xyz(&self, distance_m: f64) -> [f64; 3]
	{
		let (sb, cb) = (distance_m / MOON_RADIUS_M).sin_cos();
		std::array::from_fn(|i| self.origin[i] * cb + self.tangent[i] * sb)
	}

	fn destination(&self, distance_m: f64) -> (f64, f64)
	{
		let [x, y, z] = self.xyz(distance_m);
		(z.atan2(x.hypot(y)).to_degrees(), y.atan2(x).to_degrees())
	}
}

/// Exact radial-tangent elevation in degrees; observer_z includes instrument height.
pub fn spherical_elevation(z: f64, observer_z: f64, beta: f64) -> f64
{
	let (vertical, horizontal) = spherical_components(z, observer_z, beta);
	vertical.atan2(horizontal).to_degrees()
}

fn spherical_components(z: f64, observer_z: f64, beta: f64) -> (f64, f64)
{
	// Rewrite cos(beta)-1 to avoid cancellation for meter-scale baselines.
	let vertical = z - observer_z - 2.0 * (MOON_RADIUS_M + z) * (beta / 2.0).sin().powi(2);
	(vertical, (MOON_RADIUS_M + z) * beta.sin())
}

/// Sample 0..360 geographic bearings to the first unavailable point on each ray.
/// The sampler can live in prep and perform I/O; this crate only evaluates geometry.
pub fn horizon_from_terrain(
	lat: f64,
	lon: f64,
	n_az: usize,
	observer_height_m: f64,
	max_distance_m: Option<f64>,
	mut sample: impl FnMut(f64, f64) -> Result<Option<TerrainSample>, String>,
) -> Result<Horizon, String>
{
	// Reject invalid integration inputs instead of producing plausible empty terrain.
	if !lat.is_finite()
		|| !(-90.0..=90.0).contains(&lat)
		|| !lon.is_finite()
		|| n_az == 0
		|| !observer_height_m.is_finite()
		|| observer_height_m < 0.0
		|| max_distance_m.is_some_and(|v| !v.is_finite() || v <= 0.0)
	{
		return Err("Invalid horizon sampling parameters".into());
	}
	let (z0, near) = horizon_origin(lat, lon, observer_height_m, &mut sample)?;
	// A quarter bearing-bin width is the adaptive angular target.
	let angular = std::f64::consts::TAU / n_az as f64 / 4.0;
	// A ray never wraps around the Moon; normal production termination is coverage, not this guard.
	let limit = max_distance_m.unwrap_or(std::f64::consts::PI * MOON_RADIUS_M);
	let mut h_deg = vec![-90.0_f32; n_az];
	for (i, h) in h_deg.iter_mut().enumerate()
	{
		let az = i as f64 * 360.0 / n_az as f64;
		*h = terrain_ray(
			HorizonRay {
				lat,
				lon,
				z0,
				near,
				limit,
				angular,
			},
			az,
			&mut sample,
		)?;
	}
	Ok(Horizon { h_deg })
}

pub fn sun_fraction(el: f32, h: f32) -> f32
{
	// Normalize the horizon's chord offset by the apparent solar radius.
	let u = ((f64::from(h) - f64::from(el)) / f64::from(SUN_RADIUS_DEG)).clamp(-1.0, 1.0);
	// Integrate the visible circular segment, not the visible diameter.
	((u.acos() - u * (1.0 - u * u).sqrt()) / std::f64::consts::PI).clamp(0.0, 1.0) as f32
}

pub fn visible(track: &Track, horizon: &Horizon, margin_deg: f32) -> Vec<bool>
{
	// Parallel arrays are an explicit scientific precondition.
	assert_eq!(track.t.len(), track.az_deg.len());
	assert_eq!(track.t.len(), track.el_deg.len());
	track
		.az_deg
		.iter()
		.zip(&track.el_deg)
		.map(|(&az, &el)| el > horizon.at(az) + margin_deg)
		.collect()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats
{
	pub pct_sun: f64,
	pub longest_night_h: f64,
	pub pct_earth: f64,
	pub pct_both: f64,
}

/// Time-weighted percentages using boolean visibility held over [t[i], t[i+1]).
/// Gaps count in full; the final sample has no duration. Empty/single series return zeros.
/// Panics for unequal lengths or timestamps that are not strictly increasing.
pub fn stats(t: &[i64], sun_vis: &[bool], earth_vis: &[bool]) -> Stats
{
	assert_eq!(t.len(), sun_vis.len());
	assert_eq!(t.len(), earth_vis.len());
	let mut result = Stats::default();
	let mut night: f64 = 0.0;
	let mut total = 0.0;
	// Integrate half-open intervals, never invent a duration for the last point.
	for (i, pair) in t.windows(2).enumerate()
	{
		assert!(pair[1] > pair[0]);
		// Cast before subtraction to avoid signed-integer overflow.
		let dt = (pair[1] as f64 - pair[0] as f64) / 3600.0;
		total += dt;
		// Visibility at the left endpoint is held across this interval.
		if sun_vis[i]
		{
			result.pct_sun += dt;
			night = 0.0;
		}
		else
		{
			night += dt;
			result.longest_night_h = result.longest_night_h.max(night);
		}
		if earth_vis[i]
		{
			result.pct_earth += dt;
		}
		if sun_vis[i] && earth_vis[i]
		{
			result.pct_both += dt;
		}
	}
	if total > 0.0
	{
		result.pct_sun *= 100.0 / total;
		result.pct_earth *= 100.0 / total;
		result.pct_both *= 100.0 / total;
	}
	result
}

pub fn parse_horizons(text: &str) -> Result<Track, String>
{
	// Isolate only the service's marked numeric ephemeris block.
	let (_, rest) = text.split_once("$$SOE").ok_or("Missing $$SOE")?;
	let (body, _) = rest.split_once("$$EOE").ok_or("Missing $$EOE")?;
	let mut track = Track::default();
	for line in body.lines().filter(|line| !line.trim().is_empty())
	{
		let (time, az, el) = horizons_row(line)?;
		if track.t.last().is_some_and(|&last| time <= last)
		{
			return Err("Timestamps are not increasing".into());
		}
		track.t.push(time);
		track.az_deg.push(az);
		track.el_deg.push(el);
	}
	// Empty marked output is an error rather than an apparently successful download.
	if track.t.is_empty()
	{
		return Err("Empty ephemeris".into());
	}
	Ok(track)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod terrain_tests;

fn horizons_row(line: &str) -> Result<(i64, f32, f32), String>
{
	// Whitespace tokenization also accepts optional printed event flags.
	let fields: Vec<_> = line.split_whitespace().collect();
	// Need a date, time, azimuth and elevation.
	if fields.len() < 4
	{
		return Err(format!("Invalid ephemeris row: {line}"));
	}
	// Accept both minute and second precision in Horizons UTC output.
	let time =
		NaiveDateTime::parse_from_str(&format!("{} {}", fields[0], fields[1]), "%Y-%b-%d %H:%M")
			.or_else(|_| {
				NaiveDateTime::parse_from_str(
					&format!("{} {}", fields[0], fields[1]),
					"%Y-%b-%d %H:%M:%S",
				)
			})
			.map_err(|e| format!("Invalid date: {e}"))?
			.and_utc()
			.timestamp();
	// The requested two angles are the last two numeric columns, after any event flags.
	let mut angles = fields[2..]
		.iter()
		.rev()
		.filter_map(|s| s.parse::<f32>().ok());
	// Reverse order yields elevation first, then azimuth.
	let el = angles.next();
	let az = angles.next();
	// A successful row must contain both values.
	let (Some(az), Some(el)) = (az, el)
	else
	{
		return Err(format!("Expected azimuth and elevation: {line}"));
	};
	if !az.is_finite() || !el.is_finite()
	{
		return Err(format!("Expected finite azimuth and elevation: {line}"));
	}
	Ok((time, az, el))
}

fn terrain_ray(
	ray: HorizonRay,
	az: f64,
	sample: &mut impl FnMut(f64, f64) -> Result<Option<TerrainSample>, String>,
) -> Result<f32, String>
{
	let path = GreatCircle::new(ray.lat, ray.lon, az);
	terrain_ray_samples(ray, az, &mut |s| {
		let (lat, lon) = path.destination(s);
		sample(lat, lon)
	})
}

fn terrain_ray_samples(
	ray: HorizonRay,
	az: f64,
	sample: &mut impl FnMut(f64) -> Result<Option<TerrainSample>, String>,
) -> Result<f32, String>
{
	let HorizonRay {
		z0,
		near,
		limit,
		angular,
		..
	} = ray;
	let mut best: Option<(f64, f64)> = None;
	// Start off the observer so atan2 never evaluates the coincident point.
	let mut s = near;
	// Trace until the requested compatibility limit or continuous DEM edge.
	while s <= limit
	{
		// Propagate decoding errors; only absent data terminates a ray.
		let Some(point) = sample(s)?
		else
		{
			break;
		};
		// A malformed sample is an error, never an artificial horizon gap.
		if !point.z_m.is_finite()
			|| point.z_m <= -MOON_RADIUS_M
			|| !point.resolution_m.is_finite()
			|| point.resolution_m <= 0.0
		{
			return Err("Invalid terrain sample".into());
		}
		// Horizontal components are positive before the antipode; compare slopes without atan2.
		let (vertical, horizontal) = spherical_components(point.z_m, z0, s / MOON_RADIUS_M);
		if best.is_none_or(|(v, h)| {
			if h > 0.0 && horizontal > 0.0
			{
				vertical * h > v * horizontal
			}
			else
			{
				vertical.atan2(horizontal) > v.atan2(h)
			}
		})
		{
			best = Some((vertical, horizontal));
		}
		// Grow only up to half a local pixel: do not skip resolved narrow radial features.
		let next = s + (s * angular).max(near).min(point.resolution_m / 2.0);
		// A broken callback must not stall a finite ray forever.
		if next <= s
		{
			return Err("Terrain spacing is below floating-point resolution".into());
		}
		// Advance only after validating floating-point progress.
		s = next;
	}
	let (vertical, horizontal) =
		best.ok_or_else(|| format!("No terrain samples at azimuth {az}"))?;
	Ok(vertical.atan2(horizontal).to_degrees() as f32)
}

#[derive(Clone, Copy)]
struct HorizonRay
{
	lat: f64,
	lon: f64,
	z0: f64,
	near: f64,
	limit: f64,
	angular: f64,
}

fn horizon_origin(
	lat: f64,
	lon: f64,
	observer_height_m: f64,
	sample: &mut impl FnMut(f64, f64) -> Result<Option<TerrainSample>, String>,
) -> Result<(f64, f64), String>
{
	// A missing observer invalidates the whole calculation.
	let origin = sample(lat, lon)?.ok_or("Observer is outside valid DEM coverage")?;
	if !origin.z_m.is_finite()
		|| origin.z_m <= -MOON_RADIUS_M
		|| !origin.resolution_m.is_finite()
		|| origin.resolution_m <= 0.0
	{
		return Err("Invalid observer terrain sample".into());
	}
	// The instrument is above the local radial surface, not above a flat zero plane.
	let z0 = origin.z_m + observer_height_m;
	// Use half the finest starting ground pixel near the observer.
	let near = origin.resolution_m / 2.0;
	// Refuse overflow or a spacing rounded to zero before entering the integration loop.
	if !z0.is_finite() || near <= 0.0
	{
		return Err("Invalid observer height or spacing".into());
	}
	Ok((z0, near))
}
