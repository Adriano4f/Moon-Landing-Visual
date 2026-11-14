# 02: Full code walkthrough

This document describes the current source, including the geographic-north terrain-ray correction. Source quotations preserve the files' tabs, brace placement, spelling, and numeric literals. Prose writes the lunar radius as 1,737,400 m; Rust writes the same value with underscores.

## 0. Reading contract and notation

Read [the build vocabulary](01-skeleton.md#1-build-vocabulary-before-syntax) and [the Rust vocabulary](01-skeleton.md#2-rust-vocabulary-in-dependency-order) first. They define packages, crates, workspaces, dependencies, targets, TOML, modules, paths, functions, traits, ownership, borrowing, slices, closures, iterators, attributes, macros, and asynchronous execution. The walkthrough assumes these definitions rather than teaching them again.

A method is a function called through a receiver with dot syntax. An associated function is called through a type path. An API is the public interface of a library. A signature describes inputs and output. In the explanations below, "takes" includes the receiver. A shared borrow does not transfer ownership; a consuming method does. See [ownership and references](01-skeleton.md#23-data-types-and-ownership).

For quick decoding: fn defines a function; pub exports it; let binds an immutable local; mut permits reassignment. A colon introduces a type; an arrow introduces a return type. A final expression without a semicolon supplies the return value. Braces delimit blocks. impl groups associated functions and methods, or implements a trait. self is the receiver. UTF-8 encodes Unicode text as bytes. String is owned UTF-8 text; str is its borrowed-view type. Vec is an owned contiguous sequence. usize is the target's pointer-sized unsigned integer; u32 is unsigned 32-bit; i64 is signed 64-bit; f32 and f64 are IEEE binary32 and binary64 floating-point. char holds one Unicode scalar value. A scalar value excludes Unicode surrogate code points.

Option represents absence with None and presence with Some. Result represents success with Ok and a recoverable error with Err. A let-else pattern extracts a variant or leaves the control path. The question-mark operator propagates None from an Option function, or an Err from a Result function with a supported error conversion. It does not catch panics. A panic stops ordinary execution, usually by unwinding natively or trapping on this wasm target. A Box owns a heap allocation; dyn Error allows runtime-dispatched error values. See [errors and traits](01-skeleton.md#24-optional-values-errors-and-traits).

An iterator produces items on demand. A closure is an anonymous callable. Vertical bars delimit its parameters; captures are inferred. A trait supplies operations for implementing types. A future represents work that can suspend; await suspends the current future rather than blocking the browser thread. See [closures](01-skeleton.md#25-closures-iterators-and-macros) and [conditional compilation](01-skeleton.md#26-source-selection-tests-and-asynchronous-code).

Operators used in the quotations have these meanings:

- Arithmetic +, -, *, /, and remainder % follow operand types. Floating-point zero division yields infinities or NaN rather than an integer-style division panic. NaN means "not a number"; infinity is also non-finite. Ordered comparisons with NaN are false, and NaN is unequal to itself.
- Unary - negates. ! negates a boolean. && and || short-circuit, so the right side is evaluated only when needed. This matters for bounds validation.
- = assigns or initializes; += and *= update a mutable place. ==, !=, <, <=, >, and >= compare values. Vec equality compares elements, not allocation addresses.
- & borrows; &mut borrows exclusively; * dereferences in expression position. A reference pattern such as &az instead copies the referenced number into a binding. A tuple pattern unpacks positional fields.
- Square brackets index or slice. Out-of-range indexing panics. String indices are byte offsets and must be UTF-8 boundaries. a..b excludes b, a..=b includes b, and an omitted range endpoint uses the sequence boundary.
- as is a numeric cast here. Integer-to-float casts may round. Float-to-integer casts truncate toward zero and saturate at the target bounds; NaN becomes zero. This differs from C++ static_cast for out-of-range floating input.
- :: qualifies a path. The extra :: before generic arguments in an expression disambiguates parsing. An underscore in a type asks for inference; an underscore pattern discards a value. An initial underscore in a variable name suppresses unused-variable warnings.
- Parentheses group expressions, call functions, or form tuples. () is also the unit value. A trailing comma in a parameter list or literal is permitted.
- #[...] attaches an attribute to the next item; #![...] attaches one to the enclosing crate. /// is an API documentation comment; // is an ordinary comment.
- A macro name ends in !. Macro token input is expanded as Rust syntax, not C++ textual substitution. A single-quoted value is a character; a double-quoted value is a string. The escape \n is a newline.

TOML uses table headers, key/value assignments, arrays, and inline tables. JSON uses objects, arrays, strings, numbers, and null. Both formats are data, not Rust expressions. HTML is a nested document language; CSS gives its presentation. JavaScript is the browser's scripting language. A DOM is the browser's object representation of that document. These terms are needed for the last files.

API descriptions were checked against the [Rust standard library](https://doc.rust-lang.org/std/), [Chrono 0.4.45](https://docs.rs/chrono/0.4.45/chrono/), [reqwest 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/blocking/), [Serde JSON](https://docs.rs/serde_json/1.0.154/serde_json/), [eframe 0.37](https://docs.rs/eframe/0.37.0/eframe/), [egui 0.37](https://docs.rs/egui/0.37.0/egui/), [web-sys](https://docs.rs/web-sys/0.3.107/web_sys/), and the specific references below. The standard-library site may display a newer compiler release. Only APIs used by this source are described. For example, Vec::with_capacity is not called anywhere in this revision.

<a id="s1"></a>
## 1. Root Cargo.toml

```toml
[workspace]
members = ["core", "prep", "app"]
resolver = "3"
```

1. The workspace table groups packages; there is no root package.
2. members lists the three directories Cargo should build together.
3. Resolver version 3 selects Cargo's modern dependency resolution, including Rust-version-aware fallback. It is not the language edition. The packages select that independently.

Cargo.lock fixes resolved dependency versions. The manifests specify compatible version requirements. A Cargo package can contain both a library and an executable, which app does. lunar_core is deliberately not named core, because Rust already provides a core crate.

<a id="s2"></a>
## 2. rustfmt.toml

```toml
hard_tabs = true
brace_style = "AlwaysNextLine"
control_brace_style = "AlwaysNextLine"
```

1. hard_tabs makes indentation use tabs.
2. brace_style requests a new line before item braces.
3. control_brace_style requests new lines for control-flow braces.

The brace options are unstable rustfmt configuration and require a nightly formatter to take effect. Stable rustfmt can warn and ignore them. They do not affect compilation. Exact source quotations below retain existing formatting, including struct literals and closures whose braces remain on their opening line.

<a id="s3"></a>
## 3. core/Cargo.toml

```toml
[package]
name = "lunar_core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
```

1. The package table defines lunar_core, version 0.1.0, using Rust edition 2024. This is a library because src/lib.rs exists.
2. The dependencies table adds Serde version 1 with its derive feature. Serde supplies Serialize and Deserialize traits and generated implementations, but no particular text format.
3. Chrono 0.4 provides calendar parsing and UTC timestamps.
4. No HTTP client or UI package is a core dependency. The scientific library can run offline.

<a id="s4"></a>
## 4. core/src/lib.rs

<a id="s4-1"></a>
### 4.1 Imports, constants, and the first three structs

Before reading the quotation: derive generates implementations of the listed traits at compile time. Debug permits developer formatting; Serialize permits encoding through Serde; Deserialize permits decoding. Default supplies an initial value. Vec defaults to an empty vector, so the derived Track default has three empty vectors. These traits resemble generated C++ formatting/serialization helpers and default constructors, not runtime reflection.

```rust
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
```

1. use imports NaiveDateTime and the two Serde traits into scope. The brace group imports two names from one module.
2. MOON_RADIUS_M is a public f64 constant, equal to 1,737,400 m. SUN_RADIUS_DEG is an f32 constant, 0.267 degrees. const values are compile-time values, closest to C++ constexpr constants.
3. Site stores an owned name, latitude, and longitude. Degrees are explicit in field names. Public fields allow caller construction without a constructor.
4. Track stores UTC Unix seconds and paired azimuth/elevation vectors. UTC is the civil time standard. Unix seconds count from 1970-01-01 00:00:00 UTC without representing leap seconds as additional Unix seconds. Index i identifies one observation across all three vectors.
5. Its documentation defines azimuth 0 as north, increasing clockwise to east. The type itself does not enforce equal lengths or finite values.
6. Horizon stores one vector of elevations. With n bins, bin i is at azimuth 360i/n. A horizon elevation is the terrain angle against which a body's elevation is compared.

<a id="s4-2"></a>
### 4.2 impl Horizon and Horizon::at

New APIs, verified in the [slice](https://doc.rust-lang.org/std/primitive.slice.html) and [floating-point](https://doc.rust-lang.org/std/primitive.f32.html) documentation:

- is_empty takes a borrowed sequence and returns whether its length is zero. For Vec this is like C++ vector::empty. For strings later it tests whether there are zero bytes.
- len takes a borrowed sequence and returns its length as usize. For a vector or slice it counts elements; for a string later it counts UTF-8 bytes, not characters. Closest C++ equivalents are size and string::size.
- rem_euclid takes a floating receiver and divisor and returns a least nonnegative remainder. Unlike C++ fmod, a negative finite dividend with a positive divisor normally gives a positive result. Floating roundoff can exceptionally produce the divisor's absolute value, not strictly less. A zero divisor or invalid infinite arithmetic yields NaN. The function below expects a finite azimuth.
- floor takes a float and returns the greatest integer-valued float no larger than it: floor(-1.2) is -2.0. It preserves infinities and propagates NaN. This corresponds to std::floor.
- fract returns self minus its truncated integer part: fract(-1.2) is approximately -0.2, not 0.8. Non-finite input produces NaN. C++ modf's fractional result is the closest operation.

```rust
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
```

1. The impl attaches at to Horizon. &self is shared access, similar to a C++ const member function.
2. Empty data returns the sentinel -90 degrees before any division or indexing.
3. p wraps azimuth, scales a turn to n bins, and becomes a fractional index. The formula is p = rem_euclid(A, 360)n/360.
4. i is floor(p), converted to usize and reduced modulo n. The modulo also protects the rare rounded wrap-to-n case.
5. a reads the current bin; b reads the next bin, wrapping n back to zero.
6. The final expression is a + (b - a)fract(p). It linearly interpolates between adjacent bins and is the function's return value.
7. A single-bin profile interpolates a value with itself. Negative angles wrap. Non-finite angles are not rejected: the numeric result can become NaN. Callers should supply finite azimuths.

See [the interpolation formula](01-skeleton.md#52-horizonat).

<a id="s4-3"></a>
### 4.3 Dem and to_xy

New APIs from the [f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html):

- to_radians takes degrees and returns degrees multiplied by pi/180, with floating rounding. to_degrees does the inverse multiplication by 180/pi. C++ has no identical member methods; use the corresponding arithmetic.
- tan takes radians and returns tangent. sin and cos take radians and return sine and cosine. They correspond to std::tan, std::sin, and std::cos. Trigonometric precision is platform-dependent. NaN propagates; infinite arguments have no finite trigonometric result and yield NaN.
- std::f64::consts::FRAC_PI_4 is the f64 approximation of pi/4, closest to a C++ constexpr expression built from std::numbers::pi.

```rust
pub struct Dem
{
	pub w: usize,
	pub h: usize,
	pub res_m: f64,
	pub x_min_m: f64,
	pub y_max_m: f64,
	pub z_m: Vec<f32>,
}
```

Dem stores width w, height h, spacing res_m, leftmost sample x_min_m, topmost sample y_max_m, and row-major elevation samples z_m. Element (row, col) occupies row*w + col. Rows run toward decreasing y. The grid limits are sample centers, not the raster's outer pixel boundaries. Dem has no derived serialization or default implementation.

```rust
pub fn to_xy(lat_deg: f64, lon_deg: f64) -> (f64, f64)
{
	let rho =
		2.0 * MOON_RADIUS_M * (std::f64::consts::FRAC_PI_4 + lat_deg.to_radians() / 2.0).tan();
	let lon = lon_deg.to_radians();
	(rho * lon.sin(), rho * lon.cos())
}
```

1. Convert latitude to radians and form pi/4 + latitude/2.
2. Take its tangent, then multiply by 2R to get projected distance rho.
3. Convert longitude to radians once.
4. Return a tuple (rho sin(longitude), rho cos(longitude)).
5. At latitude -90 degrees, rho is zero. At longitude zero, projected y is positive away from the pole. Invalid or out-of-domain geographic coordinates are not validated here.

The formulas are rho = 2R tan(pi/4 + phi/2), x = rho sin(lambda), y = rho cos(lambda). The doc comment requires checking signs and orientation against a real DEM label. See [the projection derivation](01-skeleton.md#53-dem-and-to_xy).

<a id="s4-4"></a>
### 4.4 impl Dem and Dem::height_at

New APIs:

- is_finite takes a float and returns true only for ordinary finite values, including subnormals and both signed zeros. It rejects both infinities and NaN. Closest C++ equivalent: std::isfinite.
- checked_mul takes two usize values and returns Some(product), or None if the mathematical product overflows usize. Unlike ordinary C++ unsigned multiplication, overflow is reported rather than wrapped. [Integer reference](https://doc.rust-lang.org/std/primitive.usize.html#method.checked_mul).
- min takes two ordered integers here and returns their smaller value, like std::min. There is no NaN case because these indices are integers.
- then_some takes a boolean receiver and a value, returning Some(value) if true and None otherwise. The value is evaluated before this call, even if false. Closest C++ equivalent: an already-computed value placed in std::optional conditionally. [Boolean reference](https://doc.rust-lang.org/std/primitive.bool.html#method.then_some).

```rust
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
```

1. Reject zero width/height, non-finite or nonpositive spacing, and a storage-length mismatch. checked_mul with ? returns None immediately on dimension overflow.
2. Compute col = (x - x_min)/res and row = (y_max - y)/res. The minus in the row expression accounts for decreasing y.
3. Reject non-finite coordinates and coordinates outside [0, w-1] by [0, h-1]. Short-circuiting ensures the earlier zero-dimension return happened before subtracting one.
4. floor identifies the containing sample cell. Casts are safe for ordinary validated grid sizes.
5. Clamp c+1 and r+1 to the final valid indices. This permits interpolation on the last row or column without reading past storage.
6. fract gives the fractional offsets fx and fy, narrowed to f32 to match stored heights.
7. top interpolates along the first row; bottom interpolates along the next row.
8. z interpolates between top and bottom. The formula is (1-fy)((1-fx)z00+fx*z10) + fy((1-fx)z01+fx*z11).
9. Return Some(z) only if the result is finite. No-data samples encoded as finite sentinel numbers are not recognized. A NaN neighbor can invalidate a result even with a zero coefficient, since zero times NaN is NaN.

The result is an optional f32, not an exception. See [bilinear interpolation](01-skeleton.md#54-demheight_at). Extremely large dimensions and allocations are not intended inputs.

<a id="s4-5"></a>
### 4.5 horizon_from_dem

New APIs:

- vec! constructs an owned vector. The repetition form takes a value and count, cloning the value into that many elements, like vector(count, value). The list form used in tests takes individual elements. Zero count gives an empty vector. Impossible capacities or allocation failure are not ordinary Result errors. [Macro reference](https://doc.rust-lang.org/std/macro.vec.html).
- iter_mut takes an exclusive sequence borrow and returns an iterator over exclusive element references. It resembles iterating T& over a mutable C++ vector; the original vector cannot be independently accessed incompatibly during the borrow.
- enumerate takes an iterator and returns an iterator of (usize index, item), starting at zero. It is lazy and yields no items for empty input. C++23 views::enumerate is the closest operation.
- f64::from takes a supported source number, here f32, and widens it to f64 without loss of the original f32 value. This is a From-trait conversion, closest here to static_cast<double>(float).
- atan takes a floating ratio and returns arctangent in radians in [-pi/2, pi/2]. Infinities map to the limiting angles; NaN propagates. Closest C++ equivalent: std::atan.
- max on floats returns the larger operand, ignoring a single NaN; both NaN inputs yield NaN. It can choose either signed zero when they compare equal. It is not generally identical to C++ std::max for NaN. Use std::fmax as the closer floating operation.

```rust
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
```

1. Allocate n_az bins, initially -90 degrees. With zero bins the later loop is empty.
2. Project the site and query its terrain height. The let-else returns the untouched sentinel profile if the site cannot be sampled.
3. Set marching step to half a DEM cell. height_at already rejects nonpositive or non-finite spacing for a sampled site.
4. A non-finite distance limit returns the sentinel profile. A negative or shorter-than-step limit performs no samples.
5. enumerate assigns each mutable horizon bin its azimuth index i. Compute A = 360i/n, then angle = A + longitude, and convert to radians.
6. Start the distance s at one step, not zero, avoiding division by zero at the observer.
7. Sample at x+s*sin(angle), y+s*cos(angle). Both components are added to follow geographic azimuth.
8. Stop the current ray at the first missing terrain sample or grid edge.
9. Compute dz = z-z0-s*s/(2R). The subtraction is curvature relative to the observer's tangent plane.
10. Convert atan(dz/s) to degrees, narrow to f32, and keep the maximum in the current bin through the mutable reference.
11. Advance s and repeat while s is within max_dist_m. Return the populated Horizon.

Geographic north increases latitude, away from the south pole, so N = (sin(lambda), cos(lambda)); east is E = (cos(lambda), -sin(lambda)). Thus cos(A)N+sin(A)E = (sin(A+lambda), cos(A+lambda)), matching Horizons azimuth. At longitude zero, north is positive y and east is positive x; at longitude 90 degrees, north is positive x and east is negative y. The pole itself has a longitude-dependent limiting basis.

The archived hourly Horizons data place Earth's January 2027 azimuth between 129.583760 and 144.690080 degrees for the example site, near the reported 137.3-degree Earth-side bearing. The positive-y direction has A = 360 - 222.69 = 137.31 degrees under this formula. The previous mirrored formula incorrectly assigned it 42.69 degrees. The wall test in Section 5.7 checks the corrected bearing independently of the downloaded tracks.

For short distances, the spherical drop is R-sqrt(R*R-s*s), approximately s*s/(2R). s is treated as local horizontal distance. Projected distance, surface arc length, and chord projection are not identical, but share the leading approximation near the pole. The algorithm assumes common height datums and neglects refraction, observer height, projection-scale correction, and features between samples. See [the basis and curvature derivations](01-skeleton.md#55-horizon_from_dem-and-the-corrected-directions).

<a id="s4-6"></a>
### 4.6 sun_fraction

New API: clamp takes a float and lower/upper bounds, returning the nearer bound if outside, otherwise the original value. NaN input stays NaN; reversed or NaN bounds panic. Here the fixed bounds are valid. C++ std::clamp is close for finite values, but its invalid-bound contract differs. [Reference](https://doc.rust-lang.org/std/primitive.f32.html#method.clamp).

```rust
pub fn sun_fraction(el: f32, h: f32) -> f32
{
	((el - h + SUN_RADIUS_DEG) / (2.0 * SUN_RADIUS_DEG)).clamp(0.0, 1.0)
}
```

1. Subtract the terrain horizon h from center elevation el.
2. Add solar radius r = 0.267 degrees, making zero correspond to a fully hidden disc.
3. Divide by diameter 2r, making one correspond to a center one radius above the horizon.
4. Clamp to [0, 1].

The formula is clamp((el-h+r)/(2r), 0, 1). It is a linear visibility proxy, not the exact area of a circular segment. See [the endpoint derivation](01-skeleton.md#56-sun_fraction-and-visible).

<a id="s4-7"></a>
### 4.7 visible

New APIs, including the [iterator reference](https://doc.rust-lang.org/std/iter/trait.Iterator.html):

- assert_eq! takes two expressions, compares their values, and panics with diagnostics when unequal. Success returns unit. It does not disappear in release builds. Closest C++ equivalent is a permanently enabled equality assertion, not NDEBUG-controlled assert.
- iter takes a shared sequence borrow and yields shared references in element order. No copying or allocation is required for that traversal. Empty input yields no items. C++ begin/end or a range over const T& is equivalent in purpose.
- zip takes two iterable sequences and lazily pairs corresponding items. It stops at the shorter sequence; it does not report unequal lengths. Closest C++ equivalent: views::zip.
- map on an iterator takes a callable and lazily transforms each item into an output item, like views::transform. It does nothing until the iterator is consumed.
- collect consumes an iterator and constructs an inferred result collection, here Vec<bool>. Empty input gives an empty vector. Allocation failures are not recovered here. C++ ranges::to<vector> is the closest operation.

```rust
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
```

1. Assert that all three Track vectors have equal length. This compensates for zip otherwise truncating silently.
2. Borrow the azimuth vector as an iterator.
3. Zip it with a shared borrow of elevations.
4. The closure's reference patterns copy each pair of f32 values into az and el.
5. Interpolate the terrain horizon at az, add margin_deg, and test strict elevation greater-than.
6. Collect one boolean per timestamp. Equality is false. A NaN in the comparison also produces false; callers should use validated finite angles.

margin_deg is a required clearance, not automatically the solar radius. This function is suitable for binary decisions such as Earth visibility. It does not call sun_fraction.

<a id="s4-8"></a>
### 4.8 Stats and stats

New APIs:

- default is the Default-trait constructor. Stats::default returns four zero f64 fields; Track::default later returns three empty vectors. Closest C++ equivalent: a generated value-initializing constructor. It takes no arguments and does not mean scientific defaults such as a flat horizon.
- windows takes a positive window length and returns an iterator over overlapping borrowed slices. windows(2) yields adjacent pairs, with no items for fewer than two timestamps. A zero requested length panics. C++ views::slide is the closest operation. [Reference](https://doc.rust-lang.org/std/primitive.slice.html#method.windows).
- assert! takes a boolean and panics if it is false, in debug and release. Success returns unit. Like assert_eq!, it is stronger than the usual release-disabled C++ assert.

```rust
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats
{
	pub pct_sun: f64,
	pub longest_night_h: f64,
	pub pct_earth: f64,
	pub pct_both: f64,
}
```

The fields are duration percentages, not fractions: pct_sun, pct_earth, pct_both range from 0 to 100 for valid input. longest_night_h is in hours. Debug, Default, Serialize, and Deserialize are generated as described in Section 4.1.

```rust
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
```

1. Assert equal slice lengths.
2. Initialize result and the current-night and total-duration accumulators.
3. Visit adjacent timestamp pairs, keeping their starting index i.
4. Require strictly increasing times before adding duration.
5. Convert endpoints to f64 and subtract, then divide by 3,600 to obtain hours. This avoids signed i64 subtraction overflow, but extremely large timestamps could lose precision when cast.
6. If the left-end Sun sample is true, add dt to sunny hours and reset the ongoing night.
7. Otherwise add dt to the ongoing night and update its maximum.
8. Independently add dt to Earth hours when its left sample is true.
9. Add dt to simultaneous hours only when both samples are true.
10. If any duration exists, multiply the three hour counters by 100/total, turning them into percentages.
11. Return result; the longest-night value was already in hours.

The formulas are T = sum(dt), pct_sun = 100*sum(dt*S)/T, pct_earth = 100*sum(dt*E)/T, pct_both = 100*sum(dt*S*E)/T for boolean indicators S and E. The last sample contributes no interval. Gaps count in full under left-held visibility. Empty and singleton input return zeros. See [the statistics derivation](01-skeleton.md#57-stats-and-stats).

<a id="s4-9"></a>
### 4.9 parse_horizons

New APIs in first execution order:

- split_once takes a string and delimiter and returns Some((prefix, suffix)) around the first occurrence, excluding the delimiter, or None. Here the delimiter is a fixed string. C++ string_view::find plus two substrings is equivalent. [String reference](https://doc.rust-lang.org/std/primitive.str.html).
- ok_or consumes an Option and an already-evaluated error, returning Ok(value) or Err(error). It is a branch from std::optional to an error-bearing return, closest to an explicit std::expected construction. [Reference](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or).
- lines takes a string and yields borrowed lines. LF and CRLF are recognized and stripped; a final line terminator does not add another empty line. Empty text yields no lines. C++ getline is close, but returns owned/modified buffers rather than these borrowed views.
- filter takes an iterator and a predicate over references to its items; it lazily retains items whose predicate is true. Like views::filter, it does not itself allocate.
- trim takes a string and returns a borrowed view without leading/trailing Unicode whitespace. It leaves internal whitespace intact. C++ has no single std string-view trim method.
- split_whitespace takes a string and yields borrowed nonempty fields separated by Unicode whitespace. Leading/trailing or repeated whitespace produces no empty fields. Empty input yields no fields. Closest C++ equivalent: token extraction, but with Unicode whitespace and views.
- format! takes a format string and arguments and returns an owned String. {} requests normal display, named placeholders insert local values, and :? requests Debug. It resembles std::format; \n inserts a newline. Invalid literal formatting is generally diagnosed at compilation.
- NaiveDateTime::parse_from_str takes text and a format string and returns Result<NaiveDateTime, ParseError>. %Y, %b, %d, %H, %M, and %S mean year, abbreviated month, day, 24-hour hour, minute, and second. Omitted seconds default to zero here. Invalid dates/times or mismatched fields fail. The result has no timezone yet. Closest C++ equivalent: chrono::parse into calendar/time fields. [Chrono reference](https://docs.rs/chrono/0.4.45/chrono/naive/struct.NaiveDateTime.html#method.parse_from_str).
- or_else consumes a Result and takes a callable of its error, returning the original success or the callable's fallback Result. The callable runs only on failure. Closest C++ equivalent: expected::or_else.
- map_err consumes a Result and transforms only its error, leaving a success untouched, comparable to expected::transform_error.
- and_utc takes a naive date/time and returns a UTC DateTime with the same calendar and clock fields. It assigns UTC, not local-time conversion. [Reference](https://docs.rs/chrono/0.4.45/chrono/naive/struct.NaiveDateTime.html#method.and_utc).
- timestamp takes that UTC DateTime and returns i64 non-leap seconds since the Unix epoch, negative before it. Closest C++ equivalent: duration_cast<seconds> of a system-clock epoch offset. Chrono has special leap-second representation rules, but this output does not count leap seconds. [Reference](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.timestamp).
- rev consumes a double-ended iterator and reverses its direction. A slice iterator supports it; empty remains empty. Closest C++ equivalent: views::reverse.
- filter_map takes an iterator and a callable returning Option. Some values are yielded; None values are skipped. It is a fused transform-and-filter, not a built-in C++20 ranges adaptor.
- parse takes a string and chooses its output type through the FromStr trait, returning a Result. Here parse::<f32> accepts floating syntax, including special non-finite values; invalid tokens fail. Unlike std::stof, it reports an error value and requires a valid complete numeric string.
- ok consumes a Result and returns Some(success), discarding any error as None. Here discarded errors are deliberate nonnumeric markers, not rejected rows.
- next takes a mutable iterator and returns its next item as Some, or None on exhaustion, similar to advancing/checking a C++ iterator.
- last on the timestamp vector resolves to the slice method, not Iterator::last: it takes a shared slice and returns an optional reference to the final element without consuming the vector. It is safe for empty input, unlike C++ vector::back.
- is_some_and consumes an Option and tests its contained value with a predicate, returning false for None. It resembles opt.has_value() && predicate(*opt).
- into consumes a value and converts it through the Into trait to an inferred type. Here it turns string literals into owned Strings. It is a supported type conversion, not a numeric reinterpretation.
- push takes an exclusive Vec borrow and an owned value and appends it, growing storage if needed. It resembles vector::push_back. It can fail through capacity overflow or allocation failure, not a returned Result.

```rust
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
```

1. Find the first $$SOE start marker. Convert missing-marker None to an error, then use ? to return early. The &str error converts to the function's String error.
2. Search only the remaining suffix for $$EOE. Restrict parsing to the intervening body.
3. Initialize an empty Track.
4. Iterate body lines, dropping whitespace-only ones.
5. Collect whitespace-separated fields into a vector of borrowed string views. Require at least four tokens so date, time, and two angles are possible.
6. Format date and time as one string and parse minute precision first. If that fails, retry with seconds. Convert the parse error into a contextual String.
7. Assign UTC and obtain Unix seconds.
8. Reverse the tokens after date/time. Attempt f32 parsing and skip failures, including combined markers such as *i. This intentionally ignores every nonnumeric token, not a fixed whitelist.
9. Read elevation first and azimuth second, since the iterator runs backward. The selected values are the last two numeric tokens in forward order.
10. If either is missing, return an error. If either is NaN or infinite, return an error rather than silently skipping it.
11. Require the new timestamp to exceed the previous one when a previous sample exists.
12. Append timestamp, azimuth, and elevation in matching order.
13. Reject an empty ephemeris; otherwise return Ok(track).

Earlier numeric tokens are ignored. Extra numeric fields after the desired angles would change which pair is selected, so this parser remains tied to Horizons quantity 4 text output. Empty marker fields and separate or combined marker tokens all work. The [earlier skeleton parser description](01-skeleton.md#58-parse_horizons) predates this marker fix; the source and this section describe current behavior.

<a id="s4-10"></a>
### 4.10 Test-module declaration

```rust
#[cfg(test)]
mod tests;
```

cfg(test) includes this module only for test compilation. mod tests loads core/src/tests.rs. This resembles a separate C++ test target selected at build time, not a runtime condition. Normal library consumers do not include the test functions.

<a id="s5"></a>
## 5. core/src/tests.rs

```rust
use super::*;
```

super names the parent module; the glob imports its accessible items. A test's #[test] attribute registers it with Rust's test harness. A normal test succeeds when it returns without panic. #[should_panic] reverses that expectation for the annotated test. Test functions return unit and receive no arguments.

<a id="s5-1"></a>
### 5.1 flat

```rust
fn flat() -> Dem
{
	Dem {
		w: 21,
		h: 21,
		res_m: 100.0,
		x_min_m: -1000.0,
		y_max_m: 1000.0,
		z_m: vec![0.0; 441],
	}
}
```

1. Construct and return a Dem literal.
2. Width and height are 21, spacing is 100 m.
3. Left x is -1,000 m and top y is 1,000 m. The origin falls at row 10, column 10.
4. Repeat zero 441 times, matching 21*21. This is a shared fixture, not a test by itself.

<a id="s5-2"></a>
### 5.2 pole_and_bilinear_grid

```rust
#[test]
fn pole_and_bilinear_grid()
{
	assert_eq!(to_xy(-90.0, 123.0), (0.0, 0.0));
	let dem = Dem {
		w: 2,
		h: 2,
		res_m: 1.0,
		x_min_m: 0.0,
		y_max_m: 1.0,
		z_m: vec![0.0, 2.0, 4.0, 6.0],
	};
	assert_eq!(dem.height_at(0.5, 0.5), Some(3.0));
	assert_eq!(dem.height_at(1.0, 0.0), Some(6.0));
	assert_eq!(dem.height_at(-0.1, 0.0), None);
	assert_eq!(dem.height_at(f64::NAN, 0.0), None);
}
```

1. Assert that latitude -90 degrees projects to the origin for longitude 123 degrees.
2. Construct a two-by-two grid with heights 0, 2, 4, and 6 and unit spacing.
3. At its center, each bilinear weight is one quarter, so height is 3.
4. At its lower-right sample, height is exactly 6.
5. x = -0.1 lies outside and returns None.
6. f64::NAN is the standard NaN constant; the final assertion verifies coordinate validation rejects it. Closest C++ equivalent: numeric_limits<double>::quiet_NaN.

<a id="s5-3"></a>
### 5.3 flat_horizon_includes_curvature

New API: abs takes a float and returns its absolute value, including turning negative zero positive and infinity positive. NaN stays NaN. Closest C++ equivalent: std::abs. [Reference](https://doc.rust-lang.org/std/primitive.f32.html#method.abs).

```rust
#[test]
fn flat_horizon_includes_curvature()
{
	let dem = flat();
	let horizon = horizon_from_dem(&dem, -90.0, 0.0, 8, 800.0);
	let expected = (-50.0 / (2.0 * MOON_RADIUS_M)).atan().to_degrees() as f32;
	for h in horizon.h_deg
	{
		assert!(h < 0.0 && (h - expected).abs() < 1e-6);
	}
	assert_eq!(
		horizon_from_dem(&dem, -80.0, 0.0, 1, 800.0).h_deg,
		vec![-90.0]
	);
	assert!(
		horizon_from_dem(&dem, -90.0, 0.0, 0, 800.0)
			.h_deg
			.is_empty()
	);
}
```

1. Build the flat fixture and march eight rays to 800 m.
2. With dz = 0, angle at distance s is atan(-s/(2R)). The maximum occurs at the first 50 m step.
3. Compute that expected angle, convert to degrees, and narrow to f32.
4. Consume the returned height vector, checking every bin is negative and within 0.000001 degree of the formula.
5. A latitude -80 degree site lies outside this small grid; its sole bin remains -90.
6. A zero-bin request yields an empty profile.

The scientific check includes curvature rather than incorrectly expecting an exactly zero flat horizon.

<a id="s5-4"></a>
### 5.4 wall_angle_and_local_azimuth

New API: powi takes a floating receiver and a signed integer exponent and returns that integer power. Here exponent 2 squares 500. Its rounding need not match other power implementations; negative exponents are allowed, and NaN to exponent zero can return one. C++ std::pow with an integer exponent is the closest standard call, though identical rounding is not promised. [Reference](https://doc.rust-lang.org/std/primitive.f64.html#method.powi).

```rust
#[test]
fn wall_angle_and_local_azimuth()
{
	let mut dem = flat();
	for c in 0..dem.w
	{
		dem.z_m[15 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -90.0, 90.0, 4, 800.0);
	let expected = ((100.0 - 500.0_f64.powi(2) / (2.0 * MOON_RADIUS_M)) / 500.0)
		.atan()
		.to_degrees() as f32;
	assert!((horizon.h_deg[1] - expected).abs() < 1e-4);
	assert!(horizon.h_deg[3] < 0.0);
}
```

1. Start with flat terrain.
2. Set every column of row 15 to 100 m; this row is at y = -500 m.
3. At the pole's longitude-90 limiting basis, azimuth 90 points toward negative y.
4. Compute atan((100-500^2/(2R))/500), including curvature.
5. Require the east bin to match within 0.0001 degree.
6. Require the opposite bin to remain negative.

This checks magnitude and orientation within the selected pole basis. The next two tests avoid the pole singularity.

<a id="s5-5"></a>
### 5.5 north_ridge_away_from_pole_singularity

```rust
#[test]
fn north_ridge_away_from_pole_singularity()
{
	let (_, y) = to_xy(-89.0, 0.0);
	let mut dem = Dem {
		w: 21,
		h: 41,
		res_m: 100.0,
		x_min_m: -1000.0,
		y_max_m: y + 1000.0,
		z_m: vec![0.0; 21 * 41],
	};
	for c in 0..dem.w
	{
		dem.z_m[5 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -89.0, 0.0, 4, 800.0);
	assert!(horizon.h_deg[0] > 10.0);
	assert!(horizon.h_deg[2] < 0.0);
}
```

1. Project latitude -89 degrees, longitude zero, keeping its positive y.
2. Construct a 21-by-41 DEM around that site with 100 m spacing. The site's y lies 1,000 m below the top, at row 10.
3. Put a 100 m ridge at row 5, 500 m farther toward positive y, geographically north and away from the south pole.
4. March four cardinal directions to 800 m.
5. Require the north bin to exceed 10 degrees and the south bin to stay negative.

The site is inside the DEM despite its large distance from the origin. This catches the old positive-cosine ray sign.

<a id="s5-6"></a>
### 5.6 east_at_lon_90_points_toward_negative_y

```rust
#[test]
fn east_at_lon_90_points_toward_negative_y()
{
	let (x, y) = to_xy(-89.0, 90.0);
	let mut dem = Dem {
		w: 21,
		h: 41,
		res_m: 100.0,
		x_min_m: x - 1000.0,
		y_max_m: y + 1000.0,
		z_m: vec![0.0; 21 * 41],
	};
	for c in 0..dem.w
	{
		dem.z_m[15 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, -89.0, 90.0, 4, 800.0);
	assert!(horizon.h_deg[1] > 10.0);
	assert!(horizon.h_deg[3] < 0.0);
}
```

1. Project latitude -89 degrees at longitude 90, retaining both coordinates.
2. Center the horizontal extent around x and set the top relative to y so the site is inside.
3. Again place the ridge 500 m toward smaller y.
4. Under the required basis at this longitude, that direction is east.
5. Assert a positive east obstruction above 10 degrees and a negative west horizon.

Together with Section 5.5, this distinguishes the local basis from fixed grid-axis assumptions.

<a id="s5-7"></a>
### 5.7 earth_side_wall_at_example_longitude

```rust
#[test]
fn earth_side_wall_at_example_longitude()
{
	let lat = -89.45;
	let lon = 222.69;
	let (x, y) = to_xy(lat, lon);
	let mut dem = Dem {
		w: 21,
		h: 21,
		res_m: 100.0,
		x_min_m: x - 1000.0,
		y_max_m: y + 1000.0,
		z_m: vec![0.0; 441],
	};
	for c in 0..dem.w
	{
		dem.z_m[5 * dem.w + c] = 100.0;
	}
	let horizon = horizon_from_dem(&dem, lat, lon, 720, 800.0);
	let mut peak = 0;
	for i in 1..horizon.h_deg.len()
	{
		if horizon.h_deg[i] > horizon.h_deg[peak]
		{
			peak = i;
		}
	}
	let peak_az = peak as f64 * 360.0 / horizon.h_deg.len() as f64;
	assert!((peak_az - (360.0 - lon)).abs() < 0.5);
	assert!(horizon.at((360.0 - lon) as f32) > 10.0);
	assert!(horizon.at(42.69) < 0.0);
	assert!(horizon.at(317.31) < 0.0);
}
```

1. Project the actual example site at latitude -89.45 degrees and longitude 222.69 degrees.
2. Center a 21 by 21 DEM on that site with 100 m spacing. Put a 100 m wall at row 5, 500 m in positive y.
3. Compute 720 horizon bins, one every 0.5 degrees, out to 800 m.
4. Scan all bin values with an index loop, retaining the index of the highest angle. This uses the existing len, indexing, comparison, and assignment operations without a new iterator API.
5. Convert that index to azimuth in degrees and require it to be within 0.5 degrees of 360 - 222.69 = 137.31 degrees.
6. Require an interpolated horizon above 10 degrees there. Require negative elevations at 42.69 degrees, the previous mirrored bearing, and at 317.31 degrees, the opposite direction.

This is an independent synthetic regression for the Earth-side orientation, not a claim that Earth's azimuth is constant throughout the month. The former formula would put the peak near 42.69 degrees and fail the test.

<a id="s5-8"></a>
### 5.8 interpolation_wraps

```rust
#[test]
fn interpolation_wraps()
{
	let h = Horizon {
		h_deg: vec![0.0, 10.0, 20.0, 30.0],
	};
	assert_eq!(h.at(360.0), 0.0);
	assert_eq!(h.at(-45.0), 15.0);
	assert_eq!(h.at(405.0), 5.0);
	assert_eq!(Horizon { h_deg: vec![] }.at(0.0), -90.0);
}
```

1. Define four bins at 0, 90, 180, and 270 degrees with elevations 0, 10, 20, and 30.
2. A full turn returns the north-bin value.
3. -45 degrees lies halfway from the last bin to north, giving 15.
4. 405 degrees wraps to 45 and returns 5.
5. An empty profile returns the -90 sentinel.

This tests both seam interpolation and angles outside a single turn.

<a id="s5-9"></a>
### 5.9 visibility_and_solar_disc

```rust
#[test]
fn visibility_and_solar_disc()
{
	assert_eq!(sun_fraction(0.0, 0.0), 0.5);
	assert_eq!(sun_fraction(-1.0, 0.0), 0.0);
	assert_eq!(sun_fraction(1.0, 0.0), 1.0);
	let track = Track {
		t: vec![0, 1],
		az_deg: vec![0.0, 360.0],
		el_deg: vec![0.0, 1.0],
	};
	assert_eq!(
		visible(&track, &Horizon { h_deg: vec![0.0] }, 0.5),
		vec![false, true]
	);
	assert!(visible(&Track::default(), &Horizon { h_deg: vec![] }, 0.0).is_empty());
}
```

1. Check fraction 0.5 for a solar center on the horizon.
2. Check lower and upper clamp saturation at elevations -1 and +1 degrees.
3. Construct two Track samples at equivalent azimuths 0 and 360 but elevations 0 and 1.
4. With a flat horizon and a 0.5-degree margin, the decisions are false then true.
5. Empty input returns an empty decision vector.

The same test checks a continuous proxy and a separate binary rule without conflating them.

<a id="s5-10"></a>
### 5.10 weighted_stats_include_gap

```rust
#[test]
fn weighted_stats_include_gap()
{
	let s = stats(
		&[0, 3600, 7200, 21600, 25200],
		&[true, false, false, true, true],
		&[true, true, false, false, true],
	);
	assert!((s.pct_sun - 200.0 / 7.0).abs() < 1e-9);
	assert!((s.pct_earth - 200.0 / 7.0).abs() < 1e-9);
	assert!((s.pct_both - 100.0 / 7.0).abs() < 1e-9);
	assert_eq!(s.longest_night_h, 5.0);
	assert_eq!(stats(&[], &[], &[]).pct_sun, 0.0);
	assert_eq!(stats(&[0], &[false], &[false]).longest_night_h, 0.0);
}
```

1. Supply timestamps defining durations 1, 1, 4, and 1 hours.
2. Left-held Sun values give sunny intervals 0 and 3, totaling two hours.
3. Left-held Earth values give intervals 0 and 1, also two hours.
4. Only interval 0 has both, totaling one hour.
5. Compare percentages with 200/7, 200/7, and 100/7 using tolerance 0.000000001.
6. Consecutive dark intervals 1 and 2 form a five-hour night, including the four-hour gap.
7. Empty input has zero Sun percentage; a single dark sample has no represented night duration.

The last sample's booleans do not add a fifth interval.

<a id="s5-11"></a>
### 5.11 stats_rejects_unordered_times

```rust
#[test]
#[should_panic]
fn stats_rejects_unordered_times()
{
	stats(&[1, 0], &[true; 2], &[true; 2]);
}
```

The test and should_panic attributes both apply to this function. It supplies decreasing timestamps with correctly sized boolean arrays. stats must panic at its monotonicity assertion. A two-element array written with a repeated value supplies two true entries.

<a id="s5-12"></a>
### 5.12 horizons_sample_and_errors

New APIs from [Result](https://doc.rust-lang.org/std/result/enum.Result.html):

- unwrap consumes a Result and extracts its success, panicking on an error. It is appropriate here because failure should fail the test. C++ expected::value is similar but throws instead of Rust panicking.
- is_err borrows a Result and returns true only for Err. It does not inspect the error message or consume the result. C++ equivalent: !expected.has_value().

```rust
#[test]
fn horizons_sample_and_errors()
{
	let track = parse_horizons("header\n$$SOE\n 2026-Dec-01 00:00 *    57.386895   1.766619\n 2026-Dec-01 06:00 C m 58.0 2.0\n$$EOE\nfooter").unwrap();
	assert_eq!(track.t, vec![1796083200, 1796104800]);
	assert_eq!(track.az_deg[0], 57.386895);
	assert_eq!(track.el_deg[0], 1.766619);
	for text in [
		"",
		"$$SOE",
		"$$SOE\n$$EOE",
		"$$SOE\nbad row\n$$EOE",
		"$$SOE\n2026-Dec-01 00:00 n.a. n.a.\n$$EOE",
	]
	{
		assert!(parse_horizons(text).is_err());
	}
}
```

1. Parse a header, two rows, and a footer. Only the rows between the markers count.
2. Assert the two exact Unix timestamps.
3. Assert the first f32 azimuth and elevation equal the sample literals rounded to f32.
4. Iterate malformed examples: no markers, no end marker, an empty body, a malformed row, and unavailable angles.
5. Require each parse attempt to return Err.

The second valid row also verifies separated flag tokens C and m.

<a id="s5-13"></a>
### 5.13 horizons_combined_optional_markers_and_last_numeric_pair

```rust
#[test]
fn horizons_combined_optional_markers_and_last_numeric_pair()
{
	for marker in ["*i", " i", "*", "", "CNAnim"]
	{
		let text = format!("$$SOE\n 2027-Jan-01 05:00 {marker} 123.45 1.23\n$$EOE");
		let track = parse_horizons(&text).unwrap();
		assert_eq!(track.az_deg, vec![123.45]);
		assert_eq!(track.el_deg, vec![1.23]);
	}
	let track = parse_horizons("$$SOE\n2027-Jan-01 05:00 999 *i 123.45 flag -1.23 trailing\n$$EOE")
		.unwrap();
	assert_eq!(track.az_deg, vec![123.45]);
	assert_eq!(track.el_deg, vec![-1.23]);
	for values in ["*i", "i 123.45", "*i NaN 1.23", "* 123.45 inf"]
	{
		let text = format!("$$SOE\n2027-Jan-01 05:00 {values}\n$$EOE");
		assert!(parse_horizons(&text).is_err());
	}
}
```

1. Try the same valid angle pair with *i, a separate i, *, no marker, and a longer combined marker.
2. Build one response per case with format!, unwrap its parse, and assert both angle vectors.
3. Add an earlier numeric token 999, an intervening word, and a trailing word. Verify the parser selects the final two numbers, not the first two tokens.
4. Keep a negative elevation to verify its minus sign remains numeric input.
5. Require errors for zero numbers, one number, a selected NaN, and a selected infinity.

There is no whitelist of valid marker combinations. That is the regression this test protects.

<a id="s6"></a>
## 6. prep/Cargo.toml

```toml
[package]
name = "prep"
version = "0.1.0"
edition = "2024"

[dependencies]
lunar_core = { path = "../core" }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
```

1. The package is prep, version 0.1.0, edition 2024. src/main.rs gives it an executable entry point.
2. lunar_core is a local path dependency, resolved relative to this manifest.
3. reqwest 0.12 provides HTTP. HTTP is a request/response protocol; HTTPS adds encryption and server authentication through TLS.
4. default-features = false disables reqwest's default native TLS backend. blocking enables synchronous calls; rustls-tls selects Rustls. prep therefore does not need OpenSSL or pkg-config.
5. serde with derive generates output serialization.
6. serde_json encodes and decodes JSON.
7. chrono validates dates before requests.

Version strings request compatible releases, not exact equality. Cargo.lock records the selected versions.

<a id="s7"></a>
## 7. prep/src/main.rs

<a id="s7-1"></a>
### 7.1 Imports and SiteData

```rust
use chrono::NaiveDate;
use lunar_core::{Horizon, Site, Track, parse_horizons};
use serde::Serialize;
use std::{error::Error, fs, path::Path, time::Duration};

#[derive(Serialize)]
struct SiteData
{
	site: Site,
	sun: Track,
	earth: Track,
	horizon: Option<Horizon>,
}
```

1. Import the timezone-free calendar-date type NaiveDate.
2. Import the shared scientific types and parser from lunar_core.
3. Import Serialize for derive.
4. Import the standard error interface, filesystem module, path view, and duration type.
5. SiteData is private to prep. It owns one Site, two Tracks, and an optional Horizon.
6. The Serialize implementation exposes these field names as JSON object keys. No Deserialize implementation is needed because prep only writes this wrapper.
7. None serializes to null. The CLI currently never populates a DEM horizon.

<a id="s7-2"></a>
### 7.2 download

New APIs checked against [RequestBuilder](https://docs.rs/reqwest/0.12.28/reqwest/blocking/struct.RequestBuilder.html) and [Response](https://docs.rs/reqwest/0.12.28/reqwest/blocking/struct.Response.html):

- Client::get takes a shared client and a URL-like input and returns a request builder, not a response. The URL identifies the network resource. URL conversion failures are retained for later send. A C++ HTTP library's GET-builder operation is the closest equivalent; there is no standard C++ HTTP client.
- query takes a request builder and a serializable sequence of key/value pairs, returning the builder with URL-encoded query parameters appended. It escapes spaces and punctuation. Duplicate keys append rather than replace. Serialization failures are retained for send. It does not send a JSON request body.
- send consumes the request builder and blocks until a response or error is available. It returns a Result<Response, reqwest::Error>. Connection, TLS, timeout, and redirect failures are errors. HTTP status errors need a separate check.
- error_for_status consumes a response and returns it unchanged on a non-error status, or Err for a client/server error status in the 400 or 500 ranges. It does not inspect a successful response body for Horizons error text.
- text consumes the response, reads its body, and returns a Result<String, reqwest::Error>. In this manifest the optional charset feature is disabled, so decoding attempts UTF-8; malformed sequences are replaced rather than returned as UTF-8 validation errors. Body-read failures still return Err.

```rust
fn download(
	client: &reqwest::blocking::Client,
	site: &Site,
	body: &str,
	start: &str,
	stop: &str,
	step: &str,
) -> Result<Track, Box<dyn Error>>
{
	let coords = format!("'{},{},0'", site.lon_deg, site.lat_deg);
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
			("START_TIME", &format!("'{start}'")),
			("STOP_TIME", &format!("'{stop}'")),
			("STEP_SIZE", &format!("'{step}'")),
		])
		.send()?
		.error_for_status()?
		.text()?;
	parse_horizons(&text)
		.map_err(|e| format!("Horizons for {} ({body}): {e}\n{text}", site.name).into())
}
```

1. Borrow a reusable client and Site, plus string views for target body, dates, and step. Reusing a client permits connection reuse.
2. Format a quoted longitude, latitude, and zero-height triple for Horizons. Apostrophes inside the string are transmitted data.
3. Begin a GET request to https://ssd.jpl.nasa.gov/api/horizons.api.
4. Append the following query parameters:

| Name | Meaning |
| :--- | :--- |
| format | text, because the shared parser reads text rather than a JSON envelope. |
| COMMAND | '10' for Sun or '399' for Earth, supplied by the caller. |
| CENTER | 'coord@301', a coordinate-defined site on the Moon, body 301. |
| COORD_TYPE | 'GEODETIC', interpreting the triple as longitude, latitude, height. |
| SITE_COORD | Longitude/latitude in degrees, then height in kilometers; zero is not a DEM-derived height. |
| MAKE_EPHEM | 'YES', requesting a table. |
| EPHEM_TYPE | 'OBSERVER', quantities relative to the site. |
| QUANTITIES | '4', azimuth and elevation. |
| ANG_FORMAT | 'DEG', decimal degrees. |
| START_TIME | Quoted beginning calendar date. |
| STOP_TIME | Quoted ending date; a matching endpoint can be included. |
| STEP_SIZE | Quoted interval such as '6h'. |

5. Send, reject HTTP error statuses, and read text, propagating failures through ?.
6. Parse the body using the shared parser. A successful HTTP response containing a Horizons error will still fail parsing.
7. Transform a parser error into a message containing site, body identifier, and full response. into converts the String into the function's boxed error type through a supported standard conversion.
8. Return the helper's Result directly.

Temporary formatted parameter strings remain alive through the query call. query serializes them during builder construction; it does not retain these stack references for the later request.

<a id="s7-3"></a>
### 7.3 main

New APIs, in statement order:

- std::env::args takes no input and returns an iterator over owned process argument Strings, including the executable name first. Iteration panics for non-Unicode arguments. C++ argc/argv is the nearest source, but C++ does not impose Unicode validity. [Reference](https://doc.rust-lang.org/std/env/fn.args.html).
- skip consumes an iterator and lazily ignores the first n items. Fewer than n items leaves empty output. Closest C++ equivalent: views::drop.
- contains on an inclusive range borrows the range and a comparable value, returning whether both bounds include it. NaN fails floating-range containment. It is the expression lower <= value && value <= upper, not a linear search.
- NaiveDate::parse_from_str takes a text and date format and returns a parsed date or ParseError. Here %Y-%m-%d means year, numeric month, day. Invalid calendar dates fail. It does not attach a time or timezone. Closest C++ equivalent: chrono::parse into year_month_day. [Reference](https://docs.rs/chrono/0.4.45/chrono/struct.NaiveDate.html#method.parse_from_str).
- ends_with takes a string and a supported suffix pattern. Here the character-array pattern tests whether the last character is any of m, h, or d. An empty input matches none of these. Closest C++ equivalent: checking a nonempty string's back against three allowed characters.
- unwrap_or consumes an Option or Result and a fallback and returns the contained success or the fallback. The fallback is evaluated eagerly. Here a failed u32 parse becomes zero; later an absent optional argument becomes the default output path. Closest C++ optional equivalent: value_or.
- fs::read_to_string takes a path-like input and returns Result<String, io::Error>, reading the entire file and requiring valid UTF-8. Missing files, permissions, read failures, or invalid UTF-8 fail. C++ needs a file stream plus explicit UTF-8 validation. [Reference](https://doc.rust-lang.org/std/fs/fn.read_to_string.html).
- serde_json::from_str takes borrowed JSON text and returns a Result of the inferred deserialized type. Here that is Vec<Site>. Syntax/type errors fail. Unknown Site fields are ignored by the ordinary derive unless configured otherwise; required missing fields fail. C++ needs a JSON library with a matching conversion. [Reference](https://docs.rs/serde_json/1.0.154/serde_json/fn.from_str.html).
- get on args is the slice method: it borrows a sequence and index and returns an optional element reference, None out of bounds. C++ vector::at instead throws.
- map on Option takes a callable and transforms Some, leaving None unchanged. It is not the iterator map, although the purpose is similar. Closest C++23 equivalent: optional::transform.
- String::as_str takes a shared String and returns a shared str view without copying. Closest C++ equivalent: constructing string_view from string.
- Path::new takes borrowed path-like text and returns a borrowed Path view, without accessing disk or allocating a new owned path. The default path here is relative to the process's current working directory. Closest C++ equivalent: a view-like use of filesystem::path, though the C++ type itself owns storage. [Reference](https://doc.rust-lang.org/std/path/struct.Path.html#method.new).
- reqwest::blocking::Client::builder takes no arguments and returns a configurable ClientBuilder. timeout consumes that builder and a Duration-compatible value, returning the updated builder. This timeout applies to connection and response operations including body reading, not the entire month/site batch. Duration::from_secs takes a u64 count and returns that nonnegative duration. C++ equivalent: chrono::seconds and a client-specific timeout setter.
- build consumes the builder and returns a Result<Client, reqwest::Error>. TLS/resolver initialization can fail. The blocking builder can panic if used inside an asynchronous runtime; prep's main is synchronous. [Reference](https://docs.rs/reqwest/0.12.28/reqwest/blocking/struct.ClientBuilder.html).
- std::collections::HashSet::new returns an empty hash set with the default randomized hash state. insert takes an exclusive set borrow and owned value, returning true if newly inserted, false if equivalent to an existing value. Closest C++ equivalent: unordered_set::insert(value).second.
- fs::create_dir_all takes a path and returns an I/O Result, creating missing parent directories as well. Existing directories succeed; a conflicting file or permission failure can error. It is not an atomic transaction and does not roll back earlier directories. Closest C++ equivalent: filesystem::create_directories with error handling.
- Path::join takes a path and another path-like value and returns an owned PathBuf. An absolute appended path replaces the base. Here the generated filename is relative. It corresponds to filesystem::path concatenation, not string concatenation.
- serde_json::to_string_pretty borrows a serializable value and returns a Result<String, serde_json::Error> containing indented JSON. Custom serialization failures or unsupported map keys can error. This wrapper uses struct keys; Option::None becomes null. Non-finite floats are serialized as null by this serializer, not valid numeric literals. Callers should not use serialization as coordinate validation. [Reference](https://docs.rs/serde_json/1.0.154/serde_json/fn.to_string_pretty.html).
- fs::write takes path and byte-like content and returns an I/O Result. It creates or truncates the file and writes all content; it is not an atomic replace and does not create missing parent directories. Closest C++ equivalent: opening an output stream with truncation and checking all writes. [Reference](https://doc.rust-lang.org/std/fs/fn.write.html).
- display borrows a Path and returns a formatting adapter, with potentially lossy rendering for non-Unicode paths. It does not print by itself. C++ path::string is similar in purpose but returns text directly.
- println! takes a format string and its values, formats them to standard output, adds a newline, and returns unit. Its output can panic on a write failure. Closest C++ equivalent: println or format plus a newline.

The local helper slug takes a borrowed site name and returns an owned lowercase ASCII filename stem with hyphen-separated segments. It returns an empty string if there are no ASCII-alphanumeric segments. Main uses it below; Section 7.4 quotes its implementation and explains its individual methods. It has the same input/output role as a C++ function returning std::string from std::string_view.

```rust
fn main() -> Result<(), Box<dyn Error>>
{
	let args: Vec<_> = std::env::args().skip(1).collect();
	if !(4..=5).contains(&args.len())
	{
		return Err("Usage: prep <sites.json> <start YYYY-MM-DD> <stop YYYY-MM-DD> <step, e.g. 6h> [output-dir]".into());
	}
	let start = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d")?;
	let stop = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
	if stop <= start
	{
		return Err("Stop date must be after start date".into());
	}
	let step = &args[3];
	if !step.ends_with(['m', 'h', 'd']) || step[..step.len() - 1].parse::<u32>().unwrap_or(0) == 0
	{
		return Err("Step must be a positive integer followed by m, h, or d".into());
	}
	let sites: Vec<Site> = serde_json::from_str(&fs::read_to_string(&args[0])?)?;
	let output = Path::new(args.get(4).map(String::as_str).unwrap_or("app/assets/data"));
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(120))
		.build()?;
	let mut slugs = std::collections::HashSet::new();
	for site in &sites
	{
		if !site.lat_deg.is_finite()
			|| !(-90.0..=90.0).contains(&site.lat_deg)
			|| !site.lon_deg.is_finite()
		{
			return Err(format!("Invalid coordinates for {}", site.name).into());
		}
		let slug = slug(&site.name);
		if slug.is_empty() || !slugs.insert(slug)
		{
			return Err("Site names need unique, nonempty ASCII slugs".into());
		}
	}
	fs::create_dir_all(output)?;
	for site in sites
	{
		let sun = download(&client, &site, "'10'", &args[1], &args[2], step)?;
		let earth = download(&client, &site, "'399'", &args[1], &args[2], step)?;
		if sun.t != earth.t
		{
			return Err("Sun and Earth timestamps differ".into());
		}
		let path = output.join(format!("{}.json", slug(&site.name)));
		// TODO: Load a LOLA DEM and call horizon_from_dem here when DEM input is supported.
		let data = SiteData {
			site,
			sun,
			earth,
			horizon: None,
		};
		fs::write(&path, serde_json::to_string_pretty(&data)?)?;
		println!(
			"Wrote {} ({} samples per track)",
			path.display(),
			data.sun.t.len()
		);
	}
	Ok(())
}
```

1. Collect arguments after skipping the executable name.
2. Reject any count other than four or five, returning the usage error before indexing.
3. Parse beginning and ending dates. Reject a non-increasing date range.
4. Borrow the step. Short-circuiting first requires an allowed final ASCII character. Only then does the code remove one byte and parse the numeric prefix as u32.
5. Zero, negative, overflowing, or nonnumeric prefixes fail the positive-count test. The ASCII suffix guarantees the removed-byte boundary is valid, even if the earlier prefix contains Unicode and then fails parsing.
6. Read and deserialize sites.json into owned Sites, propagating file and JSON errors separately through nested ? operators.
7. Take argument 4 as optional output directory, otherwise app/assets/data. Map its borrowed String to a str and then Path.
8. Build a reusable blocking client with a 120-second timeout.
9. Create an empty set for filename slugs. This code calls slug, defined in Section 7.4: it converts the site name to lowercase ASCII-alphanumeric segments joined by hyphens.
10. Iterate sites by shared borrow for validation. Require finite longitude, finite latitude, and latitude within [-90, 90]. Longitude is not normalized.
11. Compute each slug, rejecting empty results and collisions before any output download. Inserting moves the slug into the set.
12. Create the output directory. An empty sites list creates the directory and succeeds without downloads.
13. Consume the sites vector, moving one owned Site at a time.
14. Download Sun and Earth sequentially using the same dates and interval.
15. Require equal timestamp vectors. This catches mismatched grids before output.
16. Build the JSON path from the site's slug and extension.
17. Construct SiteData by moving in site and tracks. The shorthand fields mean site: site, sun: sun, and earth: earth.
18. Set horizon to None. The TODO is a future DEM-loading hook; there is no DEM CLI option yet.
19. Serialize the wrapper, then write the file. A serialization error occurs before writing begins, but an I/O error could leave a partial file.
20. Print the path and number of Sun samples. Return Ok(()) after all sites.
21. Any propagated error stops the run; previously written sites are not rolled back.

The main function's Result return lets the Rust runtime report an error and exit unsuccessfully without adding a local exception handler.

<a id="s7-4"></a>
### 7.4 slug

New APIs:

- split on str takes a delimiter pattern and yields borrowed pieces. Here a char predicate treats every non-ASCII-alphanumeric character as a separator. Adjacent separators can produce empty pieces; the next filter removes them. Empty input initially yields an empty piece, then an empty final vector.
- is_ascii_alphanumeric takes a char and returns true only for A through Z, a through z, and 0 through 9. It is not locale-sensitive and excludes accented letters. Closest C++ equivalent: an explicit ASCII range test, not locale-dependent isalnum.
- str::to_ascii_lowercase takes a borrowed string and returns an owned String with ASCII capitals converted; other bytes are unchanged. Here the surviving segments already contain only ASCII alphanumerics.
- join on a slice of strings takes a separator string and returns a newly allocated String with the separator between items, never at the ends. Empty input returns empty. C++ has no direct std::vector<string>::join. This differs from Path::join, which combines filesystem components.

```rust
fn slug(name: &str) -> String
{
	name.split(|c: char| !c.is_ascii_alphanumeric())
		.filter(|s| !s.is_empty())
		.map(str::to_ascii_lowercase)
		.collect::<Vec<_>>()
		.join("-")
}
```

1. Split at every disallowed character.
2. Drop empty segments.
3. Map each segment through the named lowercase function. This passes a function as a callable rather than writing a closure.
4. Collect owned lowercase strings.
5. Join with a hyphen. The example name becomes example-89-45s-222-69e.

Uniqueness is not guaranteed by slug alone; main enforces it across sites.

<a id="s8"></a>
## 8. sites.json

```json
[
  { "name": "Example 89.45S 222.69E", "lat_deg": -89.45, "lon_deg": 222.69 }
]
```

1. The outer array maps to Vec<Site>.
2. The object keys match the public Site fields.
3. name is the human-readable label.
4. lat_deg is -89.45 degrees; lon_deg is 222.69 degrees.
5. Longitude is sent unchanged to Horizons. No terrain reference or filename is stored in this input.

JSON numbers cannot contain thousands separators. Numeric syntax in all exact source quotations is preserved.

<a id="s9"></a>
## 9. app/Cargo.toml

```toml
[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
eframe = { version = "0.37", default-features = false, features = ["default_fonts", "glow", "wayland", "x11"] }
egui_plot = "0.38"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = "0.11"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlCanvasElement"] }
```

1. The package metadata names app at version 0.1.0 using edition 2024.
2. eframe is the cross-platform application framework; egui is its immediate-mode UI library. Immediate mode means the program describes controls each UI pass rather than keeping an application-owned tree of control objects. Dear ImGui follows the same interface-construction model; eframe owns the platform event loop.
3. default-features = false selects only the listed integrations. default_fonts includes fonts. glow uses OpenGL natively and WebGL in browsers, graphics APIs for rendering. wayland and x11 enable the two common Linux window-system integrations.
4. egui_plot 0.38 is a compatible plotting package, present but not used by this placeholder. It need not have the same version number as eframe.
5. The first target-specific dependency table applies when target_arch is not wasm32. env_logger initializes native logging.
6. The second applies when target_arch is wasm32. wasm-bindgen-futures schedules browser futures. Rust paths replace a package-name hyphen with an underscore.
7. web-sys enables typed browser bindings. Window gives the page environment; Document gives its DOM; HtmlCanvasElement gives a graphics canvas.
8. app/src/lib.rs supplies the reusable application type, and app/src/main.rs supplies target-specific launch code.

<a id="s10"></a>
## 10. app/Trunk.toml

```toml
[build]
filehash = false
```

Trunk is the browser build tool and local development server. The build table contains filehash = false, so generated filenames omit content hashes. This differs from the Cargo target directory and does not select an eframe renderer. Trunk compiles wasm, generates its JavaScript loader bindings, copies requested assets, and serves the bundle during development.

<a id="s11"></a>
## 11. app/index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Moon Landing Visual</title>
	<link data-trunk rel="rust">
	<link data-trunk rel="copy-dir" href="assets">
	<base data-trunk-public-url>
	<style>
		html, body { margin: 0; width: 100%; height: 100%; overflow: hidden; background: #202020; }
		canvas { position: absolute; width: 100%; height: 100%; }
		#loading_text { position: absolute; top: 50%; width: 100%; text-align: center; color: white; font-family: sans-serif; }
	</style>
</head>
<body>
	<canvas id="the_canvas_id"></canvas>
	<div id="loading_text">Loading...<noscript>JavaScript is required.</noscript></div>
</body>
</html>
```

1. The document declaration requests modern HTML parsing. html lang="en" declares English content.
2. head holds metadata, not visible application controls.
3. charset="utf-8" declares text encoding. The viewport setting uses device width and initial scale one.
4. title sets the browser-tab title, independently of the native window title.
5. The first link's data-trunk rel="rust" asks Trunk to compile and bind the Rust executable.
6. The second copies the assets directory into build output. This includes the JSON files, although the app does not fetch them yet.
7. The base element lets Trunk insert the public URL prefix for deployed resources.
8. In CSS, html and body share zero margins, full width and height, hidden overflow, and a dark background.
9. The canvas selector gives the canvas absolute positioning and full size.
10. The #loading_text selector targets the element with that id, places its top at 50 percent, spans the width, centers white text, and selects a sans-serif font.
11. body contains the canvas whose id the Rust launcher searches for.
12. The loading div appears before wasm starts. noscript adds a notice when JavaScript is unavailable.
13. Closing tags finish the nested body and document.

There is no backend application server here. Trunk serves static browser files. Horizons downloading remains the separate native prep process.

<a id="s12"></a>
## 12. app/src/lib.rs

New APIs checked against [eframe::App](https://docs.rs/eframe/0.37.0/eframe/trait.App.html), [CentralPanel](https://docs.rs/egui/0.37.0/egui/containers/panel/struct.CentralPanel.html), and [Ui](https://docs.rs/egui/0.37.0/egui/struct.Ui.html):

- CentralPanel::default takes no input and constructs a standard central content panel.
- show consumes that panel, takes an exclusive parent Ui and a closure receiving the panel's Ui, and returns an InnerResponse containing the closure result and panel interaction information. Here the result is ignored. This is egui's layout API, not an operating-system window call. C++ requires an immediate-mode UI library, not the standard library.
- heading takes an exclusive Ui and text convertible to the widget text type, adds heading-styled text, and returns its interaction Response.
- label similarly adds ordinary label text and returns a Response. The code ignores both responses because no interaction is handled.
- The App trait's ui callback takes exclusive application state, a root Ui, and a Frame. eframe invokes it as needed for UI construction. Frame provides framework integration, not a raw graphics framebuffer.

```rust
#[derive(Default)]
pub struct MoonApp;

impl eframe::App for MoonApp
{
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		eframe::egui::CentralPanel::default().show(ui, |ui| {
			ui.heading("Moon Landing Visual");
			ui.label("The panorama is coming.");
		});
	}
}
```

1. Derive Default for the unit struct MoonApp. A unit struct has no fields; the default is just its sole value.
2. Export the struct so the executable crate can construct it through app::MoonApp.
3. Implement eframe::App for MoonApp, satisfying the framework's required callback. This is trait implementation rather than C++ class inheritance.
4. The callback borrows self exclusively even though it currently changes no application data.
5. _frame names the unused framework argument without a warning.
6. Construct a central panel and show it inside the supplied Ui.
7. Its closure adds a heading and the panorama-coming label.
8. There is no persistence, dataset loading, plot, or panorama drawing yet.

eframe 0.37 uses ui here. Tutorials targeting older update-based eframe interfaces are not an exact API match.

<a id="s13"></a>
## 13. app/src/main.rs

<a id="s13-1"></a>
### 13.1 Crate attribute and native entry

New APIs and attributes:

- cfg_attr conditionally attaches an attribute. not negates a configuration predicate. debug_assertions is normally enabled in development profiles and disabled in release.
- windows_subsystem = "windows" requests the Windows GUI subsystem, avoiding an extra console window for a Windows GUI executable. It does not select the wasm or native branch.
- cfg includes an item only when its predicate holds. target_arch = "wasm32" selects the browser-target main; its negation selects native main. Only one main exists in the compiled executable. This is Rust's syntax-aware counterpart to C++ #if.
- env_logger::init takes no input, installs a global logger, and returns unit. Configuration can come from RUST_LOG. It panics if logging was already initialized; main calls it once. [Reference](https://docs.rs/env_logger/0.11.11/env_logger/fn.init.html).
- NativeOptions::default constructs eframe's standard native window/renderer options. It returns a value, not a window.
- Box::new takes ownership of a value and returns heap ownership of it, comparable to make_unique. It is used for both the app-creation closure and the created application; allocation failure is not a Result here.
- eframe::run_native takes an application/window name, native options, and a boxed app-creation closure. It starts the native event loop and returns the framework's Result when it finishes or fails. The closure receives a creation context and returns a boxed App or an initialization error. Closest C++ equivalent requires a GUI framework's event-loop runner. [Reference](https://docs.rs/eframe/0.37.0/eframe/fn.run_native.html).

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

```rust
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result
{
	env_logger::init();
	eframe::run_native(
		"Moon Landing Visual",
		eframe::NativeOptions::default(),
		Box::new(|_| Ok(Box::new(app::MoonApp))),
	)
}
```

1. Initialize logging before launching the UI.
2. Pass Moon Landing Visual as the native title.
3. Use default window options.
4. Pass a heap-owned closure, ignoring its creation-context argument with an underscore pattern.
5. Construct the fieldless MoonApp, box it, and wrap it in Ok.
6. Return run_native's Result directly as the final expression. eframe::Result is a type alias for the framework's error-bearing return type.

<a id="s13-2"></a>
### 13.2 Wasm entry and startup/error paths

New APIs checked against [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures/0.4.80/wasm_bindgen_futures/fn.spawn_local.html), [JsCast](https://docs.rs/wasm-bindgen/0.2.130/wasm_bindgen/trait.JsCast.html), and [eframe's web runner source](https://docs.rs/eframe/0.37.0/src/eframe/web/web_runner.rs.html). DOM operations have no standard C++ equivalents; C++ compiled for a browser would need bindings to these same browser interfaces. The scheduling call corresponds to submitting asynchronous work to the browser event loop, not std::async on a new worker thread.

- wasm_bindgen_futures::spawn_local takes a future producing unit, schedules it on the current thread, and returns immediately. The future is run on a subsequent microtask tick even if already ready. A microtask is browser-scheduled work between ordinary event-loop tasks. This is not a new operating-system thread. The future must own or otherwise retain valid references for its scheduled lifetime.
- web_sys::window takes no input and returns an optional browser Window. It can be None outside a window context, such as a worker. There is no C++ standard browser equivalent. [Reference](https://docs.rs/web-sys/0.3.107/web_sys/fn.window.html).
- expect on Option consumes it and returns its value or panics with the provided message for None. On Result it similarly extracts Ok or panics with the message and error. It is used for required page structure, not ordinary recoverable input.
- document borrows a Window and returns an optional Document. [Reference](https://docs.rs/web-sys/0.3.107/web_sys/struct.Window.html#method.document).
- get_element_by_id borrows a Document and string id, returning an optional Element. Missing id gives None. It searches the DOM, not the filesystem. [Reference](https://docs.rs/web-sys/0.3.107/web_sys/struct.Document.html#method.get_element_by_id).
- dyn_into consumes a JavaScript-bound object and checks whether it has the requested target type. It returns Ok(converted object) or Err(original object). This is a runtime checked JavaScript type conversion, closer to checked dynamic_cast than a numeric cast. The JsCast trait makes the method available.
- WebRunner::new takes no input, installs a panic handler for logging, and constructs the browser runner. The following start borrows it and asynchronously takes a canvas, WebOptions, and an app-creation closure, returning a Result with unit success or a JavaScript error value. Graphics initialization, app creation, or browser integration failures can return an error. It attaches application rendering and event handling to that canvas. A C++ browser UI framework would need its own equivalent runner; the standard library has none.
- WebOptions::default takes no input and returns default web-runner options.
- remove borrows an Element and removes it from its parent in the DOM. It returns unit; already detached elements need no removal. [Reference](https://docs.rs/web-sys/0.3.107/web_sys/struct.Element.html#method.remove).
- set_text_content borrows a DOM Node and an optional string view, replaces its text content, and returns unit. Some supplies replacement text; None clears it. It sets text, not parsed HTML. [Reference](https://docs.rs/web-sys/0.3.107/web_sys/struct.Node.html#method.set_text_content).
- panic! formats a diagnostic and initiates a panic. It does not return a Result and cannot fall through to the next statement. Closest C++ terminating behavior is a diagnostic followed by failure; ordinary exception unwinding is not promised for this wasm target.

```rust
#[cfg(target_arch = "wasm32")]
fn main()
{
	use eframe::wasm_bindgen::JsCast as _;
	wasm_bindgen_futures::spawn_local(async {
		let document = web_sys::window()
			.expect("No window")
			.document()
			.expect("No document");
		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("Missing canvas")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("Not a canvas");
		let result = eframe::WebRunner::new()
			.start(
				canvas,
				eframe::WebOptions::default(),
				Box::new(|_| Ok(Box::new(app::MoonApp))),
			)
			.await;
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			match result
			{
				Ok(()) => loading.remove(),
				Err(err) =>
				{
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
```

1. Compile this main only for wasm32.
2. Import JsCast as an underscore so its methods participate in method resolution without a named local import.
3. Schedule an async block on the browser thread.
4. Obtain Window and Document. Missing required objects panic with explicit messages.
5. Find the canvas by its HTML id. Check that the returned element is an HtmlCanvasElement. A missing element or wrong tag panics before runner startup.
6. Construct a WebRunner and call start with the canvas and defaults.
7. The app-creation closure constructs the same MoonApp as native main.
8. await the startup Result without blocking the browser event loop.
9. Search for the loading notice. If absent, skip both notice-update branches.
10. If startup succeeded, remove the notice.
11. If startup failed and the notice exists, replace its text with a readable failure and then panic with a Debug-formatted error.
12. The async block finishes with unit on success. main itself returns after scheduling; browser callbacks keep the application running.

A subtle control-flow detail: a failed startup does not execute panic! if the loading notice is absent, because match is inside if let. The checked-in HTML contains that notice. None of these calls downloads the tracks; static assets are only copied by Trunk at this stage.

## 14. Alphabetical API and method index

The index includes every dot-call, macro call, and qualified function call found in the Rust source, plus named function references and local free functions for navigation. Same-spelling methods on different receivers are distinguished in their explanation sections. The section link points to the first explanation; later specializations also have links when needed.

- `abs`: [Section 5.3](#s5-3).
- `and_utc`: [Section 4.9](#s4-9).
- `assert!`: [Section 4.8](#s4-8).
- `assert_eq!`: [Section 4.7](#s4-7).
- `at`: [Section 4.2](#s4-2).
- `atan`: [Section 4.5](#s4-5).
- `Box::new`: [Section 13.1](#s13-1).
- `build`: [Section 7.3](#s7-3).
- `cfg`: [Section 4.10](#s4-10).
- `cfg_attr`: [Section 13.1](#s13-1).
- `checked_mul`: [Section 4.4](#s4-4).
- `clamp`: [Section 4.6](#s4-6).
- `collect`: [Section 4.7](#s4-7).
- `contains`: [Section 7.3](#s7-3).
- `cos`: [Section 4.3](#s4-3).
- `Debug`: [Section 4.1](#s4-1).
- `default`: [Section 4.8](#s4-8).
- `Default`: [Section 4.1](#s4-1).
- `Dem::height_at`: [Section 4.4](#s4-4).
- `derive`: [Section 4.1](#s4-1).
- `Deserialize`: [Section 4.1](#s4-1).
- `display`: [Section 7.3](#s7-3).
- `document`: [Section 13.2](#s13-2).
- `download`: [Section 7.2](#s7-2).
- `Duration::from_secs`: [Section 7.3](#s7-3).
- `dyn_into`: [Section 13.2](#s13-2).
- `earth_side_wall_at_example_longitude`: [Section 5.7](#s5-7).
- `east_at_lon_90_points_toward_negative_y`: [Section 5.6](#s5-6).
- `eframe::egui::CentralPanel::default`: [Section 12](#s12).
- `eframe::NativeOptions::default`: [Section 13.1](#s13-1).
- `eframe::run_native`: [Section 13.1](#s13-1).
- `eframe::WebOptions::default`: [Section 13.2](#s13-2).
- `eframe::WebRunner::new`: [Section 13.2](#s13-2).
- `ends_with`: [Section 7.3](#s7-3).
- `enumerate`: [Section 4.5](#s4-5).
- `env_logger::init`: [Section 13.1](#s13-1).
- `error_for_status`: [Section 7.2](#s7-2).
- `expect`: [Section 13.2](#s13-2).
- `f64::from`: [Section 4.5](#s4-5).
- `f64::NAN`: [Section 5.2](#s5-2).
- `filter`: [Section 4.9](#s4-9).
- `filter_map`: [Section 4.9](#s4-9).
- `flat`: [Section 5.1](#s5-1).
- `flat_horizon_includes_curvature`: [Section 5.3](#s5-3).
- `floor`: [Section 4.2](#s4-2).
- `format!`: [Section 4.9](#s4-9).
- `fract`: [Section 4.2](#s4-2).
- `fs::create_dir_all`: [Section 7.3](#s7-3).
- `fs::read_to_string`: [Section 7.3](#s7-3).
- `fs::write`: [Section 7.3](#s7-3).
- `get`: [HTTP client, Section 7.2](#s7-2); [slice lookup, Section 7.3](#s7-3).
- `get_element_by_id`: [Section 13.2](#s13-2).
- `heading`: [Section 12](#s12).
- `height_at`: [Section 4.4](#s4-4).
- `Horizon::at`: [Section 4.2](#s4-2).
- `horizon_from_dem`: [Section 4.5](#s4-5).
- `horizons_combined_optional_markers_and_last_numeric_pair`: [Section 5.13](#s5-13).
- `horizons_sample_and_errors`: [Section 5.12](#s5-12).
- `insert`: [Section 7.3](#s7-3).
- `interpolation_wraps`: [Section 5.8](#s5-8).
- `into`: [Section 4.9](#s4-9).
- `is_ascii_alphanumeric`: [Section 7.4](#s7-4).
- `is_empty`: [Section 4.2](#s4-2).
- `is_err`: [Section 5.12](#s5-12).
- `is_finite`: [Section 4.4](#s4-4).
- `is_some_and`: [Section 4.9](#s4-9).
- `iter`: [Section 4.7](#s4-7).
- `iter_mut`: [Section 4.5](#s4-5).
- `join`: [paths, Section 7.3](#s7-3); [string slices, Section 7.4](#s7-4).
- `label`: [Section 12](#s12).
- `last`: [Section 4.9](#s4-9).
- `len`: [Section 4.2](#s4-2).
- `lines`: [Section 4.9](#s4-9).
- `main (native)`: [Section 13.1](#s13-1).
- `main (prep)`: [Section 7.3](#s7-3).
- `main (wasm)`: [Section 13.2](#s13-2).
- `map`: [iterators, Section 4.7](#s4-7); [Option, Section 7.3](#s7-3).
- `map_err`: [Section 4.9](#s4-9).
- `max`: [Section 4.5](#s4-5).
- `min`: [Section 4.4](#s4-4).
- `mod`: [Section 4.10](#s4-10).
- `NaiveDate::parse_from_str`: [Section 7.3](#s7-3).
- `NaiveDateTime::parse_from_str`: [Section 4.9](#s4-9).
- `next`: [Section 4.9](#s4-9).
- `north_ridge_away_from_pole_singularity`: [Section 5.5](#s5-5).
- `not`: [Section 13.1](#s13-1).
- `ok`: [Section 4.9](#s4-9).
- `ok_or`: [Section 4.9](#s4-9).
- `or_else`: [Section 4.9](#s4-9).
- `panic!`: [Section 13.2](#s13-2).
- `parse`: [Section 4.9](#s4-9).
- `parse_horizons`: [Section 4.9](#s4-9).
- `Path::new`: [Section 7.3](#s7-3).
- `pole_and_bilinear_grid`: [Section 5.2](#s5-2).
- `powi`: [Section 5.4](#s5-4).
- `println!`: [Section 7.3](#s7-3).
- `push`: [Section 4.9](#s4-9).
- `query`: [Section 7.2](#s7-2).
- `rem_euclid`: [Section 4.2](#s4-2).
- `remove`: [Section 13.2](#s13-2).
- `reqwest::blocking::Client::builder`: [Section 7.3](#s7-3).
- `rev`: [Section 4.9](#s4-9).
- `send`: [Section 7.2](#s7-2).
- `serde_json::from_str`: [Section 7.3](#s7-3).
- `serde_json::to_string_pretty`: [Section 7.3](#s7-3).
- `Serialize`: [Section 4.1](#s4-1).
- `set_text_content`: [Section 13.2](#s13-2).
- `should_panic`: [Section 5.11](#s5-11).
- `show`: [Section 12](#s12).
- `sin`: [Section 4.3](#s4-3).
- `skip`: [Section 7.3](#s7-3).
- `slug`: [Section 7.4](#s7-4).
- `split`: [Section 7.4](#s7-4).
- `split_once`: [Section 4.9](#s4-9).
- `split_whitespace`: [Section 4.9](#s4-9).
- `start`: [Section 13.2](#s13-2).
- `stats`: [Section 4.8](#s4-8).
- `Stats::default`: [Section 4.8](#s4-8).
- `stats_rejects_unordered_times`: [Section 5.11](#s5-11).
- `std::collections::HashSet::new`: [Section 7.3](#s7-3).
- `std::env::args`: [Section 7.3](#s7-3).
- `String::as_str`: [Section 7.3](#s7-3).
- `str::to_ascii_lowercase`: [Section 7.4](#s7-4).
- `sun_fraction`: [Section 4.6](#s4-6).
- `tan`: [Section 4.3](#s4-3).
- `test`: [Section 5](#s5).
- `text`: [Section 7.2](#s7-2).
- `then_some`: [Section 4.4](#s4-4).
- `timeout`: [Section 7.3](#s7-3).
- `timestamp`: [Section 4.9](#s4-9).
- `to_degrees`: [Section 4.3](#s4-3).
- `to_radians`: [Section 4.3](#s4-3).
- `to_xy`: [Section 4.3](#s4-3).
- `Track::default`: [Section 4.8](#s4-8).
- `trim`: [Section 4.9](#s4-9).
- `ui`: [Section 12](#s12).
- `unwrap`: [Section 5.12](#s5-12).
- `unwrap_or`: [Section 7.3](#s7-3).
- `vec!`: [Section 4.5](#s4-5).
- `visibility_and_solar_disc`: [Section 5.9](#s5-9).
- `visible`: [Section 4.7](#s4-7).
- `wall_angle_and_local_azimuth`: [Section 5.4](#s5-4).
- `wasm_bindgen_futures::spawn_local`: [Section 13.2](#s13-2).
- `web_sys::window`: [Section 13.2](#s13-2).
- `weighted_stats_include_gap`: [Section 5.10](#s5-10).
- `windows`: [Section 4.8](#s4-8).
- `zip`: [Section 4.7](#s4-7).
