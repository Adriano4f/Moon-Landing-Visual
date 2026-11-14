use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub const MOON_RADIUS_M: f64 = 1_737_400.0;
pub const SUN_RADIUS_DEG: f32 = 0.267;

#[derive(Debug, Serialize, Deserialize)]
pub struct Site
{
	pub name: String,
	pub lat_deg: f64,
	pub lon_deg: f64,
}

/// UTC Unix seconds and angles in degrees. Azimuth is north, clockwise to east.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track
{
	pub t: Vec<i64>,
	pub az_deg: Vec<f32>,
	pub el_deg: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Horizon
{
	pub h_deg: Vec<f32>,
}

impl Horizon
{
	/// An empty profile represents an unobstructed horizon below all elevations.
	pub fn at(&self, az_deg: f32) -> f32
	{
		if self.h_deg.is_empty()
		{
			return -90.0;
		}
		let p = az_deg.rem_euclid(360.0) / 360.0 * self.h_deg.len() as f32;
		let i = p.floor() as usize % self.h_deg.len();
		let a = self.h_deg[i];
		let b = self.h_deg[(i + 1) % self.h_deg.len()];
		a + (b - a) * p.fract()
	}
}

pub struct Dem
{
	pub w: usize,
	pub h: usize,
	pub res_m: f64,
	pub x_min_m: f64,
	pub y_max_m: f64,
	pub z_m: Vec<f32>,
}

/// South polar stereographic. Check this sign/orientation convention against the DEM label.
pub fn to_xy(lat_deg: f64, lon_deg: f64) -> (f64, f64)
{
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

/// Rays follow local north/east, rotated into the polar grid by site longitude.
/// Missing terrain (including an outside-grid site) leaves a bin at -90 degrees.
pub fn horizon_from_dem(dem: &Dem, lat: f64, lon: f64, n_az: usize, max_dist_m: f64) -> Horizon
{
	let mut horizon = Horizon {
		h_deg: vec![-90.0; n_az],
	};
	let (x, y) = to_xy(lat, lon);
	let Some(z0) = dem.height_at(x, y)
	else
	{
		return horizon;
	};
	let step = dem.res_m / 2.0;
	if !max_dist_m.is_finite()
	{
		return horizon;
	}
	for (i, h) in horizon.h_deg.iter_mut().enumerate()
	{
		let angle = (i as f64 * 360.0 / n_az as f64 + lon).to_radians();
		let mut s = step;
		while s <= max_dist_m
		{
			let Some(z) = dem.height_at(x + s * angle.sin(), y + s * angle.cos())
			else
			{
				break;
			};
			let dz = f64::from(z - z0) - s * s / (2.0 * MOON_RADIUS_M);
			*h = h.max((dz / s).atan().to_degrees() as f32);
			s += step;
		}
	}
	horizon
}

pub fn sun_fraction(el: f32, h: f32) -> f32
{
	((el - h + SUN_RADIUS_DEG) / (2.0 * SUN_RADIUS_DEG)).clamp(0.0, 1.0)
}

pub fn visible(track: &Track, horizon: &Horizon, margin_deg: f32) -> Vec<bool>
{
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
	for (i, pair) in t.windows(2).enumerate()
	{
		assert!(pair[1] > pair[0]);
		let dt = (pair[1] as f64 - pair[0] as f64) / 3600.0;
		total += dt;
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
	let (_, rest) = text.split_once("$$SOE").ok_or("Missing $$SOE")?;
	let (body, _) = rest.split_once("$$EOE").ok_or("Missing $$EOE")?;
	let mut track = Track::default();
	for line in body.lines().filter(|line| !line.trim().is_empty())
	{
		let fields: Vec<_> = line.split_whitespace().collect();
		if fields.len() < 4
		{
			return Err(format!("Invalid ephemeris row: {line}"));
		}
		let time = NaiveDateTime::parse_from_str(
			&format!("{} {}", fields[0], fields[1]),
			"%Y-%b-%d %H:%M",
		)
		.or_else(|_| {
			NaiveDateTime::parse_from_str(
				&format!("{} {}", fields[0], fields[1]),
				"%Y-%b-%d %H:%M:%S",
			)
		})
		.map_err(|e| format!("Invalid date: {e}"))?
		.and_utc()
		.timestamp();
		let mut angles = fields[2..]
			.iter()
			.rev()
			.filter_map(|s| s.parse::<f32>().ok());
		let el = angles.next();
		let az = angles.next();
		let (Some(az), Some(el)) = (az, el)
		else
		{
			return Err(format!("Expected azimuth and elevation: {line}"));
		};
		if !az.is_finite() || !el.is_finite()
		{
			return Err(format!("Expected finite azimuth and elevation: {line}"));
		}
		if track.t.last().is_some_and(|&last| time <= last)
		{
			return Err("Timestamps are not increasing".into());
		}
		track.t.push(time);
		track.az_deg.push(az);
		track.el_deg.push(el);
	}
	if track.t.is_empty()
	{
		return Err("Empty ephemeris".into());
	}
	Ok(track)
}

#[cfg(test)]
mod tests;
