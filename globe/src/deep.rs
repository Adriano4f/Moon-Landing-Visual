//! View-dependent WMTS quadtree selection and a bounded download scheduler.
//! EQ covers |lat| <= 77 deg, the SP/NP stereographic squares cover |lat| >= 75 deg.
mod fetch;

use crate::{assets, math};
use egui::mutex::Mutex;
use std::{
	cmp::Ordering,
	collections::{BinaryHeap, HashMap, HashSet, VecDeque},
	f32::consts::{FRAC_PI_2, PI},
	sync::Arc,
};

const EQ: &str = "https://trek.nasa.gov/tiles/Moon/EQ/LRO_WAC_Mosaic_Global_303ppd_v02/1.0.0/default/default028mm";
const SP: &str = "https://trek.nasa.gov/tiles/Moon/SP/LRO_WAC_Mosaic_SPole60_100m_v02/1.0.0/default/default028mm";
const NP: &str = "https://trek.nasa.gov/tiles/Moon/NP/LRO_WAC_Mosaic_NPole60_100m_v02/1.0.0/default/default028mm";
pub(crate) const EQ_MAX: u32 = 8;
pub(crate) const POLE_MAX: u32 = 5;
// Lunar radius and half side of the polar stereographic square, meters.
pub(crate) const RADIUS: f32 = 1737400.0;
pub(crate) const EXTENT: f32 = 1095930.0;
// Polar square ground pixel at z0, meters.
const POLE_PIXEL: f32 = 8561.953;
// Polar tiles are used poleward of 75 deg, EQ equatorward of 77 deg.
pub(crate) const POLE_LAT: f32 = 75.0 * PI / 180.0;
pub(crate) const EQ_LAT: f32 = 77.0 * PI / 180.0;
// Selected tiles (ancestors plus leaves) never exceed this.
pub(crate) const BUDGET: usize = 80;
// Refine while one texel covers more than this many screen pixels.
const SHARP: f32 = 1.0;
const MAX_FLIGHT: usize = 4;
const MAX_QUEUE: usize = 6;
const MAX_FAILED: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Tile
{
	pub z: u32,
	pub row: u32,
	pub col: u32,
	// 0 = EQ, -1 = south polar, +1 = north polar.
	pub pole: i32,
}

impl Tile
{
	// Tile grid at level z: EQ is 2^(z+1) x 2^z, polar squares are 2^z x 2^z.
	pub fn grid(pole: i32, z: u32) -> (u32, u32)
	{
		if pole == 0
		{
			(1 << (z + 1), 1 << z)
		}
		else
		{
			(1 << z, 1 << z)
		}
	}

	pub fn max_z(self) -> u32
	{
		if self.pole == 0 { EQ_MAX } else { POLE_MAX }
	}

	pub fn url(self) -> String
	{
		let (base, ext) = match self.pole
		{
			0 => (EQ, "jpg"),
			p if p < 0 => (SP, "png"),
			_ => (NP, "png"),
		};
		format!("{base}/{}/{}/{}.{ext}", self.z, self.row, self.col)
	}

	pub fn parent(self) -> Option<Tile>
	{
		(self.z > 0).then(|| Tile {
			z: self.z - 1,
			row: self.row / 2,
			col: self.col / 2,
			pole: self.pole,
		})
	}

	pub fn children(self) -> [Tile; 4]
	{
		std::array::from_fn(|i| Tile {
			z: self.z + 1,
			row: self.row * 2 + (i as u32 / 2),
			col: self.col * 2 + (i as u32 % 2),
			pole: self.pole,
		})
	}

	// Unit direction of normalized tile coordinates (u right, v down) in [0, 1]^2.
	pub fn point(self, u: f32, v: f32) -> [f32; 3]
	{
		let (w, h) = Tile::grid(self.pole, self.z);
		let x = (self.col as f32 + u) / w as f32;
		let y = (self.row as f32 + v) / h as f32;
		if self.pole == 0
		{
			return math::direction(FRAC_PI_2 - y * PI, (x - 0.5) * 2.0 * PI);
		}
		pole_direction(
			self.pole,
			(2.0 * x - 1.0) * EXTENT,
			(1.0 - 2.0 * y) * EXTENT,
		)
	}

	// Smallest and largest |lat| (radians) touched by the tile.
	pub fn abs_lat(self) -> (f32, f32)
	{
		let (w, h) = Tile::grid(self.pole, self.z);
		if self.pole == 0
		{
			let top = FRAC_PI_2 - self.row as f32 / h as f32 * PI;
			let bottom = FRAC_PI_2 - (self.row + 1) as f32 / h as f32 * PI;
			let low = if top > 0.0 && bottom < 0.0
			{
				0.0
			}
			else
			{
				top.abs().min(bottom.abs())
			};
			return (low, top.abs().max(bottom.abs()));
		}
		let side = 2.0 * EXTENT;
		let x0 = -EXTENT + self.col as f32 / w as f32 * side;
		let x1 = -EXTENT + (self.col + 1) as f32 / w as f32 * side;
		let y1 = EXTENT - self.row as f32 / h as f32 * side;
		let y0 = EXTENT - (self.row + 1) as f32 / h as f32 * side;
		let near = 0.0_f32.clamp(x0, x1).hypot(0.0_f32.clamp(y0, y1));
		let far = x0.abs().max(x1.abs()).hypot(y0.abs().max(y1.abs()));
		(rho_lat(far), rho_lat(near))
	}

	// Angular size (radians) of one texel, worst case over the tile.
	pub fn texel(self) -> f32
	{
		if self.pole == 0
		{
			return PI / (1u32 << self.z) as f32 / 256.0;
		}
		let (_, high) = self.abs_lat();
		let t = ((FRAC_PI_2 - high) / 2.0).tan();
		POLE_PIXEL / (1u32 << self.z) as f32 / RADIUS / (1.0 + t * t)
	}

	// Vertical range [v0, v1] of the tile inside its source band; EQ is clipped to |lat| <= EQ_LAT.
	pub fn span(self) -> (f32, f32)
	{
		if self.pole != 0
		{
			return (0.0, 1.0);
		}
		let h = (1u32 << self.z) as f32;
		let v = |lat: f32| ((FRAC_PI_2 - lat) / PI * h - self.row as f32).clamp(0.0, 1.0);
		(v(EQ_LAT), v(-EQ_LAT))
	}

	// Only tiles touching their own latitude band are worth drawing.
	pub fn useful(self) -> bool
	{
		let (low, high) = self.abs_lat();
		if self.pole == 0
		{
			low <= EQ_LAT
		}
		else
		{
			high >= POLE_LAT
		}
	}
}

// |lat| at polar plane radius rho (stereographic, tangent at the pole).
fn rho_lat(rho: f32) -> f32
{
	FRAC_PI_2 - 2.0 * (rho / (2.0 * RADIUS)).atan()
}

// Inverse polar stereographic: SP lon = atan2(x, y), NP lon = atan2(x, -y).
pub(crate) fn pole_direction(pole: i32, x: f32, y: f32) -> [f32; 3]
{
	let lat = rho_lat(x.hypot(y));
	if pole < 0
	{
		math::direction(-lat, x.atan2(y))
	}
	else
	{
		math::direction(lat, x.atan2(-y))
	}
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32
{
	a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(a: [f32; 3]) -> [f32; 3]
{
	let n = dot(a, a).sqrt().max(1e-12);
	[a[0] / n, a[1] / n, a[2] / n]
}

// Spherical cap containing a surface patch: points p with dot(p, center) >= cos(angle).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cap
{
	pub center: [f32; 3],
	pub angle: f32,
}

impl Cap
{
	// Conservative cap of the tile's useful span from a 9x9 sample grid plus a margin.
	pub fn of(tile: Tile) -> Cap
	{
		const N: usize = 9;
		let (v0, v1) = tile.span();
		let mut samples = Vec::with_capacity(N * N + 1);
		for i in 0..N
		{
			for j in 0..N
			{
				samples.push(tile.point(
					i as f32 / (N - 1) as f32,
					v0 + (v1 - v0) * j as f32 / (N - 1) as f32,
				));
			}
		}
		let sum = samples
			.iter()
			.fold([0.0; 3], |s, p| [s[0] + p[0], s[1] + p[1], s[2] + p[2]]);
		let center = if dot(sum, sum) < 1e-8
		{
			samples[N * N / 2]
		}
		else
		{
			normalize(sum)
		};
		let angle = samples
			.iter()
			.map(|p| dot(*p, center).clamp(-1.0, 1.0).acos())
			.fold(0.0_f32, f32::max);
		Cap {
			center,
			angle: (angle * 1.1 + 1e-3).min(PI),
		}
	}

	// Polar band poleward of POLE_LAT.
	pub fn pole(pole: i32) -> Cap
	{
		Cap {
			center: [0.0, pole.signum() as f32, 0.0],
			angle: FRAC_PI_2 - POLE_LAT,
		}
	}

	// Euclidean ball containing the cap: (center, radius).
	fn ball(self) -> ([f32; 3], f32)
	{
		if self.angle >= FRAC_PI_2
		{
			return ([0.0; 3], 1.0);
		}
		let (s, c) = self.angle.sin_cos();
		(
			[self.center[0] * c, self.center[1] * c, self.center[2] * c],
			s,
		)
	}
}

// Camera derived from View, mirroring View::camera_distance and View::projection.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Camera
{
	model: [f32; 9],
	// Eye direction in model space; the eye sits at distance * eye.
	eye: [f32; 3],
	distance: f32,
	sx: f32,
	sy: f32,
	// Screen pixels per unit of tan(angle) vertically.
	focal: f32,
	// Surface seen on screen when the whole screen is covered by the sphere.
	screen: Option<Cap>,
}

impl Camera
{
	pub fn new(view: math::View) -> Camera
	{
		let model = view.model();
		let distance = view.camera_distance();
		let p = view.projection();
		let mut camera = Camera {
			model,
			eye: [model[2], model[5], model[8]],
			distance,
			sx: p[0],
			sy: p[5],
			focal: p[5] * view.pixels[1].max(1.0) / 2.0,
			screen: None,
		};
		camera.screen = camera.screen_cap();
		camera
	}

	// Model-space surface point under normalized device coordinates (x, y), if the ray hits.
	fn hit(self, x: f32, y: f32) -> Option<[f32; 3]>
	{
		let r = normalize([x / self.sx, y / self.sy, -1.0]);
		// |o + t r|^2 = 1 with o = (0, 0, distance): t^2 + 2 b t + c = 0.
		let b = self.distance * r[2];
		let c = (self.distance - 1.0) * (self.distance + 1.0);
		let disc = b * b - c;
		if disc < 0.0
		{
			return None;
		}
		let t = c / (-b + disc.sqrt());
		let v = [t * r[0], t * r[1], self.distance + t * r[2]];
		let m = self.model;
		Some(normalize(std::array::from_fn(|j| {
			m[3 * j] * v[0] + m[3 * j + 1] * v[1] + m[3 * j + 2] * v[2]
		})))
	}

	// Cap around the screen boundary's surface image; None when some edge ray misses the sphere.
	fn screen_cap(self) -> Option<Cap>
	{
		const N: usize = 8;
		let center = self.hit(0.0, 0.0)?;
		let mut angle = 0.0_f32;
		for i in 0..=N
		{
			let a = -1.0 + 2.0 * i as f32 / N as f32;
			for (x, y) in [(a, -1.0), (a, 1.0), (-1.0, a), (1.0, a)]
			{
				let p = self.hit(x, y)?;
				angle = angle.max(dot(p, center).clamp(-1.0, 1.0).acos());
			}
		}
		Some(Cap {
			center,
			angle: angle * 1.05 + 1e-4,
		})
	}

	// Cap intersects the camera's visible hemisphere and the view frustum.
	pub fn sees(self, cap: Cap) -> bool
	{
		if let Some(screen) = self.screen
		{
			return dot(cap.center, screen.center).clamp(-1.0, 1.0).acos()
				<= cap.angle + screen.angle;
		}
		let horizon = (1.0 / self.distance).acos();
		if dot(cap.center, self.eye).clamp(-1.0, 1.0).acos() - cap.angle > horizon
		{
			return false;
		}
		let (c, r) = cap.ball();
		let m = self.model;
		let q: [f32; 3] = std::array::from_fn(|i| m[i] * c[0] + m[i + 3] * c[1] + m[i + 6] * c[2]);
		let w = self.distance - q[2];
		if w + r <= 0.0
		{
			return false;
		}
		// Side planes through the eye: s * q - w = 0 with normal (s, 0, 1) or (0, s, 1).
		let nx = (self.sx * self.sx + 1.0).sqrt();
		let ny = (self.sy * self.sy + 1.0).sqrt();
		(self.sx * q[0].abs() - w) / nx <= r && (self.sy * q[1].abs() - w) / ny <= r
	}

	// Screen pixels per texel at the nearest point of the cap.
	pub fn error(self, tile: Tile, cap: Cap) -> f32
	{
		let (c, r) = cap.ball();
		let e = [
			self.eye[0] * self.distance,
			self.eye[1] * self.distance,
			self.eye[2] * self.distance,
		];
		let d = [e[0] - c[0], e[1] - c[1], e[2] - c[2]];
		let near = (dot(d, d).sqrt() - r).max(self.distance - 1.0).max(1e-4);
		tile.texel() / near * self.focal
	}
}

struct Node
{
	error: f32,
	tile: Tile,
}

impl PartialEq for Node
{
	fn eq(&self, o: &Self) -> bool
	{
		self.cmp(o) == Ordering::Equal
	}
}

impl Eq for Node {}

impl PartialOrd for Node
{
	fn partial_cmp(&self, o: &Self) -> Option<Ordering>
	{
		Some(self.cmp(o))
	}
}

impl Ord for Node
{
	fn cmp(&self, o: &Self) -> Ordering
	{
		self.error
			.total_cmp(&o.error)
			.then_with(|| o.tile.cmp(&self.tile))
	}
}

// Visible quadtree nodes (ancestors and leaves), coarse first, at most BUDGET.
// Worst-resolved node is refined first; a node whose visible children don't fit stays a leaf.
pub(crate) fn select(view: math::View) -> Vec<Tile>
{
	let camera = Camera::new(view);
	let mut chosen = Vec::new();
	let mut heap = BinaryHeap::new();
	let push = |tile: Tile, cap: Cap, chosen: &mut Vec<Tile>, heap: &mut BinaryHeap<Node>| {
		chosen.push(tile);
		let error = camera.error(tile, cap);
		if tile.z < tile.max_z() && error > SHARP
		{
			heap.push(Node { error, tile });
		}
	};
	for col in 0..2
	{
		let tile = Tile {
			z: 0,
			row: 0,
			col,
			pole: 0,
		};
		let cap = Cap::of(tile);
		if camera.sees(cap)
		{
			push(tile, cap, &mut chosen, &mut heap);
		}
	}
	for pole in [-1, 1]
	{
		let tile = Tile {
			z: 0,
			row: 0,
			col: 0,
			pole,
		};
		if camera.sees(Cap::pole(pole))
		{
			push(tile, Cap::of(tile), &mut chosen, &mut heap);
		}
	}
	while let Some(Node { tile, .. }) = heap.pop()
	{
		let kids: Vec<(Tile, Cap)> = tile
			.children()
			.into_iter()
			.filter(|t| t.useful())
			.map(|t| (t, Cap::of(t)))
			.filter(|(t, c)| camera.sees(*c) && (t.pole == 0 || camera.sees(Cap::pole(t.pole))))
			.collect();
		if chosen.len() + kids.len() > BUDGET
		{
			continue;
		}
		for (t, c) in kids
		{
			push(t, c, &mut chosen, &mut heap);
		}
	}
	chosen.sort();
	chosen
}

#[derive(Clone, Copy, Debug)]
struct Failure
{
	attempts: u32,
	// None until the scheduler stamps it with the next retry time.
	retry: Option<f64>,
}

#[derive(Default)]
pub(crate) struct State
{
	// Visible tiles, coarse first; GPU keeps these and evicts others.
	pub wanted: Vec<Tile>,
	// Decoded images awaiting GPU upload.
	pub pending: VecDeque<(Tile, assets::TextureImage)>,
	// Tiles on the GPU; the GPU owner updates it after upload and eviction.
	pub resident: HashSet<Tile>,
	// Last download error, cleared by the next success.
	pub error: Option<String>,
	flight: HashSet<Tile>,
	failed: HashMap<Tile, Failure>,
	// Insertion order of failed, oldest first, for the bound.
	failed_order: VecDeque<Tile>,
	view_key: Option<([u32; 9], [u32; 4])>,
}

impl State
{
	fn have(&self, tile: Tile) -> bool
	{
		self.resident.contains(&tile) || self.pending.iter().any(|(t, _)| *t == tile)
	}

	// Pick new downloads from wanted, coarse first, and mark them in flight.
	// A tile waits for its parent unless the parent is present or failed.
	fn plan(&mut self, now: f64) -> Vec<Tile>
	{
		for f in self.failed.values_mut()
		{
			if f.retry.is_none()
			{
				f.retry = Some(now + (2.0 * 2f64.powi(f.attempts as i32)).min(60.0));
			}
		}
		let mut jobs = Vec::new();
		for i in 0..self.wanted.len()
		{
			if self.flight.len() >= MAX_FLIGHT
				|| self.pending.len() + self.flight.len() >= MAX_QUEUE
			{
				break;
			}
			let tile = self.wanted[i];
			if self.have(tile) || self.flight.contains(&tile)
			{
				continue;
			}
			if self
				.failed
				.get(&tile)
				.is_some_and(|f| f.retry.is_some_and(|r| r > now))
			{
				continue;
			}
			if let Some(p) = tile.parent()
				&& !self.have(p)
				&& !self.failed.contains_key(&p)
			{
				continue;
			}
			self.flight.insert(tile);
			jobs.push(tile);
		}
		jobs
	}

	fn finish(&mut self, tile: Tile, result: Result<assets::TextureImage, String>)
	{
		self.flight.remove(&tile);
		if !self.wanted.contains(&tile)
		{
			return;
		}
		match result
		{
			Ok(image) =>
			{
				self.failed.remove(&tile);
				self.failed_order.retain(|t| *t != tile);
				self.error = None;
				self.pending.push_back((tile, image));
			}
			Err(error) =>
			{
				self.error = Some(error);
				let attempts = self.failed.get(&tile).map_or(1, |f| f.attempts + 1);
				self.failed_order.retain(|t| *t != tile);
				self.failed_order.push_back(tile);
				self.failed.insert(
					tile,
					Failure {
						attempts,
						retry: None,
					},
				);
				while self.failed_order.len() > MAX_FAILED
				{
					if let Some(old) = self.failed_order.pop_front()
					{
						self.failed.remove(&old);
					}
				}
			}
		}
	}
}

pub(crate) type Shared = Arc<Mutex<State>>;

// Recompute the visible set and start at most MAX_FLIGHT downloads, pending + in flight <= MAX_QUEUE.
pub(crate) fn advance(shared: &Shared, view: math::View, ctx: &egui::Context)
{
	let key = (
		view.model().map(f32::to_bits),
		[view.zoom, view.aspect, view.pixels[0], view.pixels[1]].map(f32::to_bits),
	);
	let changed = shared.lock().view_key != Some(key);
	let wanted = changed.then(|| select(view));
	let now = ctx.input(|i| i.time);
	let jobs = {
		let mut state = shared.lock();
		if let Some(wanted) = wanted
		{
			state.pending.retain(|(tile, _)| wanted.contains(tile));
			state.wanted = wanted;
			state.view_key = Some(key);
		}
		state.plan(now)
	};
	for tile in jobs
	{
		let shared = Arc::clone(shared);
		let ctx = ctx.clone();
		fetch::image(tile.url(), move |result| {
			shared.lock().finish(tile, result);
			ctx.request_repaint();
		});
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	fn view(lat: f32, lon: f32, zoom: f32) -> math::View
	{
		// model() maps direction(lat, lon) to +z when yaw = -lon and tilt = lat.
		math::View {
			yaw: -lon,
			tilt: lat,
			zoom,
			aspect: 16.0 / 9.0,
			pixels: [1920.0, 1080.0],
			scale: 1.0,
		}
	}

	fn deg(x: f32) -> f32
	{
		x * PI / 180.0
	}

	fn tile_lat_lon(t: Tile) -> (f32, f32)
	{
		let p = t.point(0.5, 0.5);
		(p[1].clamp(-1.0, 1.0).asin(), p[0].atan2(p[2]))
	}

	fn check(tiles: &[Tile])
	{
		assert!(tiles.len() <= BUDGET, "{} tiles", tiles.len());
		let set: HashSet<Tile> = tiles.iter().copied().collect();
		assert_eq!(set.len(), tiles.len());
		for (i, t) in tiles.iter().enumerate()
		{
			if let Some(p) = t.parent()
			{
				let j = tiles.iter().position(|x| *x == p).expect("ancestor kept");
				assert!(j < i, "coarse first");
			}
			assert!(t.z <= t.max_z());
			let (w, h) = Tile::grid(t.pole, t.z);
			assert!(t.col < w && t.row < h);
		}
	}

	fn max_z(tiles: &[Tile], pole: i32) -> Option<u32>
	{
		tiles.iter().filter(|t| t.pole == pole).map(|t| t.z).max()
	}

	#[test]
	fn camera_center_matches_direction()
	{
		for (lat, lon) in [(0.0, 0.0), (30.0, 100.0), (-60.0, -150.0), (-89.0, 10.0)]
		{
			let c = Camera::new(view(deg(lat), deg(lon), 2.0));
			let d = math::direction(deg(lat), deg(lon));
			assert!(dot(c.eye, d) > 0.9999, "{lat} {lon}");
		}
	}

	#[test]
	fn projections_round_trip_shader_uv()
	{
		for pole in [-1, 1]
		{
			for (x, y) in [(3e5_f32, -2e5_f32), (-9e5, 7e5), (1.0, 1e6)]
			{
				let p = pole_direction(pole, x, y);
				let s = pole as f32;
				// SP uv = (p.x, -p.z) / (1 - p.y), NP uv = (p.x, p.z) / (1 + p.y), scaled by R/EXTENT.
				let k = 2.0 * RADIUS / (1.0 + s * p[1]);
				assert!((k * p[0] - x).abs() < 5.0, "{pole} x");
				assert!((-s * k * p[2] - y).abs() < 5.0, "{pole} y");
			}
		}
		let t = Tile {
			z: 3,
			row: 1,
			col: 9,
			pole: 0,
		};
		let (lat, lon) = tile_lat_lon(t);
		assert!((lat - deg(90.0 - 22.5 * 1.5)).abs() < 1e-4);
		assert!((lon - deg(-180.0 + 22.5 * 9.5)).abs() < 1e-4);
		assert!(t.url().ends_with(
			"/EQ/LRO_WAC_Mosaic_Global_303ppd_v02/1.0.0/default/default028mm/3/1/9.jpg"
		));
		assert!(
			Tile {
				z: 5,
				row: 2,
				col: 3,
				pole: -1
			}
			.url()
			.ends_with("SPole60_100m_v02/1.0.0/default/default028mm/5/2/3.png")
		);
		assert!(
			Tile {
				z: 1,
				row: 0,
				col: 1,
				pole: 1
			}
			.url()
			.contains("/NP/LRO_WAC_Mosaic_NPole60_100m_v02/")
		);
	}

	#[test]
	fn caps_contain_tiles()
	{
		for t in [
			Tile {
				z: 0,
				row: 0,
				col: 0,
				pole: 0,
			},
			Tile {
				z: 1,
				row: 1,
				col: 3,
				pole: 0,
			},
			Tile {
				z: 5,
				row: 0,
				col: 17,
				pole: 0,
			},
			Tile {
				z: 0,
				row: 0,
				col: 0,
				pole: -1,
			},
			Tile {
				z: 3,
				row: 3,
				col: 4,
				pole: 1,
			},
		]
		{
			let cap = Cap::of(t);
			for i in 0..=40
			{
				for j in 0..=40
				{
					let (v0, v1) = t.span();
					let p = t.point(i as f32 / 40.0, v0 + (v1 - v0) * j as f32 / 40.0);
					assert!(dot(p, cap.center) >= cap.angle.cos() - 1e-5, "{t:?}");
				}
			}
		}
	}

	#[test]
	fn overview_is_coarse_and_bounded()
	{
		let tiles = select(view(0.0, 0.0, 1.0));
		check(&tiles);
		assert!(max_z(&tiles, 0).unwrap() <= 3);
		assert!(tiles.len() >= 2);
	}

	#[test]
	fn high_zoom_refines_only_near_view()
	{
		for (lat, lon) in [
			(0.0, 0.0),
			(20.0, 179.9),
			(-40.0, -179.9),
			(60.0, 90.0),
			(0.0, 180.0),
		]
		{
			let tiles = select(view(deg(lat), deg(lon), 512.0));
			check(&tiles);
			assert_eq!(max_z(&tiles, 0), Some(EQ_MAX), "{lat} {lon}");
			let target = math::direction(deg(lat), deg(lon));
			for t in tiles.iter().filter(|t| t.z >= 3)
			{
				assert!(dot(Cap::of(*t).center, target) > 0.5, "{t:?} far from view");
			}
			// Opposite hemisphere: no deep tile there.
			assert!(
				tiles
					.iter()
					.filter(|t| t.z >= 2)
					.all(|t| dot(t.point(0.5, 0.5), target) > 0.0)
			);
			assert!(tiles.iter().all(|t| t.pole == 0), "no polar tiles at {lat}");
		}
	}

	#[test]
	fn seam_takes_both_sides()
	{
		let tiles = select(view(0.0, deg(180.0), 512.0));
		let deep: Vec<_> = tiles.iter().filter(|t| t.z == EQ_MAX).collect();
		let (w, _) = Tile::grid(0, EQ_MAX);
		assert!(deep.iter().any(|t| t.col == 0) && deep.iter().any(|t| t.col == w - 1));
	}

	#[test]
	fn south_pole_uses_polar_tree_without_eq_stripes()
	{
		for zoom in [4.0, 64.0, 1024.0]
		{
			let tiles = select(view(-FRAC_PI_2, 0.0, zoom));
			check(&tiles);
			assert!(tiles.iter().all(|t| t.pole != 1));
			if zoom >= 64.0
			{
				assert_eq!(max_z(&tiles, -1), Some(POLE_MAX), "zoom {zoom}");
			}
			for t in tiles.iter().filter(|t| t.pole == 0)
			{
				assert!(t.abs_lat().0 <= EQ_LAT, "{t:?} EQ stripe at pole");
			}
			if zoom >= 64.0
			{
				let eq: Vec<_> = tiles
					.iter()
					.filter(|t| t.pole == 0)
					.map(|t| (t, t.abs_lat()))
					.collect();
				assert!(
					max_z(&tiles, 0).unwrap_or(0) <= 1,
					"EQ refined at pole zoom {zoom}: {eq:?}"
				);
			}
		}
		let north = select(view(FRAC_PI_2, 0.0, 256.0));
		check(&north);
		assert_eq!(max_z(&north, 1), Some(POLE_MAX));
		assert!(north.iter().all(|t| t.pole != -1));
	}

	#[test]
	fn band_edge_loads_both_sources()
	{
		let tiles = select(view(deg(-76.0), deg(30.0), 128.0));
		check(&tiles);
		assert!(max_z(&tiles, -1).unwrap_or(0) >= 3);
		assert!(max_z(&tiles, 0).unwrap_or(0) >= 5);
	}

	#[test]
	fn high_latitude_stays_bounded()
	{
		for lat in [70.0_f32, 76.9, -76.9, 89.0, -89.9]
		{
			for zoom in [2.0, 8.0, 40.0, 300.0, 1024.0]
			{
				for aspect in [0.4, 16.0 / 9.0, 3.0]
				{
					let mut v = view(deg(lat), deg(45.0), zoom);
					v.aspect = aspect;
					check(&select(v));
				}
			}
		}
	}

	#[test]
	fn zoom_in_refines_and_zoom_out_coarsens()
	{
		let mut last = 0;
		for zoom in [1.0, 4.0, 16.0, 64.0, 256.0]
		{
			let z = max_z(&select(view(deg(10.0), deg(20.0), zoom)), 0).unwrap();
			assert!(z >= last);
			last = z;
		}
		assert!(last > 5);
		let back = select(view(deg(10.0), deg(20.0), 1.0));
		assert!(max_z(&back, 0).unwrap() <= 3);
		assert_eq!(back, select(view(deg(10.0), deg(20.0), 1.0)));
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn image() -> assets::TextureImage
	{
		assets::TextureImage::new(1, 1)
	}

	#[cfg(not(target_arch = "wasm32"))]
	#[test]
	fn scheduler_is_bounded_coarse_first_and_retries_with_cooldown()
	{
		let mut state = State::default();
		state.wanted = select(view(deg(10.0), deg(20.0), 256.0));
		let jobs = state.plan(0.0);
		assert!(!jobs.is_empty() && jobs.len() <= MAX_FLIGHT);
		assert!(jobs.iter().all(|t| t.z == 0));
		assert!(state.plan(0.0).is_empty() || state.flight.len() <= MAX_FLIGHT);
		for t in jobs.clone()
		{
			state.finish(t, Ok(image()));
		}
		let next = state.plan(0.0);
		assert!(next.iter().all(|t| t.z == 1));
		assert!(state.pending.len() + state.flight.len() <= MAX_QUEUE);
		while let Some((t, _)) = state.pending.pop_front()
		{
			state.resident.insert(t);
		}
		for t in next.clone()
		{
			state.finish(t, Err("HTTP 404".into()));
		}
		assert!(state.error.is_some());
		let again = state.plan(1.0);
		assert!(again.iter().all(|t| !next.contains(t)));
		state.flight.clear();
		let later = state.plan(100.0);
		assert!(later.iter().any(|t| next.contains(t)));
		for i in 0..200
		{
			state.finish(
				Tile {
					z: 8,
					row: i,
					col: 0,
					pole: 0,
				},
				Err("x".into()),
			);
		}
		assert!(state.failed.len() <= MAX_FAILED && state.failed_order.len() <= MAX_FAILED);
		state.resident.extend(state.wanted.iter().copied());
		state.flight.clear();
		assert!(state.plan(1000.0).is_empty());
	}
}
