//! Moon mean-Earth apparent positions, expressed as X=90E, Y=north, Z=0E.

use crate::MOON_RADIUS_M;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPosition
{
	/// Unit Moon-fixed direction, renormalized after quantization.
	pub direction: [f32; 3],
	/// Moon-center range in meters.
	pub distance_m: f32,
}

impl BodyPosition
{
	/// Quantize only the final Moon-centered representation.
	pub fn from_cartesian(position_m: [f64; 3]) -> Result<Self, String>
	{
		let distance = position_m.iter().map(|x| x * x).sum::<f64>().sqrt();
		if !distance.is_finite() || distance <= MOON_RADIUS_M || distance > f64::from(f32::MAX)
		{
			return Err("Invalid Moon-centered body position".into());
		}
		// Preserve direction and distance separately to retain angular precision.
		let body = Self {
			direction: position_m.map(|x| (x / distance) as f32),
			distance_m: distance as f32,
		};
		// Quantization must not round an otherwise valid distance onto the excluded sphere.
		body.cartesian()?;
		Ok(body)
	}

	pub fn cartesian(self) -> Result<[f64; 3], String>
	{
		// Convert before arithmetic to avoid float norm overflow.
		let direction = self.direction.map(f64::from);
		let norm = direction.iter().map(|x| x * x).sum::<f64>().sqrt();
		// Reject corrupted directions rather than silently repairing arbitrary vectors.
		if !norm.is_finite()
			|| (norm - 1.0).abs() > 1e-4
			|| !self.distance_m.is_finite()
			|| f64::from(self.distance_m) <= MOON_RADIUS_M
		{
			return Err("Invalid ephemeris direction or distance".into());
		}
		// Remove unit-vector quantization drift independently of range.
		Ok(direction.map(|x| x / norm * f64::from(self.distance_m)))
	}
}

/// Convert one finite-distance body position to north-through-east azimuth/elevation.
pub fn topocentric(
	body: BodyPosition,
	lat_deg: f64,
	lon_deg: f64,
	height_m: f64,
) -> Result<(f64, f64), String>
{
	// Heights are radial altitude above the sphere, including terrain and instrument.
	if !lat_deg.is_finite()
		|| !(-90.0..=90.0).contains(&lat_deg)
		|| !lon_deg.is_finite()
		|| !height_m.is_finite()
		|| height_m <= -MOON_RADIUS_M
	{
		return Err("Invalid observer coordinates or radial altitude".into());
	}
	// Construct the same right-handed ENU basis as the globe.
	let (sp, cp) = lat_deg.to_radians().sin_cos();
	let (sl, cl) = lon_deg.rem_euclid(360.0).to_radians().sin_cos();
	let up = [cp * sl, sp, cp * cl];
	let east = [cl, 0.0, -sl];
	let north = [-sp * sl, cp, -sp * cl];
	// Subtract the actual observer, retaining Earth parallax.
	let position = body.cartesian()?;
	let relative =
		std::array::from_fn::<_, 3, _>(|i| position[i] - (MOON_RADIUS_M + height_m) * up[i]);
	let distance = relative.iter().map(|x| x * x).sum::<f64>().sqrt();
	if !distance.is_finite() || distance < 1e-6
	{
		return Err("Body coincides with observer or exceeds numeric range".into());
	}
	let dot = |axis: [f64; 3]| relative.iter().zip(axis).map(|(a, b)| a * b).sum::<f64>();
	let (e, n, u) = (dot(east), dot(north), dot(up));
	Ok((
		e.atan2(n).to_degrees().rem_euclid(360.0),
		u.atan2(e.hypot(n)).to_degrees(),
	))
}

/// A versioned uniformly sampled pair: Sun first, Earth second.
#[derive(Debug, PartialEq)]
pub struct Ephemeris
{
	/// Inclusive first UTC epoch in Unix seconds.
	pub start_utc: i64,
	/// Positive sample spacing in elapsed UTC seconds.
	pub step_s: u32,
	/// Unit directions and ranges at each epoch.
	pub samples: Vec<[BodyPosition; 2]>,
}

impl Ephemeris
{
	/// Resolve an index without overflow or extrapolation.
	pub fn epoch(&self, index: usize) -> Option<i64>
	{
		// Public fields must not permit out-of-range timestamp construction.
		if index >= self.samples.len() || self.step_s == 0
		{
			return None;
		}
		self.start_utc.checked_add(
			i64::try_from(index)
				.ok()?
				.checked_mul(i64::from(self.step_s))?,
		)
	}

	/// Decode explicitly little-endian values without alignment or unsafe assumptions.
	pub fn decode(bytes: &[u8]) -> Result<Self, String>
	{
		if bytes.len() < 24 || &bytes[..8] != b"MLVEPH01"
		{
			return Err("Invalid ephemeris header".into());
		}
		let start_utc = i64::from_le_bytes(bytes[8..16].try_into().unwrap());
		let step_s = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
		let count = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
		// Compare without overflow on wasm32.
		if count == 0 || step_s == 0 || bytes.len().checked_sub(24) != count.checked_mul(32)
		{
			return Err("Invalid ephemeris cadence or payload length".into());
		}
		let mut samples = Vec::with_capacity(count);
		for record in bytes[24..].as_chunks::<32>().0
		{
			let values: [f32; 8] = std::array::from_fn(|i| {
				f32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap())
			});
			let pair = [0, 4].map(|i| BodyPosition {
				direction: [values[i], values[i + 1], values[i + 2]],
				distance_m: values[i + 3],
			});
			for body in pair
			{
				body.cartesian()?;
			}
			samples.push(pair);
		}
		// Reject timestamps that overflow the serialized UTC domain.
		let result = Self {
			start_utc,
			step_s,
			samples,
		};
		result
			.epoch(count - 1)
			.ok_or("Ephemeris end time overflows")?;
		Ok(result)
	}

	/// Serialize the portable format, validating even manually constructed values.
	pub fn encode(&self) -> Result<Vec<u8>, String>
	{
		let count = u32::try_from(self.samples.len()).map_err(|_| "Too many ephemeris samples")?;
		let size = self
			.samples
			.len()
			.checked_mul(32)
			.and_then(|n| n.checked_add(24))
			.ok_or("Ephemeris size overflows")?;
		if count == 0 || self.epoch(self.samples.len() - 1).is_none()
		{
			return Err("Empty ephemeris or invalid cadence".into());
		}
		let mut bytes = Vec::with_capacity(size);
		bytes.extend_from_slice(b"MLVEPH01");
		bytes.extend_from_slice(&self.start_utc.to_le_bytes());
		bytes.extend_from_slice(&self.step_s.to_le_bytes());
		bytes.extend_from_slice(&count.to_le_bytes());
		for pair in &self.samples
		{
			for body in pair
			{
				body.cartesian()?;
				for value in body.direction.into_iter().chain([body.distance_m])
				{
					bytes.extend_from_slice(&value.to_le_bytes());
				}
			}
		}
		Ok(bytes)
	}
}

#[cfg(test)]
#[path = "ephemeris_tests.rs"]
mod tests;
