# Real lighting and a freely movable camera

> The annotated shader below preserves the earlier lighting implementation.
> Main `5e4d5b7` additionally has a parallel Hapke observer, Realistic DEM shadows
> and 2048× navigation; see [19](19-far-field-photometry.md),
> [20](20-terrain-shadows.md) and [23](23-deep-zoom-windows.md). The dated inertial
> transform is explained in [17](17-inertial-rotation.md).

## 1. What changes with the date

The project compares south-polar landing sites and dates for solar power and direct Earth communication. The panorama remains the scientific local-horizon view. The globe now also uses the selected site's actual hourly Horizons directions for its sunlight and Earth light. Moving the UTC time slider moves the terminator, the boundary between lunar day and night. It also changes the Earth direction, exposing libration: the small apparent rocking of the Moon seen from Earth. There is no synthetic sixty-second spin by default.

This is a Moon-centered direction visualization, not a scale drawing of three bodies or a new orbit integrator. The existing Horizons samples encode the orbital evolution. Sun and Earth endpoint disks remain illustrative direction symbols near a landing marker, not the positions of those bodies in space. The scientific terrain availability and summary cards are unchanged.

Earth view also identifies the approximate phase and geometric illuminated area. Near new Moon, the Earth-facing surface is naturally dark, not covered by an eclipse object. The shader does not simulate eclipses. A visible area percentage is not a brightness estimate. Below 10 percent lit, Boost earthshine enables the existing 1,000 times display aid with one click; it never changes the numerical availability results. Phase classification is isolated in `globe/src/phase.rs`.

At 1× the perspective camera sits five lunar radii from the center, instead of the earlier 3.3. Current navigation ranges from 1× to 2048×, with distance D=1+4/zoom; View from Earth restores 1×. CPU picking and GPU geometry share that distance/projection, while Hapke uses a separate parallel observer to prevent the moving opposition hotspot. At the 1× distance the finite visible cap can hide a crescent with at most about 1.010 percent geometrically illuminated area; that number is not a constant for every zoom.

## 2. One coordinate frame for bodies, lights and markers

The renderer's Moon-fixed axes are +y at the north pole, +z at latitude zero and longitude zero, and +x at latitude zero and 90 degrees east. The previous guide derives the local east, north and up vectors. For sky azimuth A measured north toward east and elevation e, the direction is d=cos(e)sin(A)E+cos(e)cos(A)N+sin(e)U. The public Rust function `moon_fixed` wraps that tested conversion with degree-valued inputs. Its return type `Option<[f32; 3]>` is like a C++ optional containing an array of three floats; invalid angles return no direction rather than a fabricated light.

The selected site's two current directions become a copied `Lighting` snapshot captured by the deferred GL callback. Local availability flags do not switch off global sunlight: a mountain hiding the Sun from one observer does not turn off the Sun for the rest of the Moon. If the selected data is missing or invalid, the app explicitly reports that real illumination is unavailable and supplies zero physical irradiance, not the former fixed presentation light.

Both actual light directions are transformed by `u_model`, exactly like lunar geometry and mapped normals. Therefore dragging the camera rotates the view, not the Sun relative to a fixed lunar feature. A time change changes that physical relationship. Realistic's `light_response` evaluates Hapke separately for Sun and Earth; direct sunlight additionally uses a DEM visibility mask. The geometric-night guard remains for unresolved paths, not a blanket cutoff for resolved elevated terrain. Fast uses a cheap geometric response instead. Contributions add before tone mapping and the single sRGB encoding step. Rays retain their unlit diagram colors.

## 3. Camera modes from zero

Dragging changes yaw and tilt and leaves Earth-following mode. Wheel, pinch and plus/minus controls now clamp zoom to 1 through 2048. These remain camera controls, not changes to the observing site's coordinates. Higher zoom can request bounded WAC/NAC imagery or DEM detail; it does not manufacture information beyond source resolution. See chapter 23 for the shared camera and visible-window selection.

The default is Earth view. For the current Moon-fixed Earth unit vector E=(x,y,z), choose yaw=-atan2(x,z) and tilt=asin(y). Substitution into the shared Rx(tilt)Ry(yaw) matrix rotates E onto +z, the camera axis. Earth view keeps recomputing these two angles when time changes, so libration is visible. The View from Earth button returns to this mode and resets zoom to the full-disk overview. Free orbit keeps its chosen angles when time changes, making the moving terminator easy to compare.

Go to site leaves Earth view and retains the documented 0.8-second shortest-yaw, eased, logarithmic-zoom transition to the selected site's radial vector. The Auto-rotate checkbox is an optional presentation spin, off by default; enabling it leaves Earth view. This artificial spin is not an orbital period. The UI names the active camera mode.

View from Earth means the dated Earth line of sight, not a camera literally at Earth distance or an Earth-sized angular image. The inspection camera is at five lunar radii at 1× and moves toward the surface with zoom. Moon-fixed Sun lighting is independent of that presentation distance; the far-field photometric approximation is explained in chapter 19.

## 4. Topocentric versus Moon-center direction

Horizons directions are measured at the selected observer, not at the lunar center. Across a lunar radius, Earth parallax is approximately asin(1737.4/384400)=0.259 degree. Sun parallax is approximately 0.00067 degree. We accept these small offsets for lighting and camera direction; do not silently describe them as center-corrected ephemerides. Terrain statistics continue to use the original topocentric samples without this approximation.

An independent ENU sweep of the bundled C1 series gives subsolar latitude -1.587370 through +1.559565 degrees, and sub-Earth longitude -8.043021 through +8.053838 degrees and latitude -6.609914 through +7.095109 degrees. The small latitude offset beyond seven degrees is consistent with using the south-polar observer rather than the center. Regression tests transform every hourly sample through the production public conversion, bound these physical ranges, and require substantial changes over time so a constant placeholder direction cannot pass.

## 5. How much light comes from Earth

Earthshine is sunlight reflected by Earth onto the Moon. It is not blue radio radiation and it does not establish a communication link. [NASA's Earth fact sheet](https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html) gives geometric albedo 0.434, mean radius 6,371 km and solar irradiance 1,361 W per square metre. [NASA's Moon fact sheet](https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html) gives mean orbital semimajor axis 384,400 km. These are the source parameters for our visible-light estimate, not a claimed NASA measurement of this app's rendered brightness.

Geometric albedo compares full-phase disk brightness with a reference reflecting disk. It is the appropriate albedo here, not the total-energy Bond albedo. A disk's apparent area scales as radius squared divided by distance squared. For full Earth the incident irradiance relative to sunlight is therefore:

$$q_0=0.434\left(\frac{6371}{384400}\right)^2=0.000119217.$$

This corresponds to about 0.162 W per square metre when multiplied by 1,361, or about one part in 8,388 of direct sunlight. Treat this as a visible-light rendering estimate, not a bolometric thermal model.

Let psi=acos(S dot E) be the lunar phase angle at the Moon. To the distant-Sun approximation, Earth's phase angle is its supplement alpha=pi-psi: full Earth accompanies new Moon as seen from Earth. Model Earth's unresolved disk as a Lambert sphere. Integrating incidence cosine over its illuminated visible hemisphere gives the normalized phase function:

$$\Phi(\alpha)=\frac{\sin\alpha+(\pi-\alpha)\cos\alpha}{\pi},\qquad q=q_0\Phi(\alpha).$$

At full Earth Phi=1; at quarter Earth Phi=1/pi; at new Earth Phi=0. The numeric function normalizes the direction dot product, clamps roundoff before acos, and rejects zero or nonfinite vectors. Tests cover all three phases and malformed inputs. The shader adds q times the Earth-direction Hapke response to the direct Sun response. Both use the same solar-relative units.

This is phase-dependent, not weather-dependent. Earth is not a perfect Lambert sphere; clouds, oceans and continents change its true phase curve and color. We use mean distance, so actual distance-driven brightness changes are not modeled. NASA lists actual lunar ranges around 357,000 through 407,000 km. Eclipses, Earth cloud maps and thermal emission remain absent. Realistic now has finite-sample DEM terrain shadows, described in chapter 20; Fast retains recorded photographic shadows. These visual models do not alter the independently computed terrain-horizon data.

## 6. Earthshine boost and the historical far-side aid

Earthshine boosted (1000x) multiplies only the reflected Earth-light contribution by 1,000. It is an exposure aid and is off by default. Sunlight, the scientific solar fraction, radio clearance, ray visibility and statistics are not multiplied. The word boosted and the factor remain visible beside the checkbox.

The original renderer also had a nonphysical far-side fill: a gray-blue contribution following mean far-side longitude rather than the instantaneous Earth vector. Its implementation and `u_far_fill` appear in this chapter's historical listing/API map. They are not part of main `5e4d5b7`'s Fast/Realistic shader paths. Do not confuse that removed presentation aid with the still-supported physical Earthshine approximation or its explicit exposure boost.

## 7. Rust, GLSL and C++ API map

The [consolidated callable index](09-sites-and-globe-controls.md#20-checks-and-api-index) includes every callable in the changed Rust modules and is regenerated with their exact annotated source quotations. The following map highlights this guide's boundary:

| API or value | Job | C++ equivalent |
|:--|:--|:--|
| `moon_fixed` | Validated degree-valued sky to Moon-fixed unit vector | Function returning optional array |
| `earthshine_ratio` | Phase and geometric-albedo irradiance ratio | Pure float-valued helper |
| `Lighting` | Copy the selected epoch and explicit display aids | Small copyable aggregate |
| `Globe::show` | Earth lock, free orbit, go-to and optional spin | Immediate-mode member function |
| `Renderer::paint` | Upload physical direction and intensity uniforms | GL calls on the current context |
| `light_response` | One masked Hapke response per light | GLSL float function |
| `u_sun`, `u_earth` | Moon-fixed directions | `glUniform3f` inputs |
| `u_sun_scale`, `u_earthshine`, `u_far_fill` | Separate physical and visual scales | `glUniform1f` inputs |

## 8. Exact shader quotation

### 8.1. globe/src/moon.frag

```glsl
// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
in vec3 v_position;
// Pass unrotated object position so texture coordinates remain attached to the Moon.
in vec3 v_object;
// Bind the selected tier's SRGB8_ALPHA8 color texture through its texture-unit index.
uniform sampler2D u_albedo;
// Bind the selected tier's linear RGB8 object-space normal texture.
uniform sampler2D u_normal;
uniform sampler2D u_polar_color;
uniform sampler2D u_polar_normal;
uniform float u_polar_ready;
uniform float u_polar_extent;
// Receive the same rigid model matrix as the vertex shader and CPU marker; C++ glUniformMatrix3fv.
uniform mat3 u_model;
// Match the vertex shader's camera position and uniform precision.
uniform highp float u_camera_distance;
// Write opaque sRGB display color to egui's framebuffer.
out vec4 out_color;
// A uniform branch reuses the exact globe vertex transform for unlit direction lines; C++ integer material flag.
uniform highp int u_ray_mode;
// Interpolated signed-distance coordinates, independent of clip-space offsets.
in vec2 v_coverage;
// Direction-line colors are display colors, independent of illustrative illumination; C++ RGB uniform.
uniform vec3 u_ray_color;
// Receive the actual Moon-fixed Sun direction; C++ glUniform3f.
uniform vec3 u_sun;
// Receive the actual Moon-fixed Earth direction; C++ glUniform3f.
uniform vec3 u_earth;
// Disable physical sunlight when ephemerides are unavailable; C++ glUniform1f.
uniform float u_sun_scale;
// Scale reflected Earth light relative to solar irradiance; C++ glUniform1f.
uniform float u_earthshine;
// Enable a nonphysical visual aid independently of physical energy; C++ glUniform1f.
uniform float u_far_fill;
// Define pi once so all angular expressions use radians consistently.
const float PI = 3.141592653589793;
// Project mapped normals into the view-facing half-space before renormalizing; dimensionless cosine.
const float MIN_EMISSION_COSINE = 0.01;
// Display exposure is eight times relative linear radiance, one stop above the previous value of four.
const float EXPOSURE = 8.0;
// Area-weighted 566 nm WAC map means, not a new fit. See assets/photometry.py.
// ISIS3 fadd004da2f733525209ee4f66ffdb3a9a9d193e:
// isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp, HGfunc and photometry.
// Set the area-weighted single-scattering albedo: the fraction of energy one grain scattering retains.
const float W = 0.3261142672;
// Set the HG lobe shape coefficient from the 566 nm parameter map, not the opposition amplitude.
const float B = 0.2348928321;
// Set the signed HG mixing coefficient; positive C favors backward scattering with weight (1+C)/2.
const float C = 0.3574639778;
// Set the amplitude of shadow-hiding opposition brightening from the same map.
const float BS0 = 1.7707997700;
// Set its dimensionless tan(phase/2) width parameter, not a width measured directly in degrees.
const float HS = 0.0622193166;
// Convert the map's mean unresolved roughness angle from degrees to radians.
const float THETA = 23.65660095 * PI / 180.0;

// Approximate Chandrasekhar multiple scattering with the logarithmic Hapke/ISIS expression.
float h_function(float x)
{
	// Use the continuous H(0)=1 limit to numerical tolerance without evaluating log(1/0).
	x = max(x, 0.000001);
	// Compute the square root of the absorbed fraction used by the H-function approximation.
	float gamma = sqrt(1.0 - W);
	// Compute Hapke's diffuse reflectance parameter from single-scattering albedo.
	float r0 = (1.0 - gamma) / (1.0 + gamma);
	// Evaluate the logarithmic H approximation as a reciprocal, not the simpler rational approximation.
	return 1.0 / (1.0 - W * x * (r0 + 0.5 * (1.0 - 2.0 * r0 * x) * log((1.0 + x) / x)));
}

// Return Hapke I/F, a dimensionless radiance factor rather than a conventional BRDF per steradian.
float hapke(float mu0, float mu, float cosg)
{
	// Return black for a locally unlit or invisible surface instead of dividing at the terminator.
	if (mu0 <= 0.0 || mu <= 0.0)
	{
		// Emit zero reflected sunlight; no ambient or earthshine is added.
		return 0.0;
	}
	// Convert the clamped light-view dot product into phase angle in radians.
	float g = acos(clamp(cosg, -1.0, 1.0));
	// Reuse the squared HG shape parameter in both lobe denominators.
	float b2 = B * B;
	// Weight the backward lobe by (1+C)/2; the continuation adds the forward lobe.
	float phase = (1.0 + C) * 0.5 * (1.0 - b2) / pow(1.0 - 2.0 * B * cosg + b2, 1.5)
		// Add the forward HG lobe with weight (1-C)/2 and the opposite sign in its denominator.
		+ (1.0 - C) * 0.5 * (1.0 - b2) / pow(1.0 + 2.0 * B * cosg + b2, 1.5);
	// Compute the shadow-hiding profile, which equals one exactly at opposition.
	float shoe = 1.0 / (1.0 + tan(g * 0.5) / HS);
	// Published Bc0=0 and phi=0 imply no CBOE multiplier and K=1.
	// Compute tangent of the unresolved mean surface tilt for the roughness correction.
	float t = tan(THETA);
	// Compute the roughness normalization 1/sqrt(1+pi*tan(theta)^2).
	float chi = inversesqrt(1.0 + PI * t * t);
	// Recover sine of incidence from its cosine, clamping roundoff before sqrt.
	float sini = sqrt(max(0.0, 1.0 - mu0 * mu0));
	// Recover sine of emission from its cosine, clamping roundoff before sqrt.
	float sine = sqrt(max(0.0, 1.0 - mu * mu));
	// Compute cotangent of incidence with a finite normal-incidence limit.
	float coti = mu0 / max(sini, 0.000001);
	// Compute cotangent of emission with a finite normal-emission limit.
	float cote = mu / max(sine, 0.000001);
	// Evaluate the first roughness exponential at incidence.
	float e1i = exp(-2.0 * coti / (PI * t));
	// Evaluate the second roughness exponential at incidence.
	float e2i = exp(-coti * coti / (PI * t * t));
	// Evaluate the first roughness exponential at emission.
	float e1e = exp(-2.0 * cote / (PI * t));
	// Evaluate the second roughness exponential at emission.
	float e2e = exp(-cote * cote / (PI * t * t));
	// Compute effective incidence cosine for zero relative azimuth.
	float etai = chi * (mu0 + sini * t * e2i / (2.0 - e1i));
	// Compute effective emission cosine for zero relative azimuth.
	float etae = chi * (mu + sine * t * e2e / (2.0 - e1e));
	// Recover the azimuth between incidence and emission planes; use zero azimuth at the normal singularity.
	float cospsi = sini * sine < 0.000001 ? 1.0 : clamp((cosg - mu * mu0) / (sini * sine), -1.0, 1.0);
	// Convert that bounded cosine to a relative azimuth in radians.
	float psi = acos(cospsi);
	// Compute the azimuth-dependent overlap profile, taking its exact zero limit at pi.
	float fpsi = psi >= PI - 0.000001 ? 0.0 : exp(-2.0 * tan(psi * 0.5));
	// Use the half-angle identity to obtain sin(psi/2)^2 without another trigonometric call.
	float sin2 = (1.0 - cospsi) * 0.5;
	// Hold the roughness-adjusted incidence cosine, assigned in both geometric branches.
	float mu0e;
	// Hold the roughness-adjusted emission cosine, assigned in both geometric branches.
	float mue;
	// Hold the branch-dependent ratio used to normalize roughness shadowing.
	float q;
	// Select the i<=e roughness branch because acos is decreasing on the valid cosine interval.
	if (mu0 >= mu)
	{
		// Compute the current incidence/emission branch's common roughness denominator.
		float denominator = 2.0 - e1e - psi / PI * e1i;
		// Assign the branch's adjusted incidence cosine from its roughness exponentials.
		mu0e = chi * (mu0 + sini * t * (cospsi * e2e + sin2 * e2i) / denominator);
		// Assign the branch's adjusted emission cosine from its roughness exponentials.
		mue = chi * (mu + sine * t * (e2e - sin2 * e2i) / denominator);
		// Assign the matching incidence or emission normalization ratio for shadowing.
		q = mu0 / etai;
	}
	// Use the complementary i>e roughness branch, exchanging the appropriate incidence/emission roles.
	else
	{
		// Compute the current incidence/emission branch's common roughness denominator.
		float denominator = 2.0 - e1i - psi / PI * e1e;
		// Assign the branch's adjusted incidence cosine from its roughness exponentials.
		mu0e = chi * (mu0 + sini * t * (e2i - sin2 * e2e) / denominator);
		// Assign the branch's adjusted emission cosine from its roughness exponentials.
		mue = chi * (mu + sine * t * (cospsi * e2i + sin2 * e2e) / denominator);
		// Assign the matching incidence or emission normalization ratio for shadowing.
		q = mu / etae;
	}
	// Assemble the unresolved shadowing factor; guard the grazing-angle denominator against zero.
	float shadow = mue * mu0 * chi / max(etae * etai * (1.0 - fpsi + fpsi * chi * q), 0.000001);
	// Add isotropic multiple scattering as the product of two H-functions minus one.
	float multiple = h_function(mu0e) * h_function(mue) - 1.0;
	// Multiply single scattering, opposition, multiple scattering, and roughness into I/F; K=1 and Bc0=0.
	return W * 0.25 * mu0e / max(mu0e + mue, 0.000001) * (phase * (1.0 + BS0 * shoe) + multiple) * shadow;
}

// Encode linear display intensities with the standard piecewise sRGB transfer function.
vec3 linear_to_srgb(vec3 x)
{
	// Select the low-intensity linear branch or the power-law branch independently for each color channel.
	return mix(x * 12.92, 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), x));
}

// Evaluate one directional light without night-side normal-map leakage; C++ float-valued helper.
float light_response(vec3 n, vec3 geometric, vec3 view, vec3 light, float normal_strength)
{
	// Local terrain normals cannot substitute for resolved shadows; C++ geometric visibility guard.
	if (dot(geometric, light) <= 0.0)
	{
		// Suppress this source on its geometric night side; C++ early return.
		return 0.0;
	}
	// Evaluate the mapped response with the shared photometric model; C++ float initialization.
	float mapped_reflectance = hapke(dot(n, light), dot(n, view), dot(light, view));
	// Evaluate the smooth-sphere response independently of texture normals; C++ float initialization.
	float geometric_reflectance = hapke(dot(geometric, light), dot(geometric, view), dot(light, view));
	// Fade unresolved relief at grazing emission angles; C++ linear-interpolation return.
	return mix(geometric_reflectance, mapped_reflectance, normal_strength);
}

// Run once per vertex or fragment respectively; GLSL entry points have no C++ object ownership.
void main()
{
	// All fragments in this draw take the same branch, before texture derivatives or Hapke evaluation; C++ uniform material switch.
	if (u_ray_mode != 0)
	{
		// Preserve opaque diagram colors; dimming is encoded in RGB, not photographic lighting.
		out_color = vec4(u_ray_color, 1.0 - smoothstep(0.15, 1.0, length(v_coverage)));
		// Rays require no albedo, normals or photometric calculations; C++ early return.
		return;
	}
	// Normalize the interpolated object-space direction before reconstructing longitude and latitude.
	vec3 p = normalize(v_object);
	// Map atan longitude and acos latitude into normalized equirectangular coordinates.
	vec2 uv = vec2(atan(p.x, p.z) / (2.0 * PI) + 0.5, acos(clamp(p.y, -1.0, 1.0)) / PI);
	// Differentiate the continuous direction, not the wrapped atan output.
	// Differentiate the continuous direction across adjacent screen fragments horizontally.
	vec3 dx = dFdx(p);
	// Differentiate the continuous direction across adjacent screen fragments vertically.
	vec3 dy = dFdy(p);
	// Bound the squared horizontal radius so analytic UV derivatives remain finite at poles.
	float horizontal = max(dot(p.xz, p.xz), 0.000001);
	// Apply the analytic longitude/latitude derivative to horizontal screen variation, avoiding the wrap jump.
	vec2 gx = vec2((p.z * dx.x - p.x * dx.z) / (2.0 * PI * horizontal), -dx.y / (PI * sqrt(horizontal)));
	// Apply the same continuous derivative to vertical screen variation for texture footprint estimation.
	vec2 gy = vec2((p.z * dy.x - p.x * dy.z) / (2.0 * PI * horizontal), -dy.y / (PI * sqrt(horizontal)));
	// Decode the RGB8 object-space normal and normalize after filtering.
	// Texture state retains trilinear mipmaps but not the anisotropic path that produced dotted arcs in fixed-pose tests.
	vec3 n = normalize(textureGrad(u_normal, uv, gx, gy).rgb * 2.0 - 1.0);
	// South stereographic: x points east at longitude zero, y points toward longitude zero.
	vec2 polar_uv = 0.5 + (1737400.0 / u_polar_extent) * vec2(p.x, -p.z) / max(1.0 - p.y, 0.000001);
	float polar_blend = u_polar_ready * (1.0 - smoothstep(sin(-77.0 * PI / 180.0), sin(-75.0 * PI / 180.0), p.y));
	if (polar_blend > 0.0)
	{
		n = normalize(mix(n, texture(u_polar_normal, polar_uv).rgb * 2.0 - 1.0, polar_blend));
	}
	// Apply the camera's rigid rotation to the normal, preserving lighting-space consistency.
	n = normalize(u_model * n);
	// Transform real Moon-fixed lights with the same rigid matrix as the Moon and markers.
	vec3 light = normalize(u_model * u_sun);
	// Keep the Earth source in the same world frame; C++ matrix-vector product and normalization.
	vec3 earth_light = normalize(u_model * u_earth);
	// Point from the surface fragment to the shared perspective camera.
	vec3 view = normalize(vec3(0.0, 0.0, u_camera_distance) - v_position);
	// Fade unresolved relief in the foreshortened limb band; use the true sphere normal below cosine 0.35.
	vec3 geometric = normalize(v_position);
	// Retain full relief above cosine 0.65, with a continuous smoothstep transition between both angles.
	float normal_strength = smoothstep(0.35, 0.65, max(dot(geometric, view), 0.0));
	// Suppress unresolved equirectangular polar detail, rather than displaying longitude streaks as terrain.
	float polar_fallback = smoothstep(cos(15.0 * PI / 180.0), cos(10.0 * PI / 180.0), abs(p.y));
	polar_fallback *= 1.0 - polar_blend;
	// Within ten degrees of either pole the overview deliberately uses a schematic sphere normal.
	normal_strength *= 1.0 - polar_fallback;
	// A normal map cannot change silhouette visibility: back-facing mapped facets must not cut black holes.
	// Add only the missing view component, then restore unit length; leave already front-facing normals alone.
	n = normalize(n + max(0.0, MIN_EMISSION_COSINE - dot(n, view)) * view);
	// Add physically distinct direct solar and phase-scaled reflected Earth contributions.
	float reflectance = u_sun_scale * light_response(n, geometric, view, light, normal_strength)
		+ u_earthshine * light_response(n, geometric, view, earth_light, normal_strength);
	// SRGB8_ALPHA8 texture sampling already decodes color and filters in linear space.
	// Sample SRGB8_ALPHA8 color with explicit seam-safe derivatives; the GPU returns linear color.
	vec3 albedo = textureGrad(u_albedo, uv, gx, gy).rgb;
	// Use the documented mean reflectance in the unresolved cap; this is not a new terrain or color measurement.
	albedo = mix(albedo, vec3(0.4832), polar_fallback);
	if (polar_blend > 0.0)
	{
		// Recorded photographic shadows are bounded visual contrast, never measured current illumination or albedo.
		vec3 mosaic = texture(u_polar_color, polar_uv).rgb;
		float valid = step(0.0001, dot(mosaic, vec3(1.0)));
		albedo *= mix(vec3(1.0), clamp(mosaic / 0.2, vec3(0.65), vec3(1.35)), 0.25 * polar_blend * valid);
	}
	// Solid-angle-weighted mean linear luminance; this is not a per-pixel inversion.
	// Modulate the averaged photometric response by relative spatial color and apply exposure.
	vec3 radiance = EXPOSURE * (albedo / 0.4832) * reflectance;
	// Only longitudes beyond +/-90 degrees from mean Earth longitude zero receive this visual aid.
	float far_side = smoothstep(0.0, 0.15, -p.z / max(length(p.xz), 0.000001));
	// This soft gray-blue term is deliberately not labeled physical illumination.
	radiance += u_far_fill * far_side * vec3(0.035, 0.045, 0.065) * (albedo / 0.4832);
	// Compress positive linear radiance with the Reinhard operator before display encoding.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	// Encode once to sRGB and set alpha to one; egui_glow disables framebuffer sRGB encoding.
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
```
