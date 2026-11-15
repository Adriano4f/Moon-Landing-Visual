# 01: Understanding the skeleton from C++

This guide assumes strong C++ algorithm skills and no Rust or browser experience. It describes the checked-in skeleton, not a finished landing-site planner. Read in order. The language reference comes first so that the later source walkthrough can use its vocabulary.

## 1. Build vocabulary before syntax

A compiler translates source into machine code or another executable representation. A linker combines compiled pieces. Rust's compiler is rustc. Cargo is its build tool, dependency downloader, and test runner. It plays several roles that a C++ project might distribute among CMake, Ninja, a package manager, and a test runner.

A crate is one Rust compilation unit. It is either a library or an executable. A package is a named distribution described by a manifest file named Cargo.toml. One package may contain a library crate and an executable crate. By convention, src/lib.rs starts the library and src/main.rs starts the executable. A workspace groups packages under one Cargo invocation. It resembles a CMake project containing several targets, but a package is not exactly a CMake target. Each package can produce several crates.

A dependency is another package used by the current package. A feature is a named, additive switch that enables optional parts of a dependency. Features are not independent preprocessor definitions. Cargo generally unifies requested features of a dependency for a build context.

A target is the platform and execution environment being compiled for. Native means running directly as an operating-system program. WebAssembly, abbreviated wasm, is a portable executable format that browsers can run inside a sandbox. This project uses the target named wasm32-unknown-unknown. It has 32-bit addresses and does not provide a normal operating-system interface.

TOML is the configuration format used by Cargo.toml. A line such as `name = "app"` assigns a string value to a key. Square-bracket headers introduce tables. An array is a comma-separated list in square brackets. Braces contain an inline table of key/value pairs. TOML booleans are `true` and `false`. A dotted key denotes nested tables. Single-quoted strings are literal strings; double-quoted strings support escapes. These are configuration syntax, not Rust syntax.

Cargo.lock records exact resolved dependency versions. Commit it for this application workspace. Cargo.toml version requirements allow compatible updates; Cargo.lock makes ordinary builds repeatable at the dependency-version level. The lockfile does not make external Horizons responses or system libraries reproducible.

## 2. Rust vocabulary in dependency order

### 2.1 Names, primitive values, and bindings

An identifier names a value, type, function, or module. A module groups Rust names, roughly like a C++ namespace with source-file organization. A path joins names with `::`, comparable to C++ namespace qualification. `std` is Rust's standard library. `use` brings a path into scope, like a C++ using-declaration. A brace group in a use-declaration imports several names from the same prefix. `as` in a use-declaration renames an import. Underscore imports are explained with traits below.

Rust's primitive types used here are `bool`, `usize`, `u32`, `i64`, `f32`, `f64`, and `char`. The numeric types correspond to boolean, pointer-sized unsigned integer, 32-bit unsigned integer, 64-bit signed integer, IEEE binary32, and IEEE binary64. C++ analogues are bool, size_t, uint32_t, int64_t, float, and double. A char is a Unicode scalar value, not a C++ single-byte char. Single quotes delimit character literals such as `'m'`. `usize` is used for vector lengths and indexing. Its width follows the target, not necessarily the development machine. A suffix such as `500.0_f64` fixes a literal's type. Underscores inside digits improve readability without changing value: `1_737_400.0` is 1,737,400.0.

`let` introduces a local binding. `let x = 1.0;` is immutable by default, closer to a local C++ const than to an ordinary local variable. `let mut x = 1.0;` permits reassignment. A colon adds an explicit type, as in `let mut night: f64 = 0.0;`. Otherwise the compiler infers types from use. Shadowing introduces a new binding with an existing name. It is not assignment to the old binding.

Operators `+`, `-`, `*`, `/`, `%`, comparisons, `&&`, `||`, `!`, and compound assignments have familiar C++ roles. `%` computes a remainder. Floating-point division is selected when the operands are floating-point. `as` in an expression is a numeric cast here, comparable to static_cast, but not identical: float-to-integer casts truncate, saturate outside range, and turn NaN into zero. Integer-to-float casts can lose precision. Casts do not validate scientific input.

`const` declares a compile-time constant. `pub` exposes an item outside its enclosing module, unlike C++ where struct fields are public by default. Rust struct fields are private unless explicitly public. `//` starts a line comment. `///` documents the following item and can appear in generated API documentation.

### 2.2 Functions, expressions, and control flow

`fn` declares a function. Parameters have the form name followed by a colon and a type. An arrow `->` introduces the return type. A function body is enclosed in braces. Rust usually returns the final expression if it has no trailing semicolon. A semicolon turns that expression into a statement. `return value;` exits early, like C++ return. No arrow means the unit type `()`, similar in purpose to void.

For example, after the definitions above, this is a complete function:

```rust
fn twice(x: f64) -> f64
{
	x * 2.0
}
```

`if` and `else` select branches. Conditions do not require parentheses. Braced blocks can themselves be expressions. `while` repeats while a condition holds. `for value in sequence` visits elements. `break` exits the nearest loop. A range `a..b` excludes b; `a..=b` includes b. Range bounds can be omitted in slicing expressions described below. An initial underscore in a parameter name, such as `_frame`, suppresses the unused-variable warning. A bare underscore discards a value or pattern position.

### 2.3 Data types and ownership

`struct` defines a product type with named fields, comparable to a C++ struct. A struct literal uses braces and `field: value` entries. Writing just `field` abbreviates `field: field`. Rust separates fields from methods. An `impl Type` block defines associated functions and methods for that type. Associated functions are called with `Type::function`, like static members. Methods use dot syntax. In an implementation, `Self` refers to the implementing type.

`self` is the receiving object, analogous to the object behind C++ this. Rust makes its access mode explicit. `&self` borrows the object immutably; `&mut self` borrows it mutably. A unit struct such as `struct MoonApp;` has no fields and needs no per-instance data.

Rust values have owners. Moving a value transfers ownership; the old binding can no longer be used as the owner. At the end of its owner's lifetime, a value is dropped, like deterministic C++ RAII destruction. This prevents accidental copies of owning containers. Types such as numbers and booleans are cheaply copied automatically. Explicitly cloning an owning value is a separate operation, not an implicit consequence of assignment.

`&T` is a shared reference to a value of type T, roughly a C++ const reference. `&mut T` is an exclusive reference, roughly a C++ mutable reference with stronger rules. While an exclusive borrow is active, no other live reference may access the same value. Shared borrows can coexist, but cannot mutate the borrowed ordinary data. References must not outlive their referents. The compiler checks these rules. Unlike a C++ const reference, a shared reference cannot coexist with arbitrary mutation through another ordinary reference. `&value` creates a shared borrow, `&mut value` creates an exclusive borrow, and `*reference` dereferences it.

Generic types use angle brackets, as C++ templates do. `Vec<T>` owns a dynamically sized contiguous array of T, similar to std::vector. `Vec::new()` constructs an empty vector. `push` appends. `len` gives the length; `is_empty` tests it. Indexing with square brackets checks bounds and panics on failure. `get` returns an optional reference instead. `String` owns UTF-8 text, unlike a byte-oriented std::string with no encoding invariant. `str` is a text view type; `&str` is a borrowed UTF-8 string view, roughly std::string_view with UTF-8 guarantees. A quoted string literal is a shared string view with program-long storage. Escapes such as `\n` represent a newline.

`&[T]` is a borrowed slice of contiguous T values with a length, comparable to std::span<const T>. A mutable slice would be exclusive. `&values[a..b]` borrows a subrange. `[..]` covers the whole range. `&args[1]` borrows one element, not a slice. `[true; 2]` is a fixed-size array repeating one value. `vec![0.0; n]` constructs a vector with n equal initial values. A tuple groups positional values, like std::tuple; `(x, y)` constructs a pair. `let (x, y) = pair;` destructures it, like a structured binding.

### 2.4 Optional values, errors, and traits

An enum is a type with alternative named variants, similar in purpose to a tagged std::variant. `Option<T>` has variants `Some(value)` and `None`. It corresponds to std::optional<T>. It does not require a heap allocation. `Some` and `None` are names available without an explicit import.

Patterns describe shapes to recognize and unpack. `let Some(z) = expression else { return value; };` extracts z on success and executes the else block on None. That else block must leave the current control path. `if let Some(x) = expression` runs a block only when the pattern matches. `match` selects among patterns, with `=>` separating each pattern from its expression. A vertical bar joins alternatives in a pattern. Patterns such as `&az` unpack a reference by copying the referenced number into az.

`Result<T, E>` has variants `Ok(value)` and `Err(error)`. It expresses a fallible operation without throwing a C++ exception. The caller must handle or propagate the result. In a function returning Result, `expression?` extracts Ok or returns the Err early, converting the error when an appropriate conversion exists. In a function returning Option, `?` similarly returns None early. It does not catch a panic. `expect(message)` and `unwrap()` extract success or panic on failure. A panic stops normal execution; its behavior is detailed with assertions below. `unwrap_or(fallback)` substitutes a fallback. `or_else` computes an alternative on failure; `map_err` transforms only an error. Their callable syntax is introduced next.

A trait defines behavior that types can implement. It is related to a C++ concept as an interface for generic code, but can also support runtime dispatch like a virtual interface. `impl Trait for Type` implements that behavior. There is no requirement that the type inherit a base class. Importing a trait with `use path::Trait as _;` makes its methods available without binding its name. `dyn Trait` denotes a runtime-dispatched trait object. `Box<T>` owns a heap allocation, like std::unique_ptr<T>. `Box<dyn Error>` owns a value implementing the standard Error trait, allowing different error types behind one return type. `Result<(), Box<dyn Error>>` therefore means either successful completion with no data or an owned error. `into()` requests a type conversion inferred from context. It uses the Into trait, not an unchecked cast.

An attribute is compiler-readable metadata attached to an item. `#[...]` applies to the following item. `#![...]` applies to the enclosing crate or module. `#[derive(...)]` asks the compiler or a library's code generator to generate trait implementations. Here Debug enables developer formatting, Default constructs a conventional initial value, Serialize converts values to a data format, and Deserialize constructs them from that format. Deriving these traits is not C++ reflection at runtime. The generated implementations are compiled normally.

### 2.5 Closures, iterators, and macros

A closure is an anonymous callable, analogous to a C++ lambda. `|x| expression` declares a parameter and body; `|x, y| { ... }` takes two. `|| expression` takes no parameters. Captures are inferred as shared borrows, exclusive borrows, or ownership transfers according to use. C++ capture lists have no exact syntactic equivalent here.

An iterator produces a sequence lazily. `.iter()` borrows elements; `.iter_mut()` yields exclusive element references. A for-loop over an owned vector consumes it. `.zip(other)` pairs elements and stops at the shorter input. `.map(closure)` transforms elements lazily. `.filter(closure)` retains selected elements. `.collect()` builds a collection and its result type comes from context. This resembles C++ ranges pipelines. `.collect::<Vec<_>>()` explicitly chooses a vector while `_` asks the compiler to infer the element type. The `::` before these generic arguments disambiguates expression syntax. `Result<Vec<_>, _>` can also be collected: the first error stops collection. `.enumerate()` attaches zero-based indices. `.any()` checks whether at least one element matches. `.last()` returns the last item when available.

Named functions can also be passed as callables. For example, `map(String::as_str)` maps owned-string references to borrowed string views without a wrapper closure. Dot-call chains pass each result to the next method. A builder is an object whose methods configure it before a final build call; this project uses that pattern for the network client. filter passes a reference to each iterator item, so a filter over an iterator of string references can require two dereferences, as in `**s`. Method-call syntax often inserts the needed borrow or dereference automatically. `f64::from(value)` performs a supported conversion through the From trait; it is distinct from an `as` cast.

Macros generate code from token input and use `!` after their names. Unlike C++ textual preprocessor macros, Rust macros participate in the language's syntax and name-handling rules. `format!` builds a String; `{}` formats a value, `{name}` inserts a named value, and `{:?}` requests Debug formatting. `println!` prints a formatted line. `vec!` was introduced above. `matches!(value, pattern)` returns whether a pattern matches. `assert!(condition)` and `assert_eq!(a, b)` panic if their condition fails. Rust assertions remain enabled in release builds. A panic is an unrecoverable programming-failure signal, not an ordinary Result error. Native builds normally unwind the stack unless configured to abort. On this wasm target a panic typically traps. Do not assume C++ exception recovery is available.

### 2.6 Source selection, tests, and asynchronous code

`mod tests;` declares a module loaded from a neighboring tests.rs file in this layout. `super` refers to the parent module. `use super::*;` imports its accessible names; `*` here is a glob import, not multiplication. `#[cfg(test)]` includes an item only in test builds. `#[test]` registers a test function. `#[should_panic]` says that a test succeeds only if it panics.

`#[cfg(target_arch = "wasm32")]` includes an item only for that target architecture. It resembles `#ifdef`, but selects syntax-aware Rust items. `not(...)` negates the configuration condition. `#[cfg_attr(condition, attribute)]` conditionally applies another attribute. In this project, `windows_subsystem = "windows"` suppresses a console window for Windows release applications when `debug_assertions` is false. Configuration predicates in Cargo target dependency tables use the same ideas.

A future is a value representing asynchronous work that can progress when polled. `async { ... }` creates a future. `.await` suspends that future until another future completes; it does not synchronously block the browser thread. A browser event loop schedules callbacks and rendering on its thread. `spawn_local` schedules a future on that thread without requiring it to be transferable to other threads. This is different from starting a C++ worker thread. The native prep program instead uses blocking network calls, which wait until each response is ready.

## 3. Workspace organization

The repository has three package directories. core contains a library named lunar_core. Naming it core would collide with Rust's built-in core library. It owns scientific data types and calculations, with no network or UI dependency. prep contains the data-fetching executable. app contains a library for the application object and an executable with platform-specific entry functions. This separation keeps tests independent of graphics and network access.

sites.json is input data. JSON is a text format with objects in braces, arrays in brackets, string keys, numbers, booleans, and null for absence. The checked-in array contains one Site. app/assets/data contains downloaded output. A browser asset is a file served alongside the application. The app currently does not read or visualize these tracks.

The root Cargo.toml has exactly three meaningful lines:

1. `[workspace]` declares a workspace rather than a root package.
2. `members = ["core", "prep", "app"]` names its package directories.
3. `resolver = "3"` chooses Cargo's version-3 dependency resolver, including Rust-version-aware dependency selection. It is appropriate for edition 2024.

The .gitignore excludes Cargo's target build directory, Trunk's dist output, and large terrain raster files with the listed tif, tiff, img, and IMG extensions. Generated example JSON is deliberately committed.

## 4. Every package manifest

### 4.1 core/Cargo.toml

`[package]` opens package metadata. `name = "lunar_core"` sets the package name and default library import name. `version = "0.1.0"` is this package's own version, not a compiler version. `edition = "2024"` selects Rust's 2024 syntax and compatibility rules; it does not mean every 2024 compiler supports every dependency.

`[dependencies]` starts dependencies for this package. `serde = { version = "1", features = ["derive"] }` requests compatible Serde 1 releases and the code generators for Serialize and Deserialize. Serde describes format-independent serialization; it does not itself choose JSON. `chrono = "0.4"` supplies date/time parsing and UTC conversion. These ordinary version strings are compatible-version requirements, not exact pins.

### 4.2 prep/Cargo.toml

`[package]`, `version = "0.1.0"`, and `edition = "2024"` have the meanings already defined. `name = "prep"` names this executable package.

Under `[dependencies]`, `lunar_core = { path = "../core" }` resolves our local library relative to this manifest. It does not download a same-named library from the registry.

`reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }` supplies an HTTP client. HTTP is the request/response protocol used by web services. HTTPS encrypts and authenticates HTTP using TLS. The blocking feature gives a synchronous client. Disabling default features prevents the default native TLS backend. The rustls-tls feature selects Rustls with its certificate-root support. No OpenSSL or pkg-config installation is needed for these downloads. This does not remove the need for a native compiler and linker for other dependencies.

`serde = { version = "1", features = ["derive"] }` generates serialization for prep's output wrapper. `serde_json = "1"` implements the JSON encoding and decoding that uses those Serde traits. `chrono = "0.4"` validates command-line dates before the network calls.

### 4.3 app/Cargo.toml

The package table names app, gives it version 0.1.0, and selects edition 2024. The dependency table requests `eframe = { version = "0.37", default-features = false, features = ["default_fonts", "glow", "wayland", "x11"] }`. eframe supplies a cross-platform application loop and window integration. Its egui library supplies UI elements. default_fonts embeds usable fonts. glow chooses the OpenGL renderer on native platforms and WebGL in browsers. OpenGL is a graphics API; WebGL exposes a restricted graphics API in a browser. wayland and x11 enable the two common Linux window-system integrations. Default features are disabled so the skeleton does not implicitly select the larger default renderer and integrations.

`egui_plot = "0.38"` supplies plotting widgets compatible with this egui generation. Its version number need not equal eframe's. It is present for future plots but currently unused.

`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` selects dependencies only for non-wasm builds. `env_logger = "0.11"` initializes native logging with environment-variable configuration.

`[target.'cfg(target_arch = "wasm32")'.dependencies]` selects browser-target dependencies. `wasm-bindgen-futures = "0.4"` connects Rust futures to browser scheduling. Rust imports a hyphenated package name with underscores. `web-sys = { version = "0.3", features = ["Window", "Document", "HtmlCanvasElement"] }` provides typed bindings to browser objects. A Window is the browser's page environment. A Document is the structured HTML page. An HtmlCanvasElement is a drawing surface in that document. Features enable only the requested binding groups and their required dependencies.

## 5. core/src/lib.rs, in source order

### 5.1 Imports, constants, and data

The first two use-declarations import Chrono's timezone-free date/time type and Serde's traits. MOON_RADIUS_M is 1,737,400.0 m, or 1737.4 km. SUN_RADIUS_DEG is the assumed apparent solar radius, 0.267 degrees. These are simplified fixed constants, not a full lunar shape or variable solar-disc model.

Site stores a name and latitude/longitude in degrees. Negative latitude means south. Track stores parallel vectors: UTC Unix seconds in t, azimuth degrees in az_deg, and elevation degrees in el_deg. UTC is the civil time standard used here. Unix time counts seconds from 1970-01-01 00:00:00 UTC using the usual non-leap-second Unix representation. Azimuth 0 means north and increases clockwise through east. Elevation is measured above the local horizontal plane. Matching vector indices refer to one observation. Public fields permit callers to construct inconsistent lengths, so visible asserts their lengths.

Horizon stores evenly spaced azimuth-bin heights. Bin i corresponds to 360i/n degrees for n bins. All these types derive serialization and developer formatting. Track also derives Default, which makes its vectors empty.

### 5.2 Horizon::at

Assumptions: the profile has n positive, evenly spaced bins; azimuth is finite; linear interpolation is acceptable between bins.

First wrap the azimuth with a nonnegative remainder: a = azimuth modulo 360, in [0, 360). Rust's rem_euclid implements this even for negative input. Then p = an/360. Set i = floor(p) modulo n and f = p - floor(p). The next index is j = (i + 1) modulo n. The interpolated height is h(i) + f(h(j) - h(i)). At a full turn, the wrapped a is zero. Between the final bin and north, j becomes zero. Thus the interpolation is periodic rather than clamped at the seam.

The empty-profile branch returns -90 degrees as a sentinel for no known obstruction. A one-bin profile returns its sole value. This API does not validate non-finite azimuth input; callers should supply finite angles.

### 5.3 Dem and to_xy

A DEM is a digital elevation model: a grid of terrain heights. Dem stores dimensions w and h, grid spacing res_m, the x coordinate of the leftmost sample, the y coordinate of the top sample, and row-major heights. Row-major index (r, c) is r times w plus c. All positions are meters. Rows increase toward decreasing y. Coordinates refer to sample centers, not outer pixel corners.

Assumptions: the Moon is a sphere of radius R; latitude phi and longitude lambda are in radians during calculation; the south polar stereographic projection has scale one at its center; the chosen DEM follows the stated orientation.

Let delta be the angular distance from the south pole. Then delta = pi/2 + phi. In a meridional cross-section, the sphere's transverse distance is R sin(delta) and its coordinate toward the tangent-plane normal is R cos(delta). Projecting from the opposite pole onto the south-pole tangent plane forms similar triangles. Their ratio is 2R / (R + R cos(delta)). Therefore:

```text
rho = R sin(delta) * 2R / (R + R cos(delta))
	= 2R sin(delta) / (1 + cos(delta))
	= 2R tan(delta / 2)
	= 2R tan(pi/4 + phi/2)
x = rho sin(lambda)
y = rho cos(lambda)
```

to_xy converts the degree inputs with to_radians and evaluates these equations. At phi = -pi/2, the tangent argument is zero, so rho = 0 and both coordinates vanish. At longitude zero, points away from the south pole lie on positive y. The doc comment deliberately requires checking the sign/orientation convention against the DEM label. Real raster products can differ in reference longitude, axis direction, scale, offsets, height datum, and no-data encoding.

### 5.4 Dem::height_at

Assumptions: the grid is valid; spacing d is positive and finite; four neighboring heights are finite; interpolation within a grid cell is bilinear.

Map meters to fractional grid coordinates:

```text
col = (x - x_min) / d
row = (y_max - y) / d
c = floor(col), r = floor(row)
fx = col - c, fy = row - r
```

Let z00 be the top-left height, z10 top-right, z01 bottom-left, and z11 bottom-right. Linear interpolation along the two rows gives top = (1 - fx)z00 + fx z10 and bottom = (1 - fx)z01 + fx z11. Interpolating those results along the column gives:

```text
z = (1 - fy)top + fy bottom
  = (1 - fy)(1 - fx)z00 + (1 - fy)fx z10
	+ fy(1 - fx)z01 + fy fx z11
```

The four weights sum to one. On an edge, clamping the upper neighbor index to the last sample lets the same expression work without reading outside the vector.

The implementation rejects empty dimensions, invalid spacing, dimension-product overflow, wrong storage length, non-finite coordinates, and coordinates outside the sample-center rectangle. checked_mul returns Option so `?` can reject overflow. A non-finite interpolated height also becomes None through then_some. It does not interpret a finite no-data sentinel; DEM loading must handle such values later.

### 5.5 horizon_from_dem and the corrected directions

Assumptions: site and terrain use the same height datum; terrain is sampled over short distances relative to R; projected meter offsets approximate local horizontal distances; refraction is neglected; the DEM orientation matches to_xy. The local planar approximation neglects stereographic scale variation away from the pole. Longitude selects a limiting orientation at the pole itself, where geographic north is singular.

The function allocates n_az bins initialized to -90 degrees. It projects the site and reads its height z0. An outside-grid site returns the sentinel profile. The radial marching step is d/2 for DEM spacing d. A non-finite maximum distance returns without marching.

For the local basis, use geographic north, as Horizons does. North increases latitude and therefore points away from the south pole. Since rho increases with latitude, the north unit vector is (sin(lambda), cos(lambda)). East follows increasing east-positive longitude:

```text
N = (sin(lambda), cos(lambda))
E = d/dlambda (sin(lambda), cos(lambda))
  = (cos(lambda), -sin(lambda))
N dot E = sin(lambda)cos(lambda) - cos(lambda)sin(lambda) = 0
|N| = |E| = 1
```

Azimuth A uses north with coefficient cos(A) and east with coefficient sin(A). Apply angle addition identities component by component:

```text
D = cos(A)N + sin(A)E
Dx = cos(A)sin(lambda) + sin(A)cos(lambda)
   = sin(A + lambda)
Dy = cos(A)cos(lambda) - sin(A)sin(lambda)
   = cos(A + lambda)
x(s) = x0 + s sin(A + lambda)
y(s) = y0 + s cos(A + lambda)
```

At longitude zero, north increases y and east increases x. At longitude 90 degrees, north increases x and east decreases y. The source uses angle A + longitude and adds both ray components, without changing the projection.

Horizons provides an independent orientation check: at latitude -89.45 degrees and longitude 222.69 degrees, the archived hourly Earth azimuths within January 2027 span 129.583760 to 144.690080 degrees, near the reported 137.3-degree Earth-side bearing. A ray in the positive-y direction has A + longitude = 360 degrees, so A = 360 - 222.69 = 137.31 degrees. This is the map direction toward longitude zero and agrees with the observed Earth-side sector. The former mirrored formula instead placed that direction at 42.69 degrees. A synthetic wall regression test checks the corrected 137.31-degree peak; Earth's varying position is not asserted to be exactly on positive y at every time.

For curvature, take a circle of radius R and a local tangent line. Let s be horizontal distance along that tangent direction. A same-radius point below it satisfies a right triangle:

```text
vertical radius component = sqrt(R^2 - s^2)
drop = R - sqrt(R^2 - s^2)
	 = s^2 / (R + sqrt(R^2 - s^2))
	 approximately s^2 / (2R), when s/R is small
```

Strictly, s in the square-root equation is the horizontal chord projection, not arc length along the surface. If the surface arc length is l, the exact drop is R(1 - cos(l/R)); its small-angle expansion is also l^2/(2R) to leading order. The marching code uses local projected distance and the leading-order approximation.

If dz = z(s) - z0 is the terrain's height difference, curvature reduces the terrain's apparent height by this drop. The angle above the site's tangent plane is therefore:

```text
effective height = dz - s^2/(2R)
tan(theta) = effective height / s
theta = atan((dz - s^2/(2R))/s)
h(A) = maximum theta over sampled distances
```

The code converts theta to degrees and retains the maximum. It stops at the requested distance or the first grid edge or missing sample. It does not interpolate a final step beyond the distance limit, model an observer mast, or inspect unsampled narrow peaks. A flat DEM has dz = 0, so theta = atan(-s/(2R)). The maximum occurs at the nearest positive sample, producing a slightly negative horizon.

### 5.6 sun_fraction and visible

Assumptions: the Sun has fixed angular radius r; the terrain threshold is h; a linear transition across the solar diameter is sufficient. This is a proxy for illuminated fraction, not the exact area of a circular segment.

When the center elevation el is h - r, visibility should be zero. When el is h + r, visibility should be one. The affine map through these endpoints is derived by shifting and dividing:

```text
u = (el - (h - r)) / ((h + r) - (h - r))
  = (el - h + r) / (2r)
fraction = min(1, max(0, u))
```

sun_fraction implements that clamp with r = 0.267 degrees. visible instead uses a binary test. It asserts matching Track vector lengths, pairs azimuth and elevation with zip, interpolates the horizon, and returns whether el is strictly greater than h + margin_deg. Equality is not visible. map constructs one boolean per pair; collect materializes the resulting vector.

### 5.7 Stats and stats

Stats contains percentages for Sun, Earth, and simultaneous visibility, plus longest continuous night in hours. stats accepts a timestamp slice and two boolean slices. It does not accept solar fractions. The booleans are assumed constant until the next sample. This convention counts a data gap as one fully represented interval, not missing data.

Assumptions: lengths agree; timestamps strictly increase; the left sample represents its entire following interval. For interval i define dt(i) = (t(i+1) - t(i))/3600 hours. Define S(i) and E(i) to be 1 for true and 0 for false. Then:

```text
T = sum dt(i)
sun hours = sum dt(i) S(i)
earth hours = sum dt(i) E(i)
both hours = sum dt(i) S(i) E(i)
pct_sun = 100 * sun hours / T
pct_earth = 100 * earth hours / T
pct_both = 100 * both hours / T
```

The last timestamp defines an endpoint, not an additional interval. Empty and singleton series have no represented duration, so the default all-zero Stats is returned. For longest night, accumulate dt while S is false, reset the accumulator when S is true, and retain its largest value. Consecutive dark intervals across a long timestamp gap therefore remain one night. windows(2) provides adjacent timestamp slices. Invalid lengths or non-increasing times panic rather than return a recoverable error.

### 5.8 parse_horizons

JPL Horizons is an ephemeris service: it computes celestial-body positions over time. parse_horizons accepts its text response, not arbitrary CSV or every Horizons configuration. split_once finds the first start marker `$$SOE`, then the first end marker `$$EOE` after it. Missing markers produce String errors. Lines outside the markers are ignored. Blank lines inside are skipped.

Each row is split on whitespace. Its first two fields are the calendar date and UTC clock time. Chrono parses a year, abbreviated English month, day, and hour/minute, with a fallback accepting seconds. and_utc assigns UTC; timestamp produces Unix seconds. The parser removes its explicit list of known flag tokens, including `*`, C, and m. It then parses exactly two finite f32 values as azimuth and elevation. Unknown flags or unavailable angles are errors, not silently skipped observations. It rejects non-increasing timestamps and an empty ephemeris.

The conversion pipeline uses iterator collection into Result so any angle parse failure returns an error. `is_some_and` tests a predicate only if the previous timestamp exists. format builds contextual error messages. This parser is deliberately small and tied to the requested quantity and angle format.

## 6. core/src/tests.rs

The test module imports its parent's names with super. The flat helper constructs a 21 by 21 zero-height DEM with 100 m spacing and the pole centered in the grid.

1. pole_and_bilinear_grid checks that projection at the south pole is zero, that midpoint interpolation gives 3.0 for corner heights 0, 2, 4, 6, that an edge sample is accepted, and that outside/non-finite coordinates are rejected.
2. flat_horizon_includes_curvature checks all eight ray bins against atan(-50/(2R)). The 50 m distance is half the 100 m grid spacing. It also checks an outside-grid site and zero requested bins.
3. wall_angle_and_local_azimuth puts a 100 m wall at y = -500 m. With the limiting longitude-90 basis at the pole, this is east. Its expected angle includes the 500 m curvature correction. The opposite ray stays negative. This checks magnitude but, alone, would not resolve the pole's geographic singularity.
4. north_ridge_away_from_pole_singularity places the site at latitude -89 degrees, longitude zero. The grid is centered around the actual projected site and has enough extent to contain it. A ridge 500 m toward larger y lies geographically north, away from the south pole. The north bin must exceed 10 degrees while the south bin remains negative.
5. east_at_lon_90_points_toward_negative_y uses latitude -89 degrees, longitude 90 degrees. A ridge toward smaller y must appear east, not west. This independently checks the east basis away from the pole.
6. earth_side_wall_at_example_longitude places a wall 500 m in positive y from the example site at latitude -89.45 degrees, longitude 222.69 degrees. The peak among 720 bins must lie within 0.5 degrees of 137.31 degrees. The interpolated horizon there exceeds 10 degrees, while the former mirrored bearing 42.69 degrees and the opposite bearing 317.31 degrees remain negative.
7. interpolation_wraps checks one full turn, a negative azimuth, an angle beyond a full turn, and the empty-profile sentinel.
8. visibility_and_solar_disc checks half visibility at the center threshold, saturation on both sides, a binary margin test, and empty input.
9. weighted_stats_include_gap uses intervals of 1, 1, 4, and 1 hours. Two hours are sunny, two have Earth visibility, and one has both. The two consecutive dark intervals sum to a five-hour night. It also checks empty and singleton input.
10. stats_rejects_unordered_times expects a panic for descending timestamps.
11. horizons_sample_and_errors checks real-format sample angles and timestamps, optional flags, missing markers, empty bodies, malformed rows, and unavailable angles.

Floating-point formula checks use tolerances rather than assuming every algebraic rearrangement rounds identically. These tests validate synthetic terrain and parsing. They do not certify a real DEM's reference frame or scientific suitability.

## 7. prep/src/main.rs

### 7.1 Output wrapper and download helper

SiteData is a private output struct containing site, sun, earth, and an optional horizon. Serialize lets serde_json encode it. None becomes JSON null. The download helper borrows a reusable blocking Client and Site, along with string views for body, dates, and step. It returns either a Track or an owned error.

A URL identifies a network resource. Here the endpoint is `https://ssd.jpl.nasa.gov/api/horizons.api`. Query parameters are named values appended to a URL. reqwest's query method escapes them correctly; do not manually concatenate a raw URL containing spaces. Horizons expects quoted values for many parameters. The apostrophes inside the Rust strings are sent as data, not Rust delimiters.

The query parameters, one by one, are:

| Parameter | Value and purpose |
| :--- | :--- |
| format | text requests the text response parsed by lunar_core. |
| COMMAND | '10' selects the Sun; '399' selects Earth. These are Horizons body identifiers. |
| CENTER | 'coord@301' selects a coordinate-defined observing site on body 301, the Moon. |
| COORD_TYPE | 'GEODETIC' selects longitude, latitude, and height input rather than a Cartesian coordinate triple. |
| SITE_COORD | 'lon,lat,0' sends the site's longitude and latitude in degrees and zero height in the service's geodetic convention. The height field is in kilometers. It is not a height measured from our DEM. |
| MAKE_EPHEM | 'YES' requests an ephemeris table. |
| EPHEM_TYPE | 'OBSERVER' requests quantities from the observing site's viewpoint. |
| QUANTITIES | '4' selects apparent azimuth and elevation. Other quantity selections would require a different parser. |
| ANG_FORMAT | 'DEG' requests decimal degrees. |
| START_TIME | The requested initial calendar date, quoted for Horizons. |
| STOP_TIME | The final date. Horizons includes endpoint samples when the step lands there. |
| STEP_SIZE | A positive step such as '6h'. |

The helper sends the request, rejects HTTP error status codes, reads text, and passes it to parse_horizons. A service-level error can arrive with a successful HTTP status, so parsing is still necessary. Parse failures include the site, body, and response text in the returned diagnostic. The function does not retry or cache requests.

### 7.2 main and argument validation

std::env::args yields the executable name followed by command-line strings. skip(1) removes the executable name; collect owns the remaining strings. Four required arguments are sites file, start date, stop date, and step. A fifth argument overrides the output directory. Other argument counts return the usage error.

NaiveDate parsing validates both calendar dates. The stop must be strictly later than the start. The step must end in m, h, or d, and its preceding substring must parse as a positive u32. A malformed step returns an error before network access. Minutes, hours, and days are the only accepted units in this CLI.

fs::read_to_string reads the sites file, and serde_json::from_str constructs a Vec of Site. A Path represents a filesystem path without assuming UTF-8 path storage. The default output directory is app/assets/data relative to the current working directory. Run prep from the repository root. The client timeout is 120 seconds per request.

A HashSet records already-used slugs so output files cannot silently overwrite one another. slug splits a name at every non-ASCII-alphanumeric character, removes empty segments, lowercases, and joins segments with hyphens. The example becomes example-89-45s-222-69e. Empty slugs and duplicate slugs are rejected. Latitude must be finite and within [-90, 90]. Longitude must be finite; the CLI does not normalize it to a particular 360-degree interval.

create_dir_all creates the output directory if necessary. For each site, main downloads Sun and Earth tracks sequentially, requires equal timestamp vectors, constructs the filename, and serializes pretty JSON. The TODO marks where DEM loading and horizon_from_dem will go. Currently no DEM argument exists, and horizon is always None. The program prints the output path and sample count. It does not yet calculate Stats or combine power and communication constraints.

### 7.3 Actual generated data

The committed example was downloaded for 2026-12-01 through 2026-12-03 in 6-hour steps. It has nine timestamps per track, including both endpoints. This is an exact prefix of app/assets/data/example-89-45s-222-69e.json:

```json
{
  "site": {
	"name": "Example 89.45S 222.69E",
	"lat_deg": -89.45,
	"lon_deg": 222.69
  },
  "sun": {
	"t": [
	  1796083200,
	  1796104800,
	  1796126400,
	  1796148000,
	  1796169600,
	  1796191200,
	  1796212800,
	  1796234400,
	  1796256000
	],
	"az_deg": [
	  57.386894,
	  54.346596,
	  51.305817,
```

The excerpt stops inside an array and is not a standalone JSON document. The complete file also contains Sun elevations, Earth timestamps/angles, and `"horizon": null`. Its first Sun elevation is 1.766619 degrees. The first Earth elevation is -2.054944 degrees. Without terrain, these are not yet terrain-aware visibility decisions.

## 8. app: one UI, two environments

### 8.1 Library and immediate mode

A graphical user interface, or UI, displays controls and receives input. egui is the Rust immediate-mode UI library used here. eframe runs egui inside a native window or a browser canvas and handles graphics integration.

Immediate mode resembles Dear ImGui in C++. Application code describes controls during each UI pass, rather than building a permanent tree of button objects and attaching event handlers. egui retains internal interaction state, but the application supplies its current UI description again when asked. It does not require an endless busy loop in application code; eframe schedules work and repainting.

app/src/lib.rs defines a fieldless MoonApp with a default constructor generated by Default. Its implementation of eframe::App supplies the ui callback. The callback borrows the application exclusively, receives an exclusive UI reference, and receives a frame reference that is currently unused. CentralPanel::default constructs the main content panel. show takes a closure that adds a heading and the placeholder label. In eframe 0.37 the trait method is ui, not the update method seen in older tutorials.

### 8.2 Native entry

app/src/main.rs has two functions named main, but configuration ensures only one is compiled for a given target. The native function initializes logging, constructs default NativeOptions, and calls run_native with the window title Moon Landing Visual. Its constructor closure receives an unused creation context and returns an allocated MoonApp wrapped in Ok. A type alias is an alternative name for a type, comparable to a C++ using-alias. eframe::Result is the framework's convenient Result alias. The function returns run_native's final expression so startup errors reach the caller.

### 8.3 Browser entry

The wasm function schedules an asynchronous block with spawn_local. web_sys::window obtains the browser window optionally. document obtains its document optionally. The expect calls make the required page environment explicit. get_element_by_id finds the canvas by its HTML identifier. dyn_into checks/converts the generic element binding to an HtmlCanvasElement binding; this is a JavaScript object-type conversion, not a Rust numeric cast. The JsCast trait is imported through eframe's re-export of wasm-bindgen so that conversion method is available.

WebRunner starts rendering into the canvas. Its start operation is asynchronous, so the code awaits it. After successful startup the loading element is removed. On failure its text becomes a readable error and panic emits a diagnostic. None of this downloads the JSON yet. A future browser data-fetching layer must use browser-compatible asynchronous networking, not prep's blocking client.

### 8.4 index.html and Trunk.toml

HTML describes a document with nested elements. A tag opens or closes an element. Attributes provide properties such as id. The head contains metadata; the body contains displayed content. CSS is the stylesheet language controlling dimensions, colors, and positioning.

index.html declares English text and UTF-8 encoding. The viewport metadata makes browser layout follow device width. Its title names the browser tab. The canvas id matches main.rs exactly. The loading element is ordinary HTML, so it can appear before the Rust executable has loaded. noscript displays a notice when JavaScript is disabled. JavaScript is the browser's built-in scripting language; wasm needs generated JavaScript bindings for browser integration.

Trunk is a build tool and development server for Rust browser apps. The link with data-trunk and rel="rust" tells it to compile the Rust executable to wasm and generate loader bindings. The copy-dir link copies assets into its output. The base element lets Trunk insert the configured public URL. It does not start a backend server for our scientific computations. The CSS fills the page with the canvas and centers the loading notice.

Trunk.toml contains `[build]`, selecting build settings, and `filehash = false`, disabling content hashes in generated filenames. cargo check for the wasm target checks Rust compilation but does not verify Trunk packaging or browser rendering. trunk serve builds the browser bundle and serves it locally, normally at http://127.0.0.1:8080. Keep its terminal open and stop it with Ctrl+C.

## 9. Install, clone, build, test, and run

A terminal runs commands. The commands below assume Git is installed and the network permits downloads from Rust's package registry and JPL. Zed is an editor, not the compiler. Zed's Rust support is built in and downloads rust-analyzer automatically for navigation, diagnostics, and completion. There is no extension to install. Open the folder in Zed with `zed .` when its command-line launcher is installed, or use File > Open. Commands beginning with cargo are run at the repository root unless a cd changes that location. Cargo's `-p` selects a package; `--` separates Cargo's own arguments from arguments passed to an executable.

### 9.1 Windows with PowerShell

Install Git for Windows and Zed from their official installers. Download and run rustup-init.exe from https://rustup.rs/. Choose the default stable MSVC toolchain. When prompted, install the Visual Studio C++ Build Tools with the desktop C++ workload and Windows SDK. This is the native linker/toolchain requirement, not a dependency on OpenSSL. Restart PowerShell after installation so the tools are on PATH, the operating system's executable search path.

Run:

```powershell
rustup update stable
rustup target add wasm32-unknown-unknown
cargo install trunk
git clone https://github.com/Adriano4f/Moon-Landing-Visual.git
cd Moon-Landing-Visual
zed .
cargo build --workspace
cargo test -p lunar_core
cargo clippy --workspace
cargo run -p prep -- sites.json 2026-12-01 2026-12-03 6h
cargo run -p app
```

Close the native app before continuing in the same terminal. To check the browser compilation and serve it:

```powershell
cargo check -p app --target wasm32-unknown-unknown
cd app
trunk serve
```

Open http://127.0.0.1:8080 in a modern browser. The window or page should show only the heading and panorama placeholder. No OpenSSL or pkg-config setup is required for prep.

### 9.2 Ubuntu Linux

Linux distributions differ in package names. These exact system-package commands target Ubuntu. The C compiler and development headers support native Rust dependencies and graphics integration. pkg-config here supports discovery of Linux graphics libraries; it is not required by prep's Rustls HTTPS backend.

```sh
sudo apt-get update
sudo apt-get install -y curl git build-essential pkg-config libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev libgl1-mesa-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup update stable
rustup target add wasm32-unknown-unknown
cargo install trunk
git clone https://github.com/Adriano4f/Moon-Landing-Visual.git
cd Moon-Landing-Visual
```

Install Zed from https://zed.dev/ if it is not already available. Then run:

```sh
zed .
cargo build --workspace
cargo test -p lunar_core
cargo clippy --workspace
cargo run -p prep -- sites.json 2026-12-01 2026-12-03 6h
cargo run -p app
```

The native application needs a graphical desktop session with working graphics drivers. Close its window before continuing:

```sh
cargo check -p app --target wasm32-unknown-unknown
cd app
trunk serve
```

Open http://127.0.0.1:8080. The downloaded data remains a preparation artifact until the application implements data loading, plots, and the terrain panorama. Do not use this skeleton alone for mission planning.

## Code style: tabs and braces on their own line

`rustfmt.toml` sets `hard_tabs = true` (indent with tabs) and `brace_style` / `control_brace_style = "AlwaysNextLine"` (every `{` goes on its own line, like Allman style in C++). The two brace options only work in the nightly version of rustfmt, so install it once:

```sh
rustup toolchain install nightly --profile minimal --component rustfmt
```

Then format with `cargo +nightly fmt --all`. Plain `cargo fmt` (stable) ignores the brace options and prints a warning.
