//! Native GeoTIFF decoding and a bounded multiresolution tile cache; core remains I/O free.

use lunar_core::{MOON_RADIUS_M, TerrainSample, to_xy};
use serde::Deserialize;
use std::{
	collections::{HashMap, VecDeque},
	fs::File,
	io::{BufReader, Read, Seek},
	path::Path,
};
use tiff::{
	decoder::{Decoder, DecodingResult},
	tags::Tag,
};

/// A published DEM asset and its reproducibility digest.
#[derive(Deserialize, serde::Serialize)]
pub struct Asset
{
	pub file: String,
	/// Source URL remains in serialized provenance.
	pub url: String,
	pub bytes: u64,
	/// SHA-256 digest recorded after the initial download.
	pub sha256: String,
}

/// Read the versioned manifest at compile time, not the DEMs.
pub fn assets() -> Result<Vec<Asset>, String>
{
	let assets: Vec<Asset> =
		serde_json::from_str(include_str!("../dem-manifest.json")).map_err(|e| e.to_string())?;
	if assets.is_empty()
		|| assets.iter().any(|a| {
			Path::new(&a.file).file_name().and_then(|p| p.to_str()) != Some(a.file.as_str())
				|| a.bytes == 0
				|| a.sha256.len() != 64
				|| !a.sha256.bytes().all(|c| c.is_ascii_hexdigit())
		})
	{
		return Err("Invalid DEM asset manifest".into());
	}
	Ok(assets)
}

/// GeoTIFF's supported affine transform, expressed at sample centers.
#[derive(Debug)]
struct Grid
{
	/// Raster dimensions count samples, not map meters.
	w: u32,
	h: u32,
	/// Square projected pixel spacing.
	res: f64,
	x: f64,
	y: f64,
}

/// Parse numeric GeoKeys without outsourcing projection interpretation.
fn key(keys: &[u16], doubles: &[f64], id: u16) -> Result<f64, String>
{
	// Require the GeoTIFF 1.x key directory header and complete four-word entries.
	if keys.len() < 4 || keys[0] != 1 || keys.len() != 4 + usize::from(keys[3]) * 4
	{
		return Err("Malformed GeoKeyDirectory".into());
	}
	let mut matches = keys[4..].as_chunks::<4>().0.iter().filter(|k| k[0] == id);
	// A required key may not be silently assumed.
	let k = matches
		.next()
		.ok_or_else(|| format!("Missing GeoKey {id}"))?;
	// Scalar numeric keys use either inline SHORT or a DOUBLE-table offset.
	if matches.next().is_some() || k[2] != 1
	{
		return Err(format!("Invalid GeoKey {id}"));
	}
	// Resolve the TIFF-defined storage location, never a guessed ordering.
	match k[1]
	{
		0 => Ok(f64::from(k[3])),
		34736 => doubles
			.get(usize::from(k[3]))
			.copied()
			.ok_or_else(|| format!("GeoKey {id} offset out of bounds")),
		_ => Err(format!("Unsupported numeric GeoKey storage for {id}")),
	}
}

/// Decode the supported unrotated, pixel-is-area affine transform.
fn grid(
	w: u32,
	h: u32,
	scale: &[f64],
	tie: &[f64],
	keys: &[u16],
	doubles: &[f64],
) -> Result<Grid, String>
{
	// Require the precise projection encoded in both PGDA products.
	for (id, expected) in [
		(1024, 1.0),
		(1025, 1.0),
		(2054, 9102.0),
		(2057, MOON_RADIUS_M),
		(2058, MOON_RADIUS_M),
		(2061, 0.0),
		(3075, 15.0),
		(3076, 9001.0),
		(3081, -90.0),
		(3082, 0.0),
		(3083, 0.0),
		(3092, 1.0),
		(3095, 0.0),
	]
	{
		// Do not accept Earth ellipsoids or a different polar convention.
		let actual = key(keys, doubles, id)?;
		// NaN must not bypass a tolerance comparison.
		if !actual.is_finite() || (actual - expected).abs() > 1e-9
		{
			return Err(format!("Unsupported projection GeoKey {id}"));
		}
	}
	if w < 2
		|| h < 2
		|| scale.len() != 3
		|| tie.len() != 6
		|| scale.iter().chain(tie).any(|v| !v.is_finite())
		|| scale[0] <= 0.0
		|| scale[0] != scale[1]
		|| scale[2] != 0.0
		|| tie[2] != 0.0
		|| tie[5] != 0.0
	{
		return Err("Unsupported GeoTIFF raster transform".into());
	}
	// RasterPixelIsArea tie points refer to corners; shift half a pixel to the first center.
	Ok(Grid {
		w,
		h,
		res: scale[0],
		x: tie[3] + (0.5 - tie[0]) * scale[0],
		y: tie[4] - (0.5 - tie[1]) * scale[1],
	})
}

/// A TIFF decoder plus bounded FIFO cache; generic Read+Seek permits in-memory test files.
struct Raster<R: Read + Seek>
{
	decoder: Decoder<R>,
	grid: Grid,
	chunk: (u32, u32),
	/// Decoded chunks keyed by TIFF chunk index.
	cache: HashMap<u32, Vec<f32>>,
	/// FIFO ordering changes only on cache insertion, keeping hits cheap.
	order: VecDeque<u32>,
	/// Account actual decoded bytes rather than guessing compression ratios.
	bytes: usize,
}

impl<R: Read + Seek> Raster<R>
{
	/// Open only the first/full-resolution IFD; COG overviews must not replace scientific samples.
	fn new(reader: R) -> Result<Self, String>
	{
		let mut decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
		Self::validate_header(&mut decoder)?;
		let (w, h) = decoder.dimensions().map_err(|e| e.to_string())?;
		let scale = decoder
			.get_tag_f64_vec(Tag::ModelPixelScaleTag)
			.map_err(|e| e.to_string())?;
		let tie = decoder
			.get_tag_f64_vec(Tag::ModelTiepointTag)
			.map_err(|e| e.to_string())?;
		let keys = decoder
			.get_tag_u16_vec(Tag::GeoKeyDirectoryTag)
			.map_err(|e| e.to_string())?;
		let doubles = decoder
			.get_tag_f64_vec(Tag::GeoDoubleParamsTag)
			.map_err(|e| e.to_string())?;
		let grid = grid(w, h, &scale, &tie, &keys, &doubles)?;
		// TIFF abstracts both strips and tiles as chunks.
		let chunk = decoder.chunk_dimensions();
		Ok(Self {
			decoder,
			grid,
			chunk,
			cache: HashMap::new(),
			order: VecDeque::new(),
			bytes: 0,
		})
	}

	fn validate_header(decoder: &mut Decoder<R>) -> Result<(), String>
	{
		// PGDA DEMs are scalar IEEE float32, not encoded normals or integer radii.
		if decoder.colortype().map_err(|e| e.to_string())? != tiff::ColorType::Gray(32)
			|| decoder
				.get_tag_u16_vec(Tag::SampleFormat)
				.map_err(|e| e.to_string())?
				!= [3]
		{
			return Err("Expected a single float32 DEM channel".into());
		}
		// Reject rotated affine transforms rather than ignoring them.
		if decoder
			.find_tag(Tag::ModelTransformationTag)
			.map_err(|e| e.to_string())?
			.is_some()
		{
			return Err("ModelTransformationTag is not supported".into());
		}
		// Top-left row orientation is required; absence means TIFF's default top-left.
		if decoder
			.find_tag(Tag::Orientation)
			.map_err(|e| e.to_string())?
			.is_some_and(|v| v.into_u16().ok() != Some(1))
		{
			return Err("Unsupported raster orientation".into());
		}
		// NaN is the published nodata representation; fail on an unimplemented numeric sentinel.
		let nodata = decoder
			.get_tag_ascii_string(Tag::GdalNodata)
			.map_err(|e| e.to_string())?;
		if !nodata.trim_end_matches('\0').eq_ignore_ascii_case("nan")
		{
			return Err("Expected NaN nodata".into());
		}
		Ok(())
	}

	/// Fetch one scalar, decoding a chunk only on a cache miss.
	fn pixel(&mut self, c: u32, r: u32) -> Result<f32, String>
	{
		let index = (r / self.chunk.1) * self.grid.w.div_ceil(self.chunk.0) + c / self.chunk.0;
		// Edge tiles have cropped output dimensions, unlike padded on-disk storage.
		let (width, _) = self.decoder.chunk_data_dimensions(index);
		if !self.cache.contains_key(&index)
		{
			// Compression errors are propagated, not reclassified as missing terrain.
			let DecodingResult::F32(values) =
				self.decoder.read_chunk(index).map_err(|e| e.to_string())?
			else
			{
				return Err("Decoder did not return float32 DEM samples".into());
			};
			// Bound each raster cache to 512 MiB.
			while self.bytes + values.len() * 4 > 512 * 1024 * 1024 && !self.order.is_empty()
			{
				let old = self.order.pop_front().unwrap();
				self.bytes -= self.cache.remove(&old).unwrap().len() * 4;
			}
			self.bytes += values.len() * 4;
			self.cache.insert(index, values);
			self.order.push_back(index);
		}
		self.cache[&index]
			.get(((r % self.chunk.1) * width + c % self.chunk.0) as usize)
			.copied()
			.ok_or_else(|| "Invalid decoded chunk dimensions".into())
	}

	fn height(&mut self, x: f64, y: f64) -> Result<Option<f64>, String>
	{
		let c = (x - self.grid.x) / self.grid.res;
		let r = (self.grid.y - y) / self.grid.res;
		// Never extrapolate a border sample beyond its center.
		if !c.is_finite()
			|| !r.is_finite()
			|| c < 0.0
			|| r < 0.0
			|| c > f64::from(self.grid.w - 1)
			|| r > f64::from(self.grid.h - 1)
		{
			return Ok(None);
		}
		let (c0, r0) = (c.floor() as u32, r.floor() as u32);
		let (c1, r1) = ((c0 + 1).min(self.grid.w - 1), (r0 + 1).min(self.grid.h - 1));
		let v = [
			self.pixel(c0, r0)?,
			self.pixel(c1, r0)?,
			self.pixel(c0, r1)?,
			self.pixel(c1, r1)?,
		];
		// Zero-weight corners must not turn a valid center into nodata.
		let weights = [
			(1.0 - c.fract()) * (1.0 - r.fract()),
			c.fract() * (1.0 - r.fract()),
			(1.0 - c.fract()) * r.fract(),
			c.fract() * r.fract(),
		];
		// A contributing nodata neighbor makes this resolution unavailable.
		if v.iter()
			.zip(weights)
			.any(|(v, w)| w > 0.0 && !v.is_finite())
		{
			return Ok(None);
		}
		Ok(Some(
			v.iter()
				.zip(weights)
				.filter(|(_, w)| *w > 0.0)
				.map(|(&v, w)| f64::from(v) * w)
				.sum(),
		))
	}
}

pub struct Terrain<R: Read + Seek = BufReader<File>>
{
	/// Each file maintains a separate bounded cache.
	rasters: Vec<Raster<R>>,
}

impl Terrain
{
	pub fn open(directory: &Path) -> Result<Self, String>
	{
		// Empty inventory must not become a silently flat Moon.
		let mut rasters = Vec::new();
		for asset in assets()?
		{
			// Resolve only a trusted manifest basename under the supplied directory.
			let path = directory.join(&asset.file);
			// Reject partial downloads before touching TIFF offsets.
			let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
			if file.metadata().map_err(|e| e.to_string())?.len() != asset.bytes
			{
				return Err(format!("Wrong DEM byte length: {}", path.display()));
			}
			let raster = Raster::new(BufReader::new(file))
				.map_err(|e| format!("{}: {e}", path.display()))?;
			// Record the exact on-disk bounds, independent of filename latitude limits.
			println!(
				"DEM {}: {}x{}, {} m/pixel, center bounds x=[{},{}] y=[{},{}], source={}, sha256={}",
				asset.file,
				raster.grid.w,
				raster.grid.h,
				raster.grid.res,
				raster.grid.x,
				raster.grid.x + f64::from(raster.grid.w - 1) * raster.grid.res,
				raster.grid.y - f64::from(raster.grid.h - 1) * raster.grid.res,
				raster.grid.y,
				asset.url,
				asset.sha256
			);
			rasters.push(raster);
		}
		// Prefer resolution, not filename lexicographic ordering.
		rasters.sort_by(|a, b| a.grid.res.total_cmp(&b.grid.res));
		Ok(Self { rasters })
	}
}

impl<R: Read + Seek> Terrain<R>
{
	/// Query a spherical location using highest-resolution valid coverage.
	pub fn sample(&mut self, lat: f64, lon: f64) -> Result<Option<TerrainSample>, String>
	{
		let (x, y) = to_xy(lat, lon);
		// Convert projected spacing to ground spacing using stereographic scale.
		let scale = 1.0 + (x * x + y * y) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M);
		// Missing high-resolution coverage or nodata falls back, but decoding errors propagate.
		for raster in &mut self.rasters
		{
			// Bilinear interpolation is never performed across two incompatible grids.
			if let Some(z_m) = raster.height(x, y)?
			{
				return Ok(Some(TerrainSample {
					z_m,
					resolution_m: raster.grid.res / scale,
				}));
			}
		}
		// None means the continuous available terrain has ended.
		Ok(None)
	}
}

#[cfg(test)]
#[path = "dem_tests.rs"]
mod tests;
