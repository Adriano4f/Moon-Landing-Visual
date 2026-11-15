//! Native GeoTIFF decoding and a bounded multiresolution tile cache; core remains I/O free.

// Import only pure geometry and sample values; C++ includes the scientific header.
use lunar_core::{MOON_RADIUS_M, TerrainSample, to_xy};
// Deserialize the reproducible asset manifest; C++ JSON-to-struct adapter.
use serde::Deserialize;
// Keep filesystem, buffering and cache ownership here, not in core; C++ standard library headers.
use std::{
	collections::{HashMap, VecDeque},
	fs::File,
	io::{BufReader, Read, Seek},
	path::Path,
};
// Decode TIFF containers, not a system GDAL library; C++ a native TIFF decoder API.
use tiff::{
	decoder::{Decoder, DecodingResult},
	tags::Tag,
};

// A published DEM asset and its reproducibility digest; C++ aggregate descriptor.
#[derive(Deserialize, serde::Serialize)]
pub struct Asset
{
	// Local basename; C++ std::string.
	pub file: String,
	// Source URL remains in serialized provenance; C++ std::string.
	pub url: String,
	// Expected byte length from HTTP headers; C++ uint64_t.
	pub bytes: u64,
	// SHA-256 digest recorded after the initial download; C++ std::string.
	pub sha256: String,
}

// Read the versioned manifest at compile time, not the DEMs; C++ embedded small text resource.
pub fn assets() -> Result<Vec<Asset>, String>
{
	// Fail on a malformed developer manifest; C++ expected-returning JSON parse.
	let assets: Vec<Asset> =
		serde_json::from_str(include_str!("../dem-manifest.json")).map_err(|e| e.to_string())?;
	// Validate trusted basenames and recorded digests before opening files; C++ all_of over descriptors.
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
	// Return owned validated descriptors; C++ vector move.
	Ok(assets)
}

// GeoTIFF's supported affine transform, expressed at sample centers; C++ plain struct.
#[derive(Debug)]
struct Grid
{
	// Raster dimensions count samples, not map meters; C++ uint32_t members.
	w: u32,
	h: u32,
	// Square projected pixel spacing; C++ double.
	res: f64,
	// First sample's map coordinates; C++ double members.
	x: f64,
	y: f64,
}

// Parse numeric GeoKeys without outsourcing projection interpretation; C++ free function.
fn key(keys: &[u16], doubles: &[f64], id: u16) -> Result<f64, String>
{
	// Require the GeoTIFF 1.x key directory header and complete four-word entries; C++ span bounds checks.
	if keys.len() < 4 || keys[0] != 1 || keys.len() != 4 + usize::from(keys[3]) * 4
	{
		return Err("Malformed GeoKeyDirectory".into());
	}
	// Find exactly one definition, rejecting ambiguous duplicate keys; C++ filtered iterator.
	let mut matches = keys[4..].as_chunks::<4>().0.iter().filter(|k| k[0] == id);
	// A required key may not be silently assumed; C++ optional extraction.
	let k = matches
		.next()
		.ok_or_else(|| format!("Missing GeoKey {id}"))?;
	// Scalar numeric keys use either inline SHORT or a DOUBLE-table offset; C++ checked tagged representation.
	if matches.next().is_some() || k[2] != 1
	{
		return Err(format!("Invalid GeoKey {id}"));
	}
	// Resolve the TIFF-defined storage location, never a guessed ordering; C++ switch.
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

// Decode the supported unrotated, pixel-is-area affine transform; C++ constructor helper.
fn grid(
	w: u32,
	h: u32,
	scale: &[f64],
	tie: &[f64],
	keys: &[u16],
	doubles: &[f64],
) -> Result<Grid, String>
{
	// Require the precise projection encoded in both PGDA products; C++ loop over expected key/value pairs.
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
		// Do not accept Earth ellipsoids or a different polar convention; C++ tolerance comparison.
		let actual = key(keys, doubles, id)?;
		// NaN must not bypass a tolerance comparison; C++ std::isfinite plus absolute difference.
		if !actual.is_finite() || (actual - expected).abs() > 1e-9
		{
			return Err(format!("Unsupported projection GeoKey {id}"));
		}
	}
	// Validate dimensions, scalar scale, and the one supported tie point; C++ geometric preconditions.
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
	// RasterPixelIsArea tie points refer to corners; shift half a pixel to the first center; C++ affine transform.
	Ok(Grid {
		w,
		h,
		res: scale[0],
		x: tie[3] + (0.5 - tie[0]) * scale[0],
		y: tie[4] - (0.5 - tie[1]) * scale[1],
	})
}

// A TIFF decoder plus bounded FIFO cache; generic Read+Seek permits in-memory test files; C++ template class.
struct Raster<R: Read + Seek>
{
	// Own the decoder and its underlying seekable file; C++ RAII decoder.
	decoder: Decoder<R>,
	// Validated spatial interpretation; C++ value member.
	grid: Grid,
	// Native strip or tile dimensions; C++ pair<uint32_t,uint32_t>.
	chunk: (u32, u32),
	// Decoded chunks keyed by TIFF chunk index; C++ unordered_map.
	cache: HashMap<u32, Vec<f32>>,
	// FIFO ordering changes only on cache insertion, keeping hits cheap; C++ deque.
	order: VecDeque<u32>,
	// Account actual decoded bytes rather than guessing compression ratios; C++ size_t.
	bytes: usize,
}

// Define decoder operations over either disk files or test cursors; C++ template member definitions.
impl<R: Read + Seek> Raster<R>
{
	// Open only the first/full-resolution IFD; COG overviews must not replace scientific samples; C++ factory.
	fn new(reader: R) -> Result<Self, String>
	{
		// Decode TIFF/BigTIFF directory and compression information; C++ checked constructor.
		let mut decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
		// PGDA DEMs are scalar IEEE float32, not encoded normals or integer radii; C++ type check.
		if decoder.colortype().map_err(|e| e.to_string())? != tiff::ColorType::Gray(32)
			|| decoder
				.get_tag_u16_vec(Tag::SampleFormat)
				.map_err(|e| e.to_string())?
				!= [3]
		{
			return Err("Expected a single float32 DEM channel".into());
		}
		// Reject rotated affine transforms rather than ignoring them; C++ optional metadata guard.
		if decoder
			.find_tag(Tag::ModelTransformationTag)
			.map_err(|e| e.to_string())?
			.is_some()
		{
			return Err("ModelTransformationTag is not supported".into());
		}
		// Top-left row orientation is required; absence means TIFF's default top-left; C++ optional default.
		if decoder
			.find_tag(Tag::Orientation)
			.map_err(|e| e.to_string())?
			.is_some_and(|v| v.into_u16().ok() != Some(1))
		{
			return Err("Unsupported raster orientation".into());
		}
		// NaN is the published nodata representation; fail on an unimplemented numeric sentinel; C++ metadata validation.
		let nodata = decoder
			.get_tag_ascii_string(Tag::GdalNodata)
			.map_err(|e| e.to_string())?;
		if !nodata.trim_end_matches('\0').eq_ignore_ascii_case("nan")
		{
			return Err("Expected NaN nodata".into());
		}
		// Read named TIFF tags instead of hardcoded raster dimensions; C++ typed tag accessors.
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
		// Validate projection and pixel semantics before allowing any samples; C++ checked factory.
		let grid = grid(w, h, &scale, &tie, &keys, &doubles)?;
		// TIFF abstracts both strips and tiles as chunks; C++ pair of dimensions.
		let chunk = decoder.chunk_dimensions();
		// Start with no decoded elevation payload; C++ aggregate initialization.
		Ok(Self {
			decoder,
			grid,
			chunk,
			cache: HashMap::new(),
			order: VecDeque::new(),
			bytes: 0,
		})
	}

	// Fetch one scalar, decoding a chunk only on a cache miss; C++ checked member function.
	fn pixel(&mut self, c: u32, r: u32) -> Result<f32, String>
	{
		// Raster bounds were checked by bilinear sampling; convert row/column into a chunk index; C++ integer arithmetic.
		let index = (r / self.chunk.1) * self.grid.w.div_ceil(self.chunk.0) + c / self.chunk.0;
		// Edge tiles have cropped output dimensions, unlike padded on-disk storage; C++ get decoded stride.
		let (width, _) = self.decoder.chunk_data_dimensions(index);
		// Avoid repeatedly decoding a neighboring tile during interpolation; C++ unordered_map membership.
		if !self.cache.contains_key(&index)
		{
			// Compression errors are propagated, not reclassified as missing terrain; C++ expected extraction.
			let DecodingResult::F32(values) =
				self.decoder.read_chunk(index).map_err(|e| e.to_string())?
			else
			{
				return Err("Decoder did not return float32 DEM samples".into());
			};
			// Bound each raster cache to 512 MiB; C++ FIFO eviction loop.
			while self.bytes + values.len() * 4 > 512 * 1024 * 1024 && !self.order.is_empty()
			{
				// Every queued key belongs to the cache; C++ pop_front and erase.
				let old = self.order.pop_front().unwrap();
				self.bytes -= self.cache.remove(&old).unwrap().len() * 4;
			}
			// Account and retain this decoded chunk; C++ move into an unordered_map.
			self.bytes += values.len() * 4;
			self.cache.insert(index, values);
			self.order.push_back(index);
		}
		// Convert the in-chunk coordinate using its actual decoded width; C++ vector::at.
		self.cache[&index]
			.get(((r % self.chunk.1) * width + c % self.chunk.0) as usize)
			.copied()
			.ok_or_else(|| "Invalid decoded chunk dimensions".into())
	}

	// Bilinearly interpolate four valid sample centers; C++ optional-returning sampler.
	fn height(&mut self, x: f64, y: f64) -> Result<Option<f64>, String>
	{
		// Invert the unrotated center-coordinate affine transform; C++ two doubles.
		let c = (x - self.grid.x) / self.grid.res;
		let r = (self.grid.y - y) / self.grid.res;
		// Never extrapolate a border sample beyond its center; C++ range guards.
		if !c.is_finite()
			|| !r.is_finite()
			|| c < 0.0
			|| r < 0.0
			|| c > f64::from(self.grid.w - 1)
			|| r > f64::from(self.grid.h - 1)
		{
			return Ok(None);
		}
		// Choose the surrounding sample indices, clamping only the last center; C++ integer indices.
		let (c0, r0) = (c.floor() as u32, r.floor() as u32);
		let (c1, r1) = ((c0 + 1).min(self.grid.w - 1), (r0 + 1).min(self.grid.h - 1));
		// Decode all four neighbors before evaluating the interpolation; C++ small fixed array.
		let v = [
			self.pixel(c0, r0)?,
			self.pixel(c1, r0)?,
			self.pixel(c0, r1)?,
			self.pixel(c1, r1)?,
		];
		// Zero-weight corners must not turn a valid center into nodata; C++ array of bilinear weights.
		let weights = [
			(1.0 - c.fract()) * (1.0 - r.fract()),
			c.fract() * (1.0 - r.fract()),
			(1.0 - c.fract()) * r.fract(),
			c.fract() * r.fract(),
		];
		// A contributing nodata neighbor makes this resolution unavailable; C++ any_of zipped values.
		if v.iter()
			.zip(weights)
			.any(|(v, w)| w > 0.0 && !v.is_finite())
		{
			return Ok(None);
		}
		// Evaluate only contributing weights in double precision; C++ filtered inner product.
		Ok(Some(
			v.iter()
				.zip(weights)
				.filter(|(_, w)| *w > 0.0)
				.map(|(&v, w)| f64::from(v) * w)
				.sum(),
		))
	}
}

// Own only the selected published rasters, ordered finest first; C++ vector of RAII decoders.
pub struct Terrain<R: Read + Seek = BufReader<File>>
{
	// Each file maintains a separate bounded cache; C++ vector<Raster<ifstream>>.
	rasters: Vec<Raster<R>>,
}

// Define the preparation-only multiresolution lookup; C++ class member definitions.
impl Terrain
{
	// Require the manifest's exact file inventory and sizes; C++ filesystem validation and factories.
	pub fn open(directory: &Path) -> Result<Self, String>
	{
		// Empty inventory must not become a silently flat Moon; C++ vector construction.
		let mut rasters = Vec::new();
		// Visit all finest nonredundant resolution levels selected for this site set; C++ range-for.
		for asset in assets()?
		{
			// Resolve only a trusted manifest basename under the supplied directory; C++ path concatenation.
			let path = directory.join(&asset.file);
			// Reject partial downloads before touching TIFF offsets; C++ RAII file and size checks.
			let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
			if file.metadata().map_err(|e| e.to_string())?.len() != asset.bytes
			{
				return Err(format!("Wrong DEM byte length: {}", path.display()));
			}
			// Buffer small tag reads and seek over compressed tile payloads; C++ buffered ifstream.
			let raster = Raster::new(BufReader::new(file))
				.map_err(|e| format!("{}: {e}", path.display()))?;
			// Record the exact on-disk bounds, independent of filename latitude limits; C++ formatted diagnostic.
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
			// Move decoder ownership into the hierarchy; C++ vector::push_back(move).
			rasters.push(raster);
		}
		// Prefer resolution, not filename lexicographic ordering; C++ stable_sort with comparator.
		rasters.sort_by(|a, b| a.grid.res.total_cmp(&b.grid.res));
		// Construct the scientific sampler with no GPU/UI coupling; C++ aggregate return.
		Ok(Self { rasters })
	}
}

// Sampling does not depend on the reader's concrete type; C++ template member definitions.
impl<R: Read + Seek> Terrain<R>
{
	// Query a spherical location using highest-resolution valid coverage; C++ expected<optional<Sample>>.
	pub fn sample(&mut self, lat: f64, lon: f64) -> Result<Option<TerrainSample>, String>
	{
		// All selected files share this validated projection; C++ one coordinate conversion.
		let (x, y) = to_xy(lat, lon);
		// Convert projected spacing to ground spacing using stereographic scale; C++ double.
		let scale = 1.0 + (x * x + y * y) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M);
		// Missing high-resolution coverage or nodata falls back, but decoding errors propagate; C++ ordered search.
		for raster in &mut self.rasters
		{
			// Bilinear interpolation is never performed across two incompatible grids; C++ checked optional.
			if let Some(z_m) = raster.height(x, y)?
			{
				return Ok(Some(TerrainSample {
					z_m,
					resolution_m: raster.grid.res / scale,
				}));
			}
		}
		// None means the continuous available terrain has ended; C++ nullopt.
		Ok(None)
	}
}

// Compile fixtures only into the preparation test binary; C++ conditional test module.
#[cfg(test)]
#[path = "dem_tests.rs"]
mod tests;
