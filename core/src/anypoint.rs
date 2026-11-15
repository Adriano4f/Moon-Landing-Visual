//! Runtime Moon Trek terrain, with no network or filesystem access.
//!
//! Fetch every URL from `terrain_requests`, preserving order, then advance a
//! `HorizonJob` a few bearings per frame. Heights are meters above 1737.4 km.
//! Rays stop at 250 km or the first coverage gap. Polar fine coverage uses
//! 5, 10 or 30 m LOLA grids; global fine coverage is roughly 118 m.

use crate::{
	GreatCircle, Horizon, HorizonRay, MOON_RADIUS_M, TerrainSample, horizon_origin, terrain_ray,
	terrain_ray_samples, to_xy,
};

pub const N_AZ: usize = 720;
const SIZE: usize = 1024;
const LIMIT_M: f64 = 250_000.0;
const SERVICE: &str = "https://trek.nasa.gov/moon/trekarcgis/rest/services";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile
{
	pub lat_deg: f64,
	pub lon_deg: f64,
	pub half_lat_deg: f64,
	pub half_lon_deg: f64,
	pub id: u32,
}

fn cells(band: u32) -> u32
{
	(180.0 * (-89.0 + 2.0 * f64::from(band)).to_radians().cos())
		.round()
		.max(1.0) as u32
}

/// Nonfinite coordinates map to zero; latitude is clamped and longitude wraps.
pub fn tile_of(lat_deg: f64, lon_deg: f64) -> Tile
{
	let lat = if lat_deg.is_finite()
	{
		lat_deg.clamp(-90.0, 90.0)
	}
	else
	{
		0.0
	};
	let lon = if lon_deg.is_finite()
	{
		lon_deg.rem_euclid(360.0)
	}
	else
	{
		0.0
	};
	let band = (((lat + 90.0) / 2.0).floor() as u32).min(89);
	let n = cells(band);
	let col = ((lon / 360.0 * f64::from(n)).floor() as u32).min(n - 1);
	Tile {
		lat_deg: -89.0 + 2.0 * f64::from(band),
		lon_deg: (f64::from(col) + 0.5) * 360.0 / f64::from(n),
		half_lat_deg: 1.0,
		half_lon_deg: 180.0 / f64::from(n),
		id: (0..band).map(cells).sum::<u32>() + col,
	}
}

pub fn tile_count() -> u32
{
	(0..90).map(cells).sum()
}

#[derive(Clone, Copy, PartialEq)]
enum Projection
{
	Geographic,
	South,
	North,
}

impl Projection
{
	fn xy(self, lat: f64, lon: f64) -> (f64, f64)
	{
		match self
		{
			Self::Geographic => ((lon + 180.0).rem_euclid(360.0) - 180.0, lat),
			Self::South => to_xy(lat, lon),
			Self::North =>
			{
				let (x, y) = to_xy(-lat, lon);
				(x, -y)
			}
		}
	}
}

#[derive(Clone, Copy)]
struct Window
{
	projection: Projection,
	bbox: [f64; 4],
	service: &'static str,
}

fn polar_fine(lat: f64, lon: f64) -> Option<Window>
{
	let south = lat < 0.0;
	let projection = if south
	{
		Projection::South
	}
	else
	{
		Projection::North
	};
	let (x, y) = projection.xy(lat, lon);
	let five = if south
	{
		"LRO_LOLA_DEM_SPole875_5mp_v04"
	}
	else
	{
		"LRO_LOLA_DEM_NPole875_5mp_v04"
	};
	let thirty = if south
	{
		"LRO_LOLA_DEM_SPole75_30mp_v04"
	}
	else
	{
		"LRO_LOLA_DEM_NPole75_30mp_v04"
	};
	let (service, radius) = if x.abs().max(y.abs()) + 2_560.0 <= 75_840.0
	{
		(five, 2_560.0)
	}
	else if south && x.abs().max(y.abs()) + 5_120.0 <= 151_680.0
	{
		("ldem_85s_10m_Spole_float_SP", 5_120.0)
	}
	else if x.abs().max(y.abs()) <= 457_440.0
	{
		(thirty, 15_360.0)
	}
	else
	{
		return None;
	};
	Some(Window {
		projection,
		bbox: [x - radius, y - radius, x + radius, y + radius],
		service,
	})
}

fn windows(lat: f64, lon: f64) -> Vec<Window>
{
	if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) || !lon.is_finite()
	{
		return Vec::new();
	}
	let projection = if lat >= 80.0
	{
		Projection::North
	}
	else if lat <= -80.0
	{
		Projection::South
	}
	else
	{
		Projection::Geographic
	};
	let (x, y) = projection.xy(lat, lon);
	let mut result = Vec::new();
	for (level, radius) in [15_360.0_f64, 300_000.0].into_iter().enumerate()
	{
		if level == 0
			&& lat.abs() >= 75.0
			&& let Some(fine) = polar_fine(lat, lon)
		{
			result.push(fine);
			continue;
		}
		let service = match projection
		{
			Projection::Geographic => "LRO_LOLA_DEM_Global_256ppd_v06",
			Projection::South => "LRO_LOLA_DEM_SPole75_30mp_v04",
			Projection::North => "LRO_LOLA_DEM_NPole75_30mp_v04",
		};
		if matches!(projection, Projection::Geographic)
		{
			// The global source has 118 m north/south pixels; avoid oversampling it.
			let dy = (radius.max(60_000.0) / MOON_RADIUS_M).to_degrees();
			let dx = dy / lat.to_radians().cos();
			let lo = (y - dy).max(-90.0);
			let hi = (y + dy).min(90.0);
			let left = x - dx;
			let right = x + dx;
			result.push(Window {
				projection,
				bbox: [left.max(-180.0), lo, right.min(180.0), hi],
				service,
			});
			if left < -180.0
			{
				result.push(Window {
					projection,
					bbox: [left + 360.0, lo, 180.0, hi],
					service,
				});
			}
			if right > 180.0
			{
				result.push(Window {
					projection,
					bbox: [-180.0, lo, right - 360.0, hi],
					service,
				});
			}
		}
		else
		{
			result.push(Window {
				projection,
				bbox: [x - radius, y - radius, x + radius, y + radius],
				service,
			});
		}
	}
	result
}

/// Fine coverage first, then coarse; seam-crossing windows use two requests.
/// Invalid coordinates return no requests. Each response is at most 1024² F32.
pub fn terrain_requests(lat_deg: f64, lon_deg: f64) -> Vec<String>
{
	windows(lat_deg, lon_deg).iter().map(|w| {
		let name = w.service;
		let [a, b, c, d] = w.bbox;
		format!("{SERVICE}/{name}/ImageServer/exportImage?f=image&bbox={a}%2C{b}%2C{c}%2C{d}&size={SIZE}%2C{SIZE}&adjustAspectRatio=false&format=tiff&pixelType=F32&compression=none&renderingRule=%7B%22rasterFunction%22%3A%22None%22%7D&interpolation=RSP_BilinearInterpolation")
	}).collect()
}

struct Raster
{
	window: Window,
	w: usize,
	h: usize,
	values: Vec<f32>,
	nodata: Option<f32>,
}

impl Raster
{
	fn sample(&self, lat: f64, lon: f64) -> Option<TerrainSample>
	{
		let (x, y) = self.window.projection.xy(lat, lon);
		self.sample_xy(x, y)
	}

	fn sample_xy(&self, mut x: f64, y: f64) -> Option<TerrainSample>
	{
		let [left, bottom, right, top] = self.window.bbox;
		if matches!(self.window.projection, Projection::Geographic) && x == -180.0 && right == 180.0
		{
			x = 180.0;
		}
		let dx = (right - left) / self.w as f64;
		let dy = (top - bottom) / self.h as f64;
		let c = (x - left) / dx - 0.5;
		let r = (top - y) / dy - 0.5;
		// Permit the half-pixel border, but never extrapolate past the requested box.
		if !c.is_finite() || !r.is_finite() || x < left || x > right || y < bottom || y > top
		{
			return None;
		}
		let c = c.clamp(0.0, (self.w - 1) as f64);
		let r = r.clamp(0.0, (self.h - 1) as f64);
		let (c0, r0) = (c.floor() as usize, r.floor() as usize);
		let (c1, r1) = ((c0 + 1).min(self.w - 1), (r0 + 1).min(self.h - 1));
		let mut z = 0.0;
		for (index, weight) in [
			(r0 * self.w + c0, (1.0 - c.fract()) * (1.0 - r.fract())),
			(r0 * self.w + c1, c.fract() * (1.0 - r.fract())),
			(r1 * self.w + c0, (1.0 - c.fract()) * r.fract()),
			(r1 * self.w + c1, c.fract() * r.fract()),
		]
		{
			if weight > 0.0
			{
				let value = self.values[index];
				if !value.is_finite() || self.nodata == Some(value) || value.abs() > 100_000.0
				{
					return None;
				}
				z += f64::from(value) * weight;
			}
		}
		let resolution_m = match self.window.projection
		{
			Projection::Geographic => dy.to_radians() * MOON_RADIUS_M,
			_ => dx.min(dy) / (1.0 + (x * x + y * y) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M)),
		};
		Some(TerrainSample {
			z_m: z,
			resolution_m,
		})
	}
}

fn sample(rasters: &[Raster], lat: f64, lon: f64) -> Result<Option<TerrainSample>, String>
{
	Ok(rasters.iter().find_map(|r| r.sample(lat, lon)))
}

pub struct HorizonJob
{
	rasters: Vec<Raster>,
	ray: HorizonRay,
	surface: f64,
	h_deg: Vec<f32>,
	next: usize,
}

impl HorizonJob
{
	pub fn new(
		lat_deg: f64,
		lon_deg: f64,
		n_az: usize,
		observer_height_m: f64,
		tiffs: &[Vec<u8>],
	) -> Result<Self, String>
	{
		let windows = windows(lat_deg, lon_deg);
		if windows.is_empty()
			|| tiffs.len() != windows.len()
			|| n_az == 0
			|| n_az > 65_536
			|| !observer_height_m.is_finite()
			|| observer_height_m < 0.0
		{
			return Err("Invalid terrain job parameters or TIFF count".into());
		}
		let rasters = windows
			.into_iter()
			.zip(tiffs)
			.map(|(window, bytes)| decode(bytes, window))
			.collect::<Result<Vec<_>, _>>()?;
		let (z0, near) = horizon_origin(lat_deg, lon_deg, observer_height_m, &mut |p, l| {
			sample(&rasters, p, l)
		})?;
		Ok(Self {
			rasters,
			ray: HorizonRay {
				lat: lat_deg,
				lon: lon_deg,
				z0,
				near,
				limit: LIMIT_M,
				angular: std::f64::consts::TAU / n_az as f64 / 4.0,
			},
			surface: z0 - observer_height_m,
			h_deg: vec![-90.0; n_az],
			next: 0,
		})
	}

	pub fn surface_m(&self) -> f64
	{
		self.surface
	}

	/// Completed jobs return the same profile again; zero bins only polls completion.
	pub fn step(&mut self, bins: usize) -> Result<Option<Horizon>, String>
	{
		let end = self.next.saturating_add(bins).min(self.h_deg.len());
		while self.next < end
		{
			let az = self.next as f64 * 360.0 / self.h_deg.len() as f64;
			let projection = self.rasters[0].window.projection;
			let uniform = self
				.rasters
				.iter()
				.all(|r| r.window.projection == projection);
			self.h_deg[self.next] = match (projection, uniform)
			{
				(Projection::Geographic, _) | (_, false) =>
				{
					terrain_ray(self.ray, az, &mut |p, l| sample(&self.rasters, p, l))?
				}
				(projection, true) =>
				{
					let path = GreatCircle::new(self.ray.lat, self.ray.lon, az);
					let sign = if matches!(projection, Projection::South)
					{
						1.0
					}
					else
					{
						-1.0
					};
					terrain_ray_samples(self.ray, az, &mut |s| {
						let [x, y, z] = path.xyz(s);
						let scale = 2.0 * MOON_RADIUS_M / (1.0 - sign * z);
						Ok(self
							.rasters
							.iter()
							.find_map(|r| r.sample_xy(scale * y, sign * scale * x)))
					})?
				}
			};
			self.next += 1;
		}
		Ok((self.next == self.h_deg.len()).then(|| Horizon {
			h_deg: self.h_deg.clone(),
		}))
	}

	pub fn progress(&self) -> f32
	{
		self.next as f32 / self.h_deg.len() as f32
	}
}

struct Tiff<'a>
{
	bytes: &'a [u8],
	little: bool,
	entries: Vec<(u16, u16, usize, usize)>,
}

impl<'a> Tiff<'a>
{
	fn u16(&self, offset: usize) -> Result<u16, String>
	{
		let b: [u8; 2] = self
			.bytes
			.get(offset..offset.saturating_add(2))
			.ok_or("Truncated TIFF u16")?
			.try_into()
			.unwrap();
		Ok(
			if self.little
			{
				u16::from_le_bytes(b)
			}
			else
			{
				u16::from_be_bytes(b)
			},
		)
	}

	fn u32(&self, offset: usize) -> Result<u32, String>
	{
		let b: [u8; 4] = self
			.bytes
			.get(offset..offset.saturating_add(4))
			.ok_or("Truncated TIFF u32")?
			.try_into()
			.unwrap();
		Ok(
			if self.little
			{
				u32::from_le_bytes(b)
			}
			else
			{
				u32::from_be_bytes(b)
			},
		)
	}

	fn tag(&self, tag: u16) -> Result<Option<&[u8]>, String>
	{
		let Some(&(_, kind, count, value)) = self.entries.iter().find(|e| e.0 == tag)
		else
		{
			return Ok(None);
		};
		let size = match kind
		{
			1 | 2 => 1,
			3 => 2,
			4 | 11 => 4,
			12 => 8,
			_ => return Err(format!("Unsupported TIFF type {kind} for tag {tag}")),
		};
		let len = count.checked_mul(size).ok_or("TIFF tag length overflow")?;
		let offset = if len <= 4
		{
			value
		}
		else
		{
			self.u32(value)? as usize
		};
		Ok(Some(
			self.bytes
				.get(offset..offset.checked_add(len).ok_or("TIFF offset overflow")?)
				.ok_or("Truncated TIFF tag")?,
		))
	}

	fn integers(&self, tag: u16) -> Result<Vec<usize>, String>
	{
		let bytes = self
			.tag(tag)?
			.ok_or_else(|| format!("Missing TIFF tag {tag}"))?;
		let kind = self.entries.iter().find(|e| e.0 == tag).unwrap().1;
		let size = match kind
		{
			3 => 2,
			4 => 4,
			_ => return Err(format!("Expected integer TIFF tag {tag}")),
		};
		bytes
			.chunks_exact(size)
			.map(|b| {
				Ok(
					if size == 2
					{
						let b = b.try_into().unwrap();
						usize::from(
							if self.little
							{
								u16::from_le_bytes(b)
							}
							else
							{
								u16::from_be_bytes(b)
							},
						)
					}
					else
					{
						let b = b.try_into().unwrap();
						(if self.little
						{
							u32::from_le_bytes(b)
						}
						else
						{
							u32::from_be_bytes(b)
						}) as usize
					},
				)
			})
			.collect()
	}

	fn scalar(&self, tag: u16, default: Option<usize>) -> Result<usize, String>
	{
		if self.tag(tag)?.is_none()
		{
			return default.ok_or_else(|| format!("Missing TIFF tag {tag}"));
		}
		let v = self.integers(tag)?;
		if v.len() != 1
		{
			return Err(format!("Expected scalar TIFF tag {tag}"));
		}
		Ok(v[0])
	}
}

fn decode(bytes: &[u8], window: Window) -> Result<Raster, String>
{
	let little = match bytes.get(..2)
	{
		Some(b"II") => true,
		Some(b"MM") => false,
		_ => return Err("Expected TIFF byte order marker".into()),
	};
	let mut t = Tiff {
		bytes,
		little,
		entries: Vec::new(),
	};
	if t.u16(2)? != 42
	{
		return Err("Expected classic TIFF".into());
	}
	let ifd = t.u32(4)? as usize;
	let count = usize::from(t.u16(ifd)?);
	for i in 0..count
	{
		let offset = ifd
			.checked_add(2 + i * 12)
			.ok_or("TIFF directory overflow")?;
		let tag = t.u16(offset)?;
		if t.entries.iter().any(|e| e.0 == tag)
		{
			return Err("Duplicate TIFF tag".into());
		}
		t.entries.push((
			tag,
			t.u16(offset + 2)?,
			t.u32(offset + 4)? as usize,
			offset + 8,
		));
	}
	for (tag, default, expected) in [
		(258, None, 32),
		(259, Some(1), 1),
		(262, Some(1), 1),
		(274, Some(1), 1),
		(277, Some(1), 1),
		(284, Some(1), 1),
		(317, Some(1), 1),
		(339, None, 3),
	]
	{
		if t.scalar(tag, default)? != expected
		{
			return Err(format!("Unsupported F32 TIFF tag {tag}"));
		}
	}
	let w = t.scalar(256, None)?;
	let h = t.scalar(257, None)?;
	if w < 2 || h < 2 || w > 4096 || h > 4096
	{
		return Err("Unsupported TIFF dimensions".into());
	}
	let tiled = t.tag(324)?.is_some();
	let (cw, ch, offsets, counts) = if tiled
	{
		(
			t.scalar(322, None)?,
			t.scalar(323, None)?,
			t.integers(324)?,
			t.integers(325)?,
		)
	}
	else
	{
		(
			w,
			t.scalar(278, Some(h))?,
			t.integers(273)?,
			t.integers(279)?,
		)
	};
	if cw == 0 || ch == 0 || cw > 16_777_216 || ch > 16_777_216
	{
		return Err("Invalid TIFF chunk dimensions".into());
	}
	let across = w.div_ceil(cw);
	if offsets.len() != across * h.div_ceil(ch) || counts.len() != offsets.len()
	{
		return Err("Invalid TIFF chunk count".into());
	}
	let mut values = vec![0.0; w * h];
	for (i, (&offset, &len)) in offsets.iter().zip(&counts).enumerate()
	{
		let x = (i % across) * cw;
		let y = (i / across) * ch;
		let rows = if tiled { ch } else { ch.min(h - y) };
		let expected = cw
			.checked_mul(rows)
			.and_then(|v| v.checked_mul(4))
			.ok_or("TIFF chunk overflow")?;
		if len < expected
			|| bytes
				.get(
					offset
						..offset
							.checked_add(len)
							.ok_or("TIFF chunk offset overflow")?,
				)
				.is_none()
		{
			return Err("Truncated TIFF chunk".into());
		}
		for r in 0..ch.min(h - y)
		{
			for c in 0..cw.min(w - x)
			{
				values[(y + r) * w + x + c] = f32::from_bits(t.u32(offset + (r * cw + c) * 4)?);
			}
		}
	}
	let nodata = t
		.tag(42113)?
		.map(|v| {
			std::str::from_utf8(v)
				.map_err(|_| "Invalid TIFF NoData text".to_string())?
				.trim_matches(char::from(0))
				.trim()
				.parse::<f32>()
				.map_err(|_| "Invalid TIFF NoData value".to_string())
		})
		.transpose()?;
	Ok(Raster {
		window,
		w,
		h,
		values,
		nodata,
	})
}

#[cfg(test)]
#[path = "anypoint_tests.rs"]
mod tests;
