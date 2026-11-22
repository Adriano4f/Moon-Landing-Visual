//! Small, demand-loaded view windows; photographs and measured relief never share a lighting path.
use crate::{deep, math};
use egui::mutex::Mutex;
use std::{io::Cursor, sync::Arc};

mod fetch;
mod regions;

const SIDE: u32 = 512;
const PHOTO_SIDE: u32 = 1024;
const SERVICE: &str = "https://trek.nasa.gov/moon/trekarcgis/rest/services";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Spec
{
	pub bbox: [f64; 4],
	pub pole: i32,
	pub source: &'static str,
	pub photo: bool,
	pub level: u32,
	pub first: [u32; 2],
}

impl Spec
{
	pub fn wanted(view: math::View, realistic: bool) -> Option<Self>
	{
		if view.zoom < 16.0 || !view.zoom.is_finite()
		{
			return None;
		}
		let point = deep::Camera::new(view).hit(0.0, 0.0)?;
		let lat = f64::from(point[1]).clamp(-1.0, 1.0).asin().to_degrees();
		let lon = f64::from(point[0]).atan2(f64::from(point[2])).to_degrees();
		let width = f64::from(
			2.0 * (view.camera_distance() - 1.0) / view.projection()[0].min(view.projection()[5]),
		) * f64::from(deep::RADIUS)
			* 1.8;
		let pole = if lat.abs() >= 75.0
		{
			if lat < 0.0 { -1 } else { 1 }
		}
		else
		{
			0
		};
		let (x, y) = coordinates(lat, lon, pole);
		if !realistic
		{
			let (source, maximum) = if lat.abs() >= 86.0
			{
				(
					if pole < 0
					{
						"LRO_NAC_Mosaic_SPole85.5_1mp"
					}
					else
					{
						"LRO_NAC_Mosaic_NPole85.5_1mp"
					},
					11,
				)
			}
			else if pole == 0
			{
				let region = regions::REGIONS
					.iter()
					.find(|r| lon > r.1[0] && lon < r.1[2] && lat > r.1[1] && lat < r.1[3])?;
				(region.0, region.2)
			}
			else
			{
				return None;
			};
			let span = if pole == 0
			{
				std::f64::consts::PI * f64::from(deep::RADIUS)
			}
			else
			{
				2.0 * f64::from(deep::EXTENT)
			};
			let level = (span * 4.0 / width).log2().floor().max(0.0) as u32;
			let level = level.min(maximum);
			let rows = f64::from(1_u32 << level);
			let (u, v) = if pole == 0
			{
				((lon + 180.0) / 360.0, (90.0 - lat) / 180.0)
			}
			else
			{
				(
					(x / f64::from(deep::EXTENT) + 1.0) / 2.0,
					(1.0 - y / f64::from(deep::EXTENT)) / 2.0,
				)
			};
			let cols = if pole == 0 { rows * 2.0 } else { rows };
			if rows < 4.0
			{
				return None;
			}
			let col = (u * cols - 2.0).round().clamp(0.0, cols - 4.0) as u32;
			let row = (v * rows - 2.0).round().clamp(0.0, rows - 4.0) as u32;
			let bbox = if pole == 0
			{
				[
					-180.0 + f64::from(col) / cols * 360.0,
					90.0 - f64::from(row + 4) / rows * 180.0,
					-180.0 + f64::from(col + 4) / cols * 360.0,
					90.0 - f64::from(row) / rows * 180.0,
				]
			}
			else
			{
				let step = 2.0 * f64::from(deep::EXTENT) / rows;
				[
					-f64::from(deep::EXTENT) + f64::from(col) * step,
					f64::from(deep::EXTENT) - f64::from(row + 4) * step,
					-f64::from(deep::EXTENT) + f64::from(col + 4) * step,
					f64::from(deep::EXTENT) - f64::from(row) * step,
				]
			};
			if pole == 0
			{
				let region = regions::REGIONS.iter().find(|r| r.0 == source)?;
				if bbox[0] < region.1[0]
					|| bbox[1] < region.1[1]
					|| bbox[2] > region.1[2]
					|| bbox[3] > region.1[3]
				{
					return None;
				}
			}
			return Some(Self {
				bbox,
				pole,
				source,
				photo: true,
				level,
				first: [col, row],
			});
		}
		let (source, native, extent) = if pole == 0
		{
			(
				"LRO_LOLA_DEM_Global_256ppd_v06",
				std::f64::consts::PI * f64::from(deep::RADIUS) / 180.0 / 256.0,
				0.0,
			)
		}
		else if x.abs().max(y.abs()) + width < 75_840.0
		{
			(
				if pole < 0
				{
					"LRO_LOLA_DEM_SPole875_5mp_v04"
				}
				else
				{
					"LRO_LOLA_DEM_NPole875_5mp_v04"
				},
				5.0,
				75_840.0,
			)
		}
		else if x.abs().max(y.abs()) + width < 457_440.0
		{
			(
				if pole < 0
				{
					"LRO_LOLA_DEM_SPole75_30mp_v04"
				}
				else
				{
					"LRO_LOLA_DEM_NPole75_30mp_v04"
				},
				30.0,
				457_440.0,
			)
		}
		else
		{
			return None;
		};
		let step = native * 2.0_f64.powf((width / f64::from(SIDE) / native).log2().ceil().max(0.0));
		let half = step * f64::from(SIDE) / 2.0;
		let (hx, hy) = if pole == 0
		{
			let hy = half / f64::from(deep::RADIUS) * 180.0 / std::f64::consts::PI;
			(hy / lat.to_radians().cos().max(0.2), hy)
		}
		else
		{
			(half, half)
		};
		let mut cx = (x / (hx / 2.0)).round() * hx / 2.0;
		let mut cy = (y / (hy / 2.0)).round() * hy / 2.0;
		if pole == 0
		{
			cx = cx.clamp(-180.0 + hx, 180.0 - hx);
			cy = cy.clamp(-90.0 + hy, 90.0 - hy);
		}
		else
		{
			cx = cx.clamp(-extent + hx, extent - hx);
			cy = cy.clamp(-extent + hy, extent - hy);
		}
		Some(Self {
			bbox: [cx - hx, cy - hy, cx + hx, cy + hy],
			pole,
			source,
			photo: false,
			level: 0,
			first: [0, 0],
		})
	}

	fn url(self, index: u32) -> String
	{
		if self.photo
		{
			let projection = if self.pole == 0
			{
				"EQ"
			}
			else if self.pole < 0
			{
				"SP"
			}
			else
			{
				"NP"
			};
			return format!(
				"https://trek.nasa.gov/tiles/Moon/{projection}/{}/1.0.0/default/default028mm/{}/{}/{}.png",
				self.source,
				self.level,
				self.first[1] + index / 4,
				self.first[0] + index % 4
			);
		}
		let [a, b, c, d] = self.bbox;
		format!(
			"{SERVICE}/{}/ImageServer/exportImage?f=image&bbox={a}%2C{b}%2C{c}%2C{d}&size={SIDE}%2C{SIDE}&adjustAspectRatio=false&format=tiff&pixelType=F32&compression=none&renderingRule=%7B%22rasterFunction%22%3A%22None%22%7D&interpolation=RSP_BilinearInterpolation",
			self.source
		)
	}

	fn retains(self, next: Self, view: math::View, zoom: f32) -> bool
	{
		if !self.photo
			|| !next.photo
			|| self.source != next.source
			|| self.pole != next.pole
			|| (self.level != next.level
				&& (self.level.abs_diff(next.level) > 1 || (view.zoom / zoom).ln().abs() > 0.14))
		{
			return false;
		}
		let Some(point) = deep::Camera::new(view).hit(0.0, 0.0)
		else
		{
			return false;
		};
		let lat = f64::from(point[1]).clamp(-1.0, 1.0).asin().to_degrees();
		let lon = f64::from(point[0]).atan2(f64::from(point[2])).to_degrees();
		let (x, y) = coordinates(lat, lon, self.pole);
		let u = (x - self.bbox[0]) / (self.bbox[2] - self.bbox[0]);
		let v = (y - self.bbox[1]) / (self.bbox[3] - self.bbox[1]);
		(0.25..=0.75).contains(&u) && (0.25..=0.75).contains(&v)
	}
}

fn coordinates(lat: f64, lon: f64, pole: i32) -> (f64, f64)
{
	if pole == 0
	{
		return (lon, lat);
	}
	let rho = 2.0 * f64::from(deep::RADIUS) * ((90.0 - lat.abs()).to_radians() / 2.0).tan();
	(
		rho * lon.to_radians().sin(),
		rho * lon.to_radians().cos() * if pole < 0 { 1.0 } else { -1.0 },
	)
}

fn position(spec: Spec, col: usize, row: usize, height: f32) -> [f64; 3]
{
	let x = spec.bbox[0] + (col as f64 + 0.5) / f64::from(SIDE) * (spec.bbox[2] - spec.bbox[0]);
	let y = spec.bbox[3] - (row as f64 + 0.5) / f64::from(SIDE) * (spec.bbox[3] - spec.bbox[1]);
	let (lat, lon) = if spec.pole == 0
	{
		(y.to_radians(), x.to_radians())
	}
	else
	{
		let lat = std::f64::consts::FRAC_PI_2
			- 2.0 * (x.hypot(y) / (2.0 * f64::from(deep::RADIUS))).atan();
		(
			lat * f64::from(spec.pole),
			x.atan2(if spec.pole < 0 { y } else { -y }),
		)
	};
	let radius = f64::from(deep::RADIUS) + f64::from(height);
	[
		radius * lat.cos() * lon.sin(),
		radius * lat.sin(),
		radius * lat.cos() * lon.cos(),
	]
}

fn normals(spec: Spec, values: &[f32]) -> Result<Vec<u8>, String>
{
	let side = SIDE as usize;
	if values.len() != side * side
		|| values
			.iter()
			.any(|h| !h.is_finite() || !(-15_000.0..15_000.0).contains(h))
	{
		return Err("Invalid zoom DEM samples".into());
	}
	let mut pixels = vec![0; side * side * 3];
	for row in 0..side
	{
		for col in 0..side
		{
			let left = col.saturating_sub(1);
			let right = (col + 1).min(side - 1);
			let top = row.saturating_sub(1);
			let bottom = (row + 1).min(side - 1);
			let a = position(spec, left, row, values[row * side + left]);
			let b = position(spec, right, row, values[row * side + right]);
			let c = position(spec, col, top, values[top * side + col]);
			let d = position(spec, col, bottom, values[bottom * side + col]);
			let u: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
			let v: [f64; 3] = std::array::from_fn(|i| c[i] - d[i]);
			let mut n = [
				u[1] * v[2] - u[2] * v[1],
				u[2] * v[0] - u[0] * v[2],
				u[0] * v[1] - u[1] * v[0],
			];
			let radial = position(spec, col, row, 0.0);
			if n.iter().zip(radial).map(|(a, b)| a * b).sum::<f64>() < 0.0
			{
				n = n.map(|v| -v);
			}
			let length = n.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-12);
			for i in 0..3
			{
				pixels[(row * side + col) * 3 + i] =
					((n[i] / length * 0.5 + 0.5) * 255.0).round() as u8;
			}
		}
	}
	Ok(pixels)
}

fn decode(spec: Spec, bytes: &[u8]) -> Result<Vec<u8>, String>
{
	let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
	if decoder.dimensions().map_err(|e| e.to_string())? != (SIDE, SIDE)
	{
		return Err("Unexpected zoom DEM dimensions".into());
	}
	let tiff::decoder::DecodingResult::F32(values) =
		decoder.read_image().map_err(|e| e.to_string())?
	else
	{
		return Err("Expected F32 zoom DEM".into());
	};
	let mut pixels = normals(spec, &values)?;
	for height in values
	{
		let packed = ((height + 15000.0) * 2.0).round() as u16;
		pixels.extend_from_slice(&[(packed >> 8) as u8, packed as u8, 255, 255]);
	}
	Ok(pixels)
}

#[derive(Default)]
pub(super) struct State
{
	pub pending: Option<(Spec, Vec<u8>)>,
	last: Option<Spec>,
	last_zoom: f32,
	flight: bool,
}

pub(super) type Shared = Arc<Mutex<State>>;

pub(super) fn advance(shared: &Shared, view: math::View, realistic: bool, ctx: &egui::Context)
{
	let Some(spec) = Spec::wanted(view, realistic)
	else
	{
		return;
	};
	{
		let mut state = shared.lock();
		if state.flight
			|| state.pending.is_some()
			|| state.last == Some(spec)
			|| state.last.is_some_and(|last| last.retains(spec, view, state.last_zoom))
		{
			return;
		}
		state.last = Some(spec);
		state.last_zoom = view.zoom;
		state.flight = true;
	}
	if spec.photo
	{
		let mut photos = Photos {
			spec,
			next: 0,
			remaining: 0,
			pixels: vec![0; (PHOTO_SIDE * PHOTO_SIDE * 3) as usize],
			failed: false,
		};
		let jobs = photos.jobs();
		let photos = Arc::new(Mutex::new(photos));
		for index in jobs
		{
			fetch_photo(Arc::clone(&photos), Arc::clone(shared), ctx.clone(), index);
		}
	}
	else
	{
		let shared = Arc::clone(shared);
		let ctx = ctx.clone();
		ehttp::fetch(ehttp::Request::get(spec.url(0)), move |response| {
			let result = response.and_then(|r| {
				if r.ok
				{
					decode(spec, &r.bytes)
				}
				else
				{
					Err(format!("Zoom DEM HTTP {}", r.status))
				}
			});
			finish(&shared, spec, result, &ctx);
		});
	}
}

fn finish(shared: &Shared, spec: Spec, result: Result<Vec<u8>, String>, ctx: &egui::Context)
{
	let mut state = shared.lock();
	state.flight = false;
	if let Ok(pixels) = result
	{
		state.pending = Some((spec, pixels));
	}
	ctx.request_repaint();
}

struct Photos
{
	spec: Spec,
	next: u32,
	remaining: u32,
	pixels: Vec<u8>,
	failed: bool,
}

impl Photos
{
	fn jobs(&mut self) -> Vec<u32>
	{
		let mut jobs = Vec::new();
		while !self.failed && self.next < 16 && self.remaining < 4
		{
			jobs.push(self.next);
			self.next += 1;
			self.remaining += 1;
		}
		jobs
	}
}

fn fetch_photo(photos: Arc<Mutex<Photos>>, shared: Shared, ctx: egui::Context, index: u32)
{
	let url = photos.lock().spec.url(index);
	fetch::photo(url, move |image| {
		let (jobs, complete) = {
			let mut batch = photos.lock();
			batch.remaining -= 1;
			match image
			{
				Ok(image) if image.len() == 256 * 256 * 3 =>
				{
					for row in 0..256_usize
					{
						let dest = (((index / 4) as usize * 256 + row) * PHOTO_SIDE as usize
							+ (index % 4) as usize * 256)
							* 3;
						batch.pixels[dest..dest + 768]
							.copy_from_slice(&image[row * 768..(row + 1) * 768]);
					}
				}
				_ => batch.failed = true,
			}
			let jobs = batch.jobs();
			let complete = if batch.remaining == 0 && (batch.failed || batch.next == 16)
			{
				Some((
					batch.spec,
					if batch.failed
					{
						Err("Incomplete zoom photograph".into())
					}
					else
					{
						Ok(std::mem::take(&mut batch.pixels))
					},
				))
			}
			else
			{
				None
			};
			(jobs, complete)
		};
		if let Some((spec, result)) = complete
		{
			finish(&shared, spec, result, &ctx);
		}
		for index in jobs
		{
			fetch_photo(Arc::clone(&photos), Arc::clone(&shared), ctx.clone(), index);
		}
	});
}

#[cfg(test)]
mod tests
{
	use super::*;
	use tiff::encoder::{TiffEncoder, colortype::Gray32Float};

	fn view(lat: f32, lon: f32, zoom: f32) -> math::View
	{
		math::View {
			rotation: math::IDENTITY,
			yaw: -lon.to_radians(),
			tilt: lat.to_radians(),
			zoom,
			pan: [0.0; 2],
			aspect: 1.0,
			pixels: [1000.0; 2],
			scale: 1.0,
		}
	}

	#[test]
	fn photographic_windows_retain_overlap_with_bounded_zoom_hysteresis()
	{
		let camera = view(0.0, -125.6, 1024.0);
		let spec = Spec::wanted(camera, false).unwrap();
		let retains = |next, view| spec.retains(next, view, camera.zoom);
		assert!(retains(spec, camera));
		assert!(!retains(Spec { source: "other", ..spec }, camera));
		assert!(!retains(Spec { pole: 1, ..spec }, camera));
		let finer = Spec { level: spec.level + 1, ..spec };
		assert!(retains(finer, camera));
		assert!(!retains(finer, math::View { zoom: 1200.0, ..camera }));
		assert!(retains(
			Spec {
				first: [spec.first[0] + 1, spec.first[1]],
				..spec
			},
			camera
		));
		assert!(!retains(
			Spec {
				level: spec.level + 2,
				..spec
			},
			camera
		));
		assert!(!retains(
			Spec {
				photo: false,
				..spec
			},
			camera
		));
		assert!(!retains(spec, view(0.0, -120.0, 1024.0)));
		assert!(!retains(
			spec,
			math::View {
				pan: [1000.0; 2],
				..camera
			}
		));
	}

	#[test]
	fn windows_are_zoom_only_bounded_and_mode_specific()
	{
		for realistic in [false, true]
		{
			for zoom in [0.5, 1.0, 8.0, f32::NAN]
			{
				assert!(Spec::wanted(view(0.0, -125.6, zoom), realistic).is_none());
			}
		}
		assert!(Spec::wanted(view(0.0, 0.0, 1024.0), false).is_none());
		for lat in [-89.5, 0.0, 89.5]
		{
			for realistic in [false, true]
			{
				let spec = Spec::wanted(view(lat, -125.6, 1024.0), realistic).unwrap();
				assert_eq!(spec.photo, !realistic);
				assert_eq!(
					spec.pole,
					if lat < -75.0
					{
						-1
					}
					else if lat > 75.0
					{
						1
					}
					else
					{
						0
					}
				);
				let (x, y) = coordinates(f64::from(lat), -125.6, spec.pole);
				assert!(
					x > spec.bbox[0] && x < spec.bbox[2] && y > spec.bbox[1] && y < spec.bbox[3]
				);
				if spec.photo
				{
					assert!(!spec.url(0).contains("DEM"));
					assert!(spec.level <= 14);
				}
			}
		}
		let mut empty = view(0.0, 0.0, 1024.0);
		empty.pan = [1000.0; 2];
		assert!(Spec::wanted(empty, true).is_none());
	}

	#[test]
	fn wrapped_longitudes_agree_and_dem_normals_face_outwards()
	{
		let a = Spec::wanted(view(0.0, -125.6, 1024.0), true).unwrap();
		let b = Spec::wanted(view(0.0, 234.4, 1024.0), true).unwrap();
		assert_eq!(a, b);
		for lat in [-89.5, 0.0, 89.5]
		{
			let spec = Spec::wanted(view(lat, 12.0, 1024.0), true).unwrap();
			let pixels = normals(spec, &vec![0.0; (SIDE * SIDE) as usize]).unwrap();
			let radial = position(spec, 256, 256, 0.0);
			let normal: [f64; 3] = std::array::from_fn(|i| {
				f64::from(pixels[(256 * SIDE as usize + 256) * 3 + i]) / 255.0 * 2.0 - 1.0
			});
			let dot: f64 = normal
				.iter()
				.zip(radial)
				.map(|(a, b)| a * b / f64::from(deep::RADIUS))
				.sum();
			assert!(dot > 0.99, "{lat}: {dot}");
		}
	}

	#[test]
	fn invalid_data_and_failed_batches_keep_the_previous_surface()
	{
		let spec = Spec::wanted(view(0.0, 0.0, 1024.0), true).unwrap();
		assert!(normals(spec, &[]).is_err());
		for value in [f32::NAN, f32::INFINITY, -32768.0, 1737400.0]
		{
			let mut heights = vec![0.0; (SIDE * SIDE) as usize];
			heights[42] = value;
			assert!(normals(spec, &heights).is_err());
		}
		assert!(decode(spec, b"not a TIFF").is_err());
		let mut bytes = Cursor::new(Vec::new());
		TiffEncoder::new(&mut bytes)
			.unwrap()
			.new_image::<Gray32Float>(SIDE, SIDE)
			.unwrap()
			.write_data(&vec![0.0; (SIDE * SIDE) as usize])
			.unwrap();
		assert_eq!(
			decode(spec, bytes.get_ref()).unwrap().len(),
			(SIDE * SIDE * 7) as usize
		);
		let decoded = decode(spec, bytes.get_ref()).unwrap();
		for pixel in decoded[(SIDE * SIDE * 3) as usize..].as_chunks::<4>().0
		{
			assert_eq!(pixel, &[117, 48, 255, 255]);
		}
		let ctx = egui::Context::default();
		let shared = Shared::default();
		finish(&shared, spec, Ok(vec![1, 2, 3]), &ctx);
		finish(&shared, spec, Err("network unavailable".into()), &ctx);
		assert_eq!(shared.lock().pending.as_ref().unwrap().1, [1, 2, 3]);
		let mut photos = Photos {
			spec,
			next: 0,
			remaining: 0,
			pixels: vec![],
			failed: false,
		};
		assert_eq!(photos.jobs(), [0, 1, 2, 3]);
		assert!(photos.jobs().is_empty());
		photos.failed = true;
		photos.remaining = 0;
		assert!(photos.jobs().is_empty());
	}
}
