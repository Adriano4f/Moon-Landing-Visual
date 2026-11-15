// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
in vec3 v_position;
// Pass unrotated object position so texture coordinates remain attached to the Moon.
in vec3 v_object;
in vec2 v_screen;
uniform mat4 u_projection;
uniform highp sampler2DArray u_deep_color;
uniform sampler2D u_deep_pages;
uniform highp int u_deep_ready;
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
// Resolve the finest resident ancestor in constant time; empty pages keep the overview.
vec4 satellite(vec2 uv, int pole)
{
	bool polar = pole != 0;
	vec2 extent = polar ? vec2(32.0) : vec2(512.0, 256.0);
	uv = vec2(polar ? clamp(uv.x, 0.0, 0.999999) : fract(uv.x), clamp(uv.y, 0.0, 0.999999));
	ivec2 cell = ivec2(floor(uv * extent)) + ivec2(pole < 0 ? 512 : (pole > 0 ? 544 : 0), 0);
	vec2 page = floor(texelFetch(u_deep_pages, cell, 0).rg * 255.0 + 0.5);
	if (page.x < 1.0 || u_deep_ready == 0) return vec4(0.0);
	float rows = exp2(page.y);
	vec2 tiles = vec2(polar ? rows : 2.0 * rows, rows);
	vec2 local = clamp(fract(uv * tiles), vec2(0.5 / 256.0), vec2(255.5 / 256.0));
	return vec4(texture(u_deep_color, vec3(local, page.x - 1.0)).rgb, 1.0);
}

// Analytic sphere intersection avoids mesh facets and remains stable near the surface.
bool surface_point(out vec3 position, out vec3 object)
{
	vec3 ray = normalize(vec3(v_screen.x / u_projection[0][0], v_screen.y / u_projection[1][1], -1.0));
	float b = -u_camera_distance * ray.z;
	float c = (u_camera_distance - 1.0) * (u_camera_distance + 1.0);
	float discriminant = b * b - c;
	if (discriminant < 0.0) return false;
	float distance = c / (b + sqrt(discriminant));
	position = vec3(ray.xy * distance, u_camera_distance + ray.z * distance);
	object = normalize(transpose(u_model) * position);
	vec4 clip = u_projection * vec4(ray * distance, 1.0);
	gl_FragDepth = 0.5 + 0.5 * clip.z / clip.w;
	return gl_FragDepth >= 0.0 && gl_FragDepth <= 1.0;
}

void main()
{
	// All fragments in this draw take the same branch, before texture derivatives or Hapke evaluation; C++ uniform material switch.
	if (u_ray_mode != 0)
	{
		gl_FragDepth = gl_FragCoord.z;
		// Preserve opaque diagram colors; dimming is encoded in RGB, not photographic lighting.
		out_color = vec4(u_ray_color, 1.0 - smoothstep(0.15, 1.0, length(v_coverage)));
		// Rays require no albedo, normals or photometric calculations; C++ early return.
		return;
	}
	// Normalize the interpolated object-space direction before reconstructing longitude and latitude.
	vec3 position;
	vec3 p;
	if (!surface_point(position, p)) discard;
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
	vec3 view = normalize(vec3(0.0, 0.0, u_camera_distance) - position);
	// Fade unresolved relief in the foreshortened limb band; use the true sphere normal below cosine 0.35.
	vec3 geometric = normalize(position);
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
	float cap = 1.0 - smoothstep(sin(75.0 * PI / 180.0), sin(77.0 * PI / 180.0), abs(p.y));
	vec4 global_detail = satellite(uv, 0);
	float global_weight = global_detail.a * cap;
	vec3 satellite_factor = mix(vec3(1.0), clamp(global_detail.rgb / 0.2, vec3(0.2), vec3(2.0)), global_weight);
	float satellite_weight = global_weight;
	if (cap < 1.0)
	{
		int pole = p.y < 0.0 ? -1 : 1;
		vec2 polar_detail_uv = 0.5 + (1737400.0 / 1095930.0) * vec2(p.x, p.y < 0.0 ? -p.z : p.z) / max(1.0 + abs(p.y), 0.000001);
		vec4 polar_detail = satellite(polar_detail_uv, pole);
		float weight = polar_detail.a * (1.0 - cap);
		satellite_factor = mix(satellite_factor, clamp(polar_detail.rgb / 0.2, vec3(0.2), vec3(2.0)), weight);
		satellite_weight = max(satellite_weight, weight);
	}
	if (polar_blend > 0.0)
	{
		// Recorded photographic shadows are bounded visual contrast, never measured current illumination or albedo.
		vec3 mosaic = texture(u_polar_color, polar_uv).rgb;
		float valid = step(0.0001, dot(mosaic, vec3(1.0)));
		albedo *= mix(vec3(1.0), clamp(mosaic / 0.2, vec3(0.65), vec3(1.35)), 0.25 * polar_blend * valid * (1.0 - satellite_weight));
	}
	// Solid-angle-weighted mean linear luminance; this is not a per-pixel inversion.
	// Modulate the averaged photometric response by relative spatial color and apply exposure.
	albedo *= mix(vec3(1.0), satellite_factor, 0.85);
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
