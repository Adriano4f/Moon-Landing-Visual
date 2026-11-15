# Portable apparent ephemeris

## 1. Scope and provenance

`core/src/ephemeris.rs` supplies I/O-free finite-distance sky geometry for arbitrary lunar latitude, east longitude and radial altitude. `prep --ephemeris` prepares the compact input from the [NASA/JPL Horizons API](https://ssd-api.jpl.nasa.gov/doc/horizons.html). The current application still uses its original seven prepared tracks and dates; arbitrary-point terrain and timeline controls are not part of this foundation. The new asset is copied into the static distribution but is not requested at startup.

The committed `app/assets/data/moon-ephemeris.bin` contains Sun and Earth positions every hour from 2027-01-01 00:00 UTC through 2035-01-01 00:00 UTC, inclusive: 70,129 samples per body. `moon-ephemeris.meta.json` records its frame, method, byte count and per-site validation maxima. Preparation also writes `moon-ephemeris.initial.bin`, a 56-byte versioned file containing exactly its first epoch. The application embeds that real initial Sun/Earth pair for a dated preview, without fetching the complete series or inventing terrain visibility. Original API responses remain in the external cache, including their DE441 source diagnostics. The independently prepared catalog tracks, terrain profiles and rounded-coordinate baseline are unchanged.

## 2. Query strategy and reproduction

Run from the repository root:

```sh
cargo run -p prep --release -- --ephemeris app/assets/data/moon-ephemeris.bin /home/ubuntu/data/horizons-moon-cache app/assets/data
```

The positional arguments are the output binary, an external raw-response cache, and the directory containing the seven independent reference JSON files. The fixed catalog is read from `sites.json`. A missing or incompatible reference prevents publication. Existing raw responses are reused, but reparsed and revalidated. Unexpected fresh responses are retained as `.failed.txt` diagnostics rather than accepted cache entries. HTTP errors propagate without retries; each request has a 180 s timeout.

Each target, Sun `10` and Earth `399`, uses eight inclusive annual requests. A leap-year request contains 8,785 rows, comfortably within the accepted service size. Annual boundary duplicates are removed only after exact hourly cadence and endpoints have been checked. No dates or gaps are silently invented.

| Parameter | Value |
|:--|:--|
| Table | `EPHEM_TYPE='OBSERVER'`, `MAKE_EPHEM='YES'` |
| Observer | `CENTER='coord@301'`, `COORD_TYPE='GEODETIC'`, `SITE_COORD='0,0,0'` |
| Quantities | `4,20`: apparent azimuth/elevation and apparent range |
| Conventions | `ANG_FORMAT='DEG'`, `APPARENT='AIRLESS'`, `EXTRA_PREC='YES'` |
| Format/cadence | `CSV_FORMAT='YES'`, `STEP_SIZE='1h'` |
| Checked diagnostics | Moon `301`, requested target, `MEAN_ME (high precision)`, zero reference coordinates, three 1,737.4 km lunar radii |

The returned range is converted from AU with exactly 149,597,870,700 m per AU. Horizons rejects a geodetic altitude of -1,737.4 km: its permitted observer altitude must be greater than -20 km. Consequently, this pipeline does not pretend that an observer table was returned at the Moon's center, nor apply an unverified inertial/PA-to-ME rotation. Instead, a valid surface reference observer reconstructs an apparent Moon-centered position. This method is empirically checked against the original topocentric tracks below.

## 3. Geometry and interpretation

The right-handed Moon-fixed axes are X toward 90 degrees east, Y toward north, and Z toward longitude zero. For latitude $\phi$ and east longitude $\lambda$:

$$\begin{aligned}
\mathbf U&=(\cos\phi\sin\lambda,\sin\phi,\cos\phi\cos\lambda),\\
\mathbf E&=(\cos\lambda,0,-\sin\lambda),\\
\mathbf N&=(-\sin\phi\sin\lambda,\cos\phi,-\sin\phi\cos\lambda).
\end{aligned}$$

At the reference point, east is positive X, north is positive Y, and up is positive Z. With apparent azimuth $A$, elevation $e$, range $\Delta$ and $R=1{,}737{,}400\ \mathrm m$:

$$\mathbf P=(\Delta\cos e\sin A,\ \Delta\cos e\cos A,\ R+\Delta\sin e).$$

Store the unit direction $\mathbf q=\mathbf P/\lVert\mathbf P\rVert$ and distance $D=\lVert\mathbf P\rVert$ as `f32`; renormalize the direction when reconstructing in `f64`. For an arbitrary observer with radial altitude $h$ above the reference sphere:

$$\mathbf s=D\frac{\mathbf q}{\lVert\mathbf q\rVert}-(R+h)\mathbf U,$$

$$A=\mathrm{atan2}(\mathbf s\cdot\mathbf E,\mathbf s\cdot\mathbf N)\pmod{2\pi},$$

$$e=\mathrm{atan2}\left(\mathbf s\cdot\mathbf U,\sqrt{(\mathbf s\cdot\mathbf E)^2+(\mathbf s\cdot\mathbf N)^2}\right).$$

`topocentric` returns degrees. Height must include terrain plus instrument elevation, not just mast height. Finite-distance subtraction retains Earth parallax. Longitude wraps; latitude must be within -90 through +90 degrees. Coincidence, nonfinite values and nonpositive observer radii are errors. At a zenith, azimuth is undefined geometrically.

These are apparent positions transported from one reference observer, not inertial geometric state vectors. The algorithm does not independently recompute observer-dependent light time, aberration or deflection. Agreement is established only for the seven south-polar reference locations and the original three-year overlap. Global locations and the later five-year extension have not undergone an independent topocentric comparison. The returned `MEAN_ME` diagnostic is verified, but not every historical ME/PA frame realization is asserted identical.

## 4. Binary format and validation

All numeric fields are little-endian. No platform alignment or native struct layout is serialized.

| Offset, bytes | Length, bytes | Meaning |
|--:|--:|:--|
| 0 | 8 | ASCII `MLVEPH01` |
| 8 | 8 | Signed 64-bit start UTC Unix seconds |
| 16 | 4 | Unsigned 32-bit step, seconds |
| 20 | 4 | Unsigned 32-bit sample count |
| 24 | 32 per epoch | Sun X/Y/Z/distance, then Earth X/Y/Z/distance; eight `f32` values |

Total size is $24+70{,}129\times32=2{,}244{,}152$ bytes, or approximately 2.24 MB. Encoding and decoding reject empty data, zero cadence, wrong lengths, timestamp overflow, invalid ranges, and directions whose norms differ from unity by more than 0.0001. The decoder rejects invalid magic and nonfinite payloads before exposing the data. Structural validation has no checksum and cannot detect every bit alteration. Source control identifies the published bytes.

The 16 fresh Horizons requests, parsing, quantization, decoded-data comparison and file writes took 7.396 s on the preparation host. This is one measured run, not a service-performance guarantee. The binary alone adds 2,244,152 bytes to deployment storage and no requests or bytes to initial application loading.

## 5. Accuracy checks

The decoded binary is compared against all seven independently requested site tracks at every original hourly timestamp: 26,305 epochs per site, 184,135 site epochs, or 368,270 body directions. The observer altitude uses the original DEM surface height plus the original 2 m instrument height. Azimuth errors are wrapped to the shortest difference; elevation errors are ordinary absolute differences.

| Site | Sun max azimuth, degrees | Sun max elevation, degrees | Earth max azimuth, degrees | Earth max elevation, degrees |
|:--|--:|--:|--:|--:|
| Connecting Ridge C1 | 0.00001791 | 0.00000073 | 0.00001302 | 0.00003013 |
| Connecting Ridge C2 | 0.00001717 | 0.00000068 | 0.00000943 | 0.00003032 |
| Shackleton S1 | 0.00001787 | 0.00000063 | 0.00001321 | 0.00003007 |
| Shackleton S2 | 0.00001747 | 0.00000071 | 0.00001297 | 0.00003013 |
| Shackleton Mazarico | 0.00001780 | 0.00000063 | 0.00001333 | 0.00003026 |
| Connecting Ridge Speyerer | 0.00001798 | 0.00000065 | 0.00001340 | 0.00003029 |
| Shackleton Speyerer | 0.00001774 | 0.00000065 | 0.00001327 | 0.00003019 |

The command refuses publication if either angular component reaches 0.01 degrees for Sun or 0.02 degrees for Earth. These are consistency tests of frame transport and compact storage, not independent accuracy estimates for the Horizons planetary ephemeris.

Unit tests cover analytic zenith, geographic-north tangent, finite-distance parallax, nontrivial latitude/longitude, negative terrain altitude, longitude wrapping, invalid observers/bodies, binary ordering/round trips, empty/corrupt data and overflow. Preparation tests cover malformed CSV, wrong frame/body/radius/reference location, missing rows, off-grid epochs and validation failures. The full reference comparison also runs as part of actual preparation.

## 6. Callable and declaration index

- `abs`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `and_then`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `and_utc`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `any`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `as_chunks`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `assert!`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `assert_eq!`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `atan`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `atan2`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `axes_parallax_and_numeric_direction`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `binary_round_trip_empty_and_corrupt`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `BodyPosition`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `BodyPosition::from_cartesian`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `build`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `cartesian`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `chain`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `check_header`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `checked_add`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `checked_mul`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `checked_sub`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `clear`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `clone`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `collect`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `contains`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `copy_from_slice`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `csv_frame_cadence_and_errors`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `decode`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `display`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Duration::from_secs`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `encode`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `enumerate`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Ephemeris`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `Ephemeris::decode`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `epoch`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `error_for_status`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `extend_from_slice`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `f32::from_le_bytes`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `f64::from`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `fill`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `filter`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `find`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `fixture`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `format!`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `from_cartesian`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `fs::create_dir_all`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `fs::read`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `fs::rename`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `fs::write`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `get`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `hypot`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `i64::from`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `i64::from_le_bytes`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `i64::try_from`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `initial_sample_normal_empty_and_invalid`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Instant::now`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `into`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `into_iter`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `invalid_positions_and_observers`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `is_empty`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `is_err`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `is_finite`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `is_none`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `iter`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `join`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `json!`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `len`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `lines`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `map`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `map_err`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `matches!`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `max`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `NaiveDateTime::parse_from_str`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `next`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `nth`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `ok`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `ok_or`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `parse`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `parse_chunk`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Path::new`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `println!`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `push`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `query`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `reference`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `reference_position`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `reference_validation_normal_empty_and_error`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `rem_euclid`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `replace`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `reqwest::blocking::Client::builder`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `run`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `send`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `serde_json::from_slice`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `sin_cos`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `skip`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `split`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `split_once`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `split_whitespace`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `sqrt`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `starts_with`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `std::array::from_fn`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `std::env::args`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `sum`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `super::read_sites`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `super::slug`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `take`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `text`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `timeout`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `timestamp`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `to_degrees`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `to_le_bytes`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs)
- `to_radians`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `to_string`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `topocentric`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `trim`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `try_into`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `u32::from_le_bytes`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `u32::try_from`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `unwrap`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `unwrap_or`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `validate_all`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `validate_site`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `vec!`: [annotated core/src/ephemeris_tests.rs](#72-coresrcephemeris_testsrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Vec::new`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `Vec::with_capacity`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs)
- `with_extension`: [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)
- `zip`: [annotated core/src/ephemeris.rs](#71-coresrcephemerisrs), [annotated prep/src/ephemeris.rs](#73-prepsrcephemerisrs)

## 7. Exact annotated sources

### 7.1. core/src/ephemeris.rs

```rust
//! Moon mean-Earth apparent positions, expressed as X=90E, Y=north, Z=0E.

// Reuse the validated spherical reference rather than another radius; C++ shared constant.
use crate::MOON_RADIUS_M;

// Store one compact Moon-centered position; C++ aggregate of four floats.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPosition
{
	// Unit Moon-fixed direction, renormalized after quantization; C++ array<float,3>.
	pub direction: [f32; 3],
	// Moon-center range in meters; C++ float.
	pub distance_m: f32,
}

impl BodyPosition
{
	// Quantize only the final Moon-centered representation; C++ checked factory.
	pub fn from_cartesian(position_m: [f64; 3]) -> Result<Self, String>
	{
		// Compute the range in double precision; C++ Euclidean norm.
		let distance = position_m.iter().map(|x| x * x).sum::<f64>().sqrt();
		// Reject invalid and unrepresentable positions before division; C++ guard.
		if !distance.is_finite() || distance <= MOON_RADIUS_M || distance > f64::from(f32::MAX)
		{
			return Err("Invalid Moon-centered body position".into());
		}
		// Preserve direction and distance separately to retain angular precision; C++ aggregate return.
		let body = Self {
			direction: position_m.map(|x| (x / distance) as f32),
			distance_m: distance as f32,
		};
		// Quantization must not round an otherwise valid distance onto the excluded sphere; C++ invariant check.
		body.cartesian()?;
		Ok(body)
	}

	// Validate stored values and reconstruct a normalized Cartesian position; C++ expected<array<double,3>>.
	pub fn cartesian(self) -> Result<[f64; 3], String>
	{
		// Convert before arithmetic to avoid float norm overflow; C++ array transform.
		let direction = self.direction.map(f64::from);
		let norm = direction.iter().map(|x| x * x).sum::<f64>().sqrt();
		// Reject corrupted directions rather than silently repairing arbitrary vectors; C++ predicate.
		if !norm.is_finite()
			|| (norm - 1.0).abs() > 1e-4
			|| !self.distance_m.is_finite()
			|| f64::from(self.distance_m) <= MOON_RADIUS_M
		{
			return Err("Invalid ephemeris direction or distance".into());
		}
		// Remove unit-vector quantization drift independently of range; C++ normalized array return.
		Ok(direction.map(|x| x / norm * f64::from(self.distance_m)))
	}
}

// Convert one finite-distance body position to north-through-east azimuth/elevation; C++ fallible geometry function.
pub fn topocentric(
	body: BodyPosition,
	lat_deg: f64,
	lon_deg: f64,
	height_m: f64,
) -> Result<(f64, f64), String>
{
	// Heights are radial altitude above the sphere, including terrain and instrument; C++ input guard.
	if !lat_deg.is_finite()
		|| !(-90.0..=90.0).contains(&lat_deg)
		|| !lon_deg.is_finite()
		|| !height_m.is_finite()
		|| height_m <= -MOON_RADIUS_M
	{
		return Err("Invalid observer coordinates or radial altitude".into());
	}
	// Construct the same right-handed ENU basis as the globe; C++ trigonometric initialization.
	let (sp, cp) = lat_deg.to_radians().sin_cos();
	let (sl, cl) = lon_deg.rem_euclid(360.0).to_radians().sin_cos();
	let up = [cp * sl, sp, cp * cl];
	let east = [cl, 0.0, -sl];
	let north = [-sp * sl, cp, -sp * cl];
	// Subtract the actual observer, retaining Earth parallax; C++ vector difference.
	let position = body.cartesian()?;
	let relative =
		std::array::from_fn::<_, 3, _>(|i| position[i] - (MOON_RADIUS_M + height_m) * up[i]);
	// Coincident or overflowed positions have no sky direction; C++ finite nonzero guard.
	let distance = relative.iter().map(|x| x * x).sum::<f64>().sqrt();
	if !distance.is_finite() || distance < 1e-6
	{
		return Err("Body coincides with observer or exceeds numeric range".into());
	}
	// Project onto local axes, then use robust quadrant-preserving angles; C++ dot products.
	let dot = |axis: [f64; 3]| relative.iter().zip(axis).map(|(a, b)| a * b).sum::<f64>();
	let (e, n, u) = (dot(east), dot(north), dot(up));
	Ok((
		e.atan2(n).to_degrees().rem_euclid(360.0),
		u.atan2(e.hypot(n)).to_degrees(),
	))
}

// A versioned uniformly sampled pair: Sun first, Earth second; C++ owning value type.
#[derive(Debug, PartialEq)]
pub struct Ephemeris
{
	// Inclusive first UTC epoch in Unix seconds; C++ int64_t.
	pub start_utc: i64,
	// Positive sample spacing in elapsed UTC seconds; C++ uint32_t.
	pub step_s: u32,
	// Unit directions and ranges at each epoch; C++ vector<array<BodyPosition,2>>.
	pub samples: Vec<[BodyPosition; 2]>,
}

impl Ephemeris
{
	// Resolve an index without overflow or extrapolation; C++ optional<int64_t>.
	pub fn epoch(&self, index: usize) -> Option<i64>
	{
		// Public fields must not permit out-of-range timestamp construction; C++ guard.
		if index >= self.samples.len() || self.step_s == 0
		{
			return None;
		}
		// Check both multiplication and addition; C++ checked integer arithmetic.
		self.start_utc.checked_add(
			i64::try_from(index)
				.ok()?
				.checked_mul(i64::from(self.step_s))?,
		)
	}

	// Decode explicitly little-endian values without alignment or unsafe assumptions; C++ byte reader.
	pub fn decode(bytes: &[u8]) -> Result<Self, String>
	{
		// Validate magic and header before indexing; C++ size/version guard.
		if bytes.len() < 24 || &bytes[..8] != b"MLVEPH01"
		{
			return Err("Invalid ephemeris header".into());
		}
		// Fixed-width header slices are now proven present; C++ little-endian reads.
		let start_utc = i64::from_le_bytes(bytes[8..16].try_into().unwrap());
		let step_s = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
		let count = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
		// Compare without overflow on wasm32; C++ checked payload-length arithmetic.
		if count == 0 || step_s == 0 || bytes.len().checked_sub(24) != count.checked_mul(32)
		{
			return Err("Invalid ephemeris cadence or payload length".into());
		}
		// Decode only complete records and validate every body; C++ range transform.
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
		// Reject timestamps that overflow the serialized UTC domain; C++ checked value construction.
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

	// Serialize the portable format, validating even manually constructed values; C++ binary encoder.
	pub fn encode(&self) -> Result<Vec<u8>, String>
	{
		// Bound counts and total allocation before writing; C++ checked conversion.
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
		// Emit the versioned header with explicit endian order; C++ byte-vector appends.
		let mut bytes = Vec::with_capacity(size);
		bytes.extend_from_slice(b"MLVEPH01");
		bytes.extend_from_slice(&self.start_utc.to_le_bytes());
		bytes.extend_from_slice(&self.step_s.to_le_bytes());
		bytes.extend_from_slice(&count.to_le_bytes());
		// Preserve Sun/Earth and component ordering exactly; C++ nested range loops.
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

// Keep independent geometry and malformed-input fixtures out of runtime builds; C++ test translation unit.
#[cfg(test)]
#[path = "ephemeris_tests.rs"]
mod tests;
```

### 7.2. core/src/ephemeris_tests.rs

```rust
// Exercise only the portable core API; C++ scientific test includes.
use super::*;

#[test]
fn axes_parallax_and_numeric_direction()
{
	// A body on the prime-meridian axis is exactly overhead at zero latitude/longitude; C++ fixture.
	let zenith = BodyPosition {
		direction: [0.0, 0.0, 1.0],
		distance_m: 384400000.0,
	};
	assert_eq!(topocentric(zenith, 0.0, 0.0, 2.0).unwrap(), (0.0, 90.0));
	// A body on the east axis has the analytic finite-distance horizon depression; C++ parallax control.
	let east = BodyPosition {
		direction: [1.0, 0.0, 0.0],
		..zenith
	};
	let (az, el) = topocentric(east, 0.0, 0.0, 2.0).unwrap();
	assert!((az - 90.0).abs() < 1e-10);
	assert!((el + ((MOON_RADIUS_M + 2.0) / 384400000.0).atan().to_degrees()).abs() < 1e-10);
	// Independent ENU components for latitude 30 and longitude 45; C++ analytic numeric fixture.
	let (az, el) = topocentric(east, 30.0, 45.0, 10.0).unwrap();
	let expected_el = (384400000.0 * 0.375_f64.sqrt() - MOON_RADIUS_M - 10.0)
		.atan2(384400000.0 * 0.625_f64.sqrt())
		.to_degrees();
	assert!((az - 116.565051177078).abs() < 1e-10 && (el - expected_el).abs() < 1e-10);
	// Geographic north and longitude wrapping use the same basis; C++ axis assertions.
	let north = BodyPosition {
		direction: [0.0, 1.0, 0.0],
		..zenith
	};
	assert_eq!(topocentric(north, 0.0, 360.0, 0.0).unwrap().0, 0.0);
	// Include the observer position to construct an exactly northward tangent; C++ analytic zero-elevation control.
	let tangent = BodyPosition::from_cartesian([0.0, 3e8, MOON_RADIUS_M]).unwrap();
	let (az, el) = topocentric(tangent, 0.0, 0.0, 0.0).unwrap();
	assert_eq!(az, 0.0);
	assert!(el.abs() < 1e-6);
	assert!(topocentric(zenith, -90.0, 222.83, -1000.0).unwrap().1 < 0.0);
}

#[test]
fn invalid_positions_and_observers()
{
	// Empty/nonfinite Cartesian data cannot create usable directions; C++ invalid-input table.
	for p in [
		[0.0; 3],
		[f64::NAN, 1.0, 0.0],
		[f64::MAX; 3],
		[0.0, 0.0, MOON_RADIUS_M + 0.0001],
	]
	{
		assert!(BodyPosition::from_cartesian(p).is_err());
	}
	let body = BodyPosition::from_cartesian([3e8, 4e8, 0.0]).unwrap();
	// Preserve direction while quantizing only its compact representation; C++ round-trip tolerance.
	assert!((body.cartesian().unwrap()[0] - 3e8).abs() < 20.0);
	for (lat, lon, h) in [
		(91.0, 0.0, 0.0),
		(0.0, f64::NAN, 0.0),
		(0.0, 0.0, -MOON_RADIUS_M),
		(0.0, 0.0, f64::INFINITY),
	]
	{
		assert!(topocentric(body, lat, lon, h).is_err());
	}
	for invalid in [
		BodyPosition {
			distance_m: 0.0,
			..body
		},
		BodyPosition {
			direction: [0.0; 3],
			..body
		},
		BodyPosition {
			direction: [f32::NAN; 3],
			..body
		},
	]
	{
		assert!(invalid.cartesian().is_err());
	}
	// A body at the observer cannot be projected; C++ coincidence error fixture.
	let same = BodyPosition {
		direction: [0.0, 0.0, 1.0],
		distance_m: 2000000.0,
	};
	assert!(topocentric(same, 0.0, 0.0, 2000000.0 - MOON_RADIUS_M).is_err());
}

#[test]
fn binary_round_trip_empty_and_corrupt()
{
	// Two bodies and epochs independently exercise record ordering; C++ serialization fixture.
	let body = BodyPosition::from_cartesian([3e8, 4e8, 0.0]).unwrap();
	let data = Ephemeris {
		start_utc: 1798761600,
		step_s: 3600,
		samples: vec![[body, body]; 2],
	};
	let bytes = data.encode().unwrap();
	assert_eq!(bytes.len(), 24 + 64);
	assert_eq!(Ephemeris::decode(&bytes).unwrap(), data);
	assert_eq!(data.epoch(1), Some(1798765200));
	assert_eq!(data.epoch(2), None);
	// Reject truncated headers/payloads and unsupported versions; C++ error fixtures.
	for n in [0, 7, 23, 24, bytes.len() - 1]
	{
		assert!(Ephemeris::decode(&bytes[..n]).is_err());
	}
	for (offset, value) in [(0, 0), (16, 0), (20, 0)]
	{
		let mut bad = bytes.clone();
		if offset == 0
		{
			bad[0] = value;
		}
		else
		{
			bad[offset..offset + 4].fill(value);
		}
		assert!(Ephemeris::decode(&bad).is_err());
	}
	// Nonfinite payloads and integer-time overflow must fail before use; C++ corruption cases.
	let mut bad = bytes.clone();
	bad[24..28].copy_from_slice(&f32::NAN.to_le_bytes());
	assert!(Ephemeris::decode(&bad).is_err());
	assert!(
		Ephemeris {
			samples: vec![],
			..data
		}
		.encode()
		.is_err()
	);
	assert!(
		Ephemeris {
			start_utc: i64::MAX,
			..data
		}
		.encode()
		.is_err()
	);
}
```

### 7.3. prep/src/ephemeris.rs

```rust
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
```
