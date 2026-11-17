precision highp float;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
in vec3 v_position;
// Unrotated object coordinates keep textures attached to the Moon.
in vec3 v_object;
in vec2 v_screen;
uniform mat4 u_projection;
uniform highp sampler2DArray u_deep_color;
uniform sampler2D u_deep_pages;
uniform highp int u_deep_ready;
// SRGB8_ALPHA8 albedo is decoded to linear before filtering.
uniform sampler2D u_albedo;
// Object-space normals use linear RGB8.
uniform sampler2D u_normal;
uniform sampler2D u_polar_color;
uniform sampler2D u_polar_normal;
uniform highp sampler2D u_height;
uniform highp sampler2D u_polar_height;
uniform highp int u_height_ready;
uniform highp int u_live_shadows;
uniform highp int u_shadow_steps;
uniform sampler2D u_shadow_mask;
uniform highp int u_shadow_pass;
uniform highp int u_mask_ready;
uniform vec4 u_shadow_viewport;
uniform float u_polar_ready;
uniform float u_polar_normal_ready;
uniform float u_polar_extent;
uniform mat3 u_model;
uniform highp float u_camera_distance;
// Write opaque sRGB display color to egui's framebuffer.
out vec4 out_color;
uniform highp int u_ray_mode;
in vec2 v_coverage;
uniform vec3 u_ray_color;
uniform vec3 u_sun;
uniform vec3 u_earth;
uniform float u_sun_scale;
// Scale reflected Earth light relative to solar irradiance.
uniform float u_earthshine;
const float PI = 3.141592653589793;
// Project mapped normals into the view-facing half-space before renormalizing; dimensionless cosine.
const float MIN_EMISSION_COSINE = 0.01;
// Display exposure scales relative linear radiance by eight.
const float EXPOSURE = 8.0;
// Area-weighted 566 nm WAC map means, not a new fit. See assets/photometry.py.
// ISIS3 fadd004da2f733525209ee4f66ffdb3a9a9d193e:
// isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp, HGfunc and photometry.
const float W = 0.3261142672;
// Set the HG lobe shape coefficient from the 566 nm parameter map, not the opposition amplitude.
const float B = 0.2348928321;
// Set the signed HG mixing coefficient; positive C favors backward scattering with weight (1+C)/2.
const float C = 0.3574639778;
// Shadow-hiding opposition amplitude.
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
	float gamma = sqrt(1.0 - W);
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
		return 0.0;
	}
	float g = acos(clamp(cosg, -1.0, 1.0));
	float b2 = B * B;
	// Weight the backward lobe by (1+C)/2; the continuation adds the forward lobe.
	float phase = (1.0 + C) * 0.5 * (1.0 - b2) / pow(1.0 - 2.0 * B * cosg + b2, 1.5)
		// Add the forward HG lobe with weight (1-C)/2 and the opposite sign in its denominator.
		+ (1.0 - C) * 0.5 * (1.0 - b2) / pow(1.0 + 2.0 * B * cosg + b2, 1.5);
	float shoe = 1.0 / (1.0 + tan(g * 0.5) / HS);
	// Published Bc0=0 and phi=0 imply no CBOE multiplier and K=1.
	// Compute tangent of the unresolved mean surface tilt for the roughness correction.
	float t = tan(THETA);
	float chi = inversesqrt(1.0 + PI * t * t);
	float sini = sqrt(max(0.0, 1.0 - mu0 * mu0));
	float sine = sqrt(max(0.0, 1.0 - mu * mu));
	// Compute cotangent of incidence with a finite normal-incidence limit.
	float coti = mu0 / max(sini, 0.000001);
	// Compute cotangent of emission with a finite normal-emission limit.
	float cote = mu / max(sine, 0.000001);
	float e1i = exp(-2.0 * coti / (PI * t));
	float e2i = exp(-coti * coti / (PI * t * t));
	float e1e = exp(-2.0 * cote / (PI * t));
	float e2e = exp(-cote * cote / (PI * t * t));
	float etai = chi * (mu0 + sini * t * e2i / (2.0 - e1i));
	float etae = chi * (mu + sine * t * e2e / (2.0 - e1e));
	// Recover the azimuth between incidence and emission planes; use zero azimuth at the normal singularity.
	float cospsi = sini * sine < 0.000001 ? 1.0 : clamp((cosg - mu * mu0) / (sini * sine), -1.0, 1.0);
	float psi = acos(cospsi);
	// Compute the azimuth-dependent overlap profile, taking its exact zero limit at pi.
	float fpsi = psi >= PI - 0.000001 ? 0.0 : exp(-2.0 * tan(psi * 0.5));
	// Use the half-angle identity to obtain sin(psi/2)^2 without another trigonometric call.
	float sin2 = (1.0 - cospsi) * 0.5;
	float mu0e;
	float mue;
	float q;
	// Select the i<=e roughness branch because acos is decreasing on the valid cosine interval.
	if (mu0 >= mu)
	{
		float denominator = 2.0 - e1e - psi / PI * e1i;
		mu0e = chi * (mu0 + sini * t * (cospsi * e2e + sin2 * e2i) / denominator);
		mue = chi * (mu + sine * t * (e2e - sin2 * e2i) / denominator);
		q = mu0 / etai;
	}
	// Use the complementary i>e roughness branch, exchanging the appropriate incidence/emission roles.
	else
	{
		float denominator = 2.0 - e1i - psi / PI * e1e;
		mu0e = chi * (mu0 + sini * t * (e2i - sin2 * e2e) / denominator);
		mue = chi * (mu + sine * t * (cospsi * e2i + sin2 * e2e) / denominator);
		q = mu / etae;
	}
	float shadow = mue * mu0 * chi / max(etae * etai * (1.0 - fpsi + fpsi * chi * q), 0.000001);
	float multiple = h_function(mu0e) * h_function(mue) - 1.0;
	// Multiply single scattering, opposition, multiple scattering, and roughness into I/F; K=1 and Bc0=0.
	return W * 0.25 * mu0e / max(mu0e + mue, 0.000001) * (phase * (1.0 + BS0 * shoe) + multiple) * shadow;
}

// Encode linear display intensities with the standard piecewise sRGB transfer function.
vec3 linear_to_srgb(vec3 x)
{
	return mix(x * 12.92, 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), x));
}

// Evaluate one directional light without night-side normal-map leakage.
float light_response(vec3 n, vec3 geometric, vec3 view, vec3 light, float normal_strength, bool resolved)
{
	// Local terrain normals cannot substitute for resolved shadows.
	if (!resolved && dot(geometric, light) <= 0.0)
	{
		// Suppress this source on its geometric night side.
		return 0.0;
	}
	float mapped_reflectance = hapke(dot(n, light), dot(n, view), dot(light, view));
	// Evaluate the smooth-sphere response independently of texture normals.
	float geometric_reflectance = hapke(dot(geometric, light), dot(geometric, view), dot(light, view));
	return mix(geometric_reflectance, mapped_reflectance, normal_strength);
}

float polar_height_weight(vec3 p)
{
	return u_polar_normal_ready * (1.0 - smoothstep(sin(-77.0 * PI / 180.0), sin(-75.0 * PI / 180.0), p.y));
}

float unpack_height(vec3 value)
{
	return dot(value.rg, vec2(65280.0, 255.0)) * 0.5 - 10000.0;
}

float terrain_height(vec3 p)
{
	float band = polar_height_weight(p);
	vec3 polar = vec3(0.0);
	if (band > 0.0)
	{
		vec2 uv = 0.5 + (1737400.0 / u_polar_extent) * vec2(p.x, -p.z) / max(1.0 - p.y, 0.000001);
		polar = textureLod(u_polar_height, uv, 0.0).rgb;
		band *= step(0.999, polar.b);
		if (band >= 1.0) return unpack_height(polar);
	}
	vec2 uv = vec2(fract(atan(p.x, p.z) / (2.0 * PI) + 0.5), acos(clamp(p.y, -1.0, 1.0)) / PI);
	float global = unpack_height(textureLod(u_height, uv, 0.0).rgb);
	return mix(global, unpack_height(polar), band);
}

float solar_visibility(vec3 p)
{
	const float R = 1737400.0;
	const float SUN_RADIUS = 0.265 * PI / 180.0;
	vec3 sun = normalize(u_sun);
	vec3 origin = p * (R + terrain_height(p) + 3.0);
	float along = dot(origin, sun);
	// Even the lowest terrain blocks this ray; mapped normals cannot leak sunlight into deep night.
	if (along < 0.0 && dot(origin, origin) - along * along < (R - 10000.0) * (R - 10000.0)) return 0.0;
	float global_spacing = 2.0 * PI * R / float(textureSize(u_height, 0).x);
	float polar_spacing = 2.0 * u_polar_extent / float(textureSize(u_polar_height, 0).x);
	float distance = 0.5 * mix(global_spacing, polar_spacing, polar_height_weight(p));
	float growth = pow(600000.0 / distance, 1.0 / float(u_shadow_steps - 1));
	float clearance_angle = 1.0;
	for (int i = 0; i < 48; ++i)
	{
		if (i >= u_shadow_steps) break;
		vec3 ray = origin + distance * sun;
		float radius = length(ray);
		// All encoded valid lunar terrain is below 12 km; the ray rises monotonically beyond this point.
		if (radius > R + 12000.0 && dot(ray, sun) > 0.0) break;
		float clearance = radius - R - terrain_height(ray / radius);
		clearance_angle = min(clearance_angle, clearance / distance);
		if (clearance_angle <= -SUN_RADIUS) return 0.0;
		distance *= growth;
	}
	float x = clamp(clearance_angle / SUN_RADIUS, -1.0, 1.0);
	return (acos(-x) + x * sqrt(max(0.0, 1.0 - x * x))) / PI;
}

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
	vec2 screen = v_screen + u_projection[2].xy;
	vec3 ray = normalize(vec3(screen.x / u_projection[0][0], screen.y / u_projection[1][1], -1.0));
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
	// All fragments in this draw take the same branch, before texture derivatives or Hapke evaluation.
	if (u_ray_mode != 0)
	{
		gl_FragDepth = gl_FragCoord.z;
		// Preserve opaque diagram colors; dimming is encoded in RGB, not photographic lighting.
		out_color = vec4(u_ray_color, 1.0 - smoothstep(0.15, 1.0, length(v_coverage)));
		// Rays require no albedo, normals or photometric calculations.
		return;
	}
	// Normalize the interpolated object-space direction before reconstructing longitude and latitude.
	vec3 position;
	vec3 p;
	if (!surface_point(position, p)) discard;
	if (u_shadow_pass != 0)
	{
		out_color = vec4(vec3(solar_visibility(p)), 1.0);
		return;
	}
	// Map atan longitude and acos latitude into normalized equirectangular coordinates.
	vec2 uv = vec2(atan(p.x, p.z) / (2.0 * PI) + 0.5, acos(clamp(p.y, -1.0, 1.0)) / PI);
	// Differentiate the continuous direction, not the wrapped atan output.
	vec3 dx = dFdx(p);
	vec3 dy = dFdy(p);
	float horizontal = max(dot(p.xz, p.xz), 0.000001);
	// Apply the analytic longitude/latitude derivative to horizontal screen variation, avoiding the wrap jump.
	vec2 gx = vec2((p.z * dx.x - p.x * dx.z) / (2.0 * PI * horizontal), -dx.y / (PI * sqrt(horizontal)));
	vec2 gy = vec2((p.z * dy.x - p.x * dy.z) / (2.0 * PI * horizontal), -dy.y / (PI * sqrt(horizontal)));
	// Decode the RGB8 object-space normal and normalize after filtering.
	// Texture state retains trilinear mipmaps but not the anisotropic path that produced dotted arcs in fixed-pose tests.
	vec3 n = normalize(textureGrad(u_normal, uv, gx, gy).rgb * 2.0 - 1.0);
	// South stereographic: x points east at longitude zero, y points toward longitude zero.
	vec2 polar_uv = 0.5 + (1737400.0 / u_polar_extent) * vec2(p.x, -p.z) / max(1.0 - p.y, 0.000001);
	float polar_band = 1.0 - smoothstep(sin(-77.0 * PI / 180.0), sin(-75.0 * PI / 180.0), p.y);
	float polar_blend = u_polar_ready * polar_band;
	float polar_normal_blend = u_polar_normal_ready * polar_band;
	if (polar_normal_blend > 0.0)
	{
		n = normalize(mix(n, texture(u_polar_normal, polar_uv).rgb * 2.0 - 1.0, polar_normal_blend));
	}
	// Apply the camera's rigid rotation to the normal, preserving lighting-space consistency.
	n = normalize(u_model * n);
	vec3 light = normalize(u_model * u_sun);
	// Keep the Earth source in the same world frame.
	vec3 earth_light = normalize(u_model * u_earth);
	vec3 view = normalize(vec3(0.0, 0.0, u_camera_distance) - position);
	// Parallel observer rays prevent the nearby zoom camera from sweeping an opposition hotspot across the surface.
	vec3 photometric_view = vec3(0.0, 0.0, 1.0);
	// Fade unresolved relief in the foreshortened limb band; use the true sphere normal below cosine 0.35.
	vec3 geometric = normalize(position);
	// Retain full relief above cosine 0.65, with a continuous smoothstep transition between both angles.
	float normal_strength = smoothstep(0.35, 0.65, max(dot(geometric, view), 0.0));
	// Suppress unresolved equirectangular polar detail, rather than displaying longitude streaks as terrain.
	float polar_fallback = smoothstep(cos(15.0 * PI / 180.0), cos(10.0 * PI / 180.0), abs(p.y));
	// Within ten degrees of either pole the overview deliberately uses a schematic sphere normal.
	normal_strength *= 1.0 - polar_fallback * (1.0 - polar_normal_blend);
	polar_fallback *= 1.0 - polar_blend;
	// A normal map cannot change silhouette visibility: back-facing mapped facets must not cut black holes.
	// Add only the missing view component, then restore unit length; leave already front-facing normals alone.
	n = normalize(n + max(0.0, MIN_EMISSION_COSINE - dot(n, view)) * view);
	// Add physically distinct direct solar and phase-scaled reflected Earth contributions.
	bool resolved = u_live_shadows != 0 && u_height_ready != 0;
	float visibility = 1.0;
	if (resolved)
	{
		visibility = u_mask_ready != 0
			? texture(u_shadow_mask, (gl_FragCoord.xy - u_shadow_viewport.xy) / u_shadow_viewport.zw).r
			: solar_visibility(p);
	}
	float reflectance = u_sun_scale * visibility * light_response(n, geometric, photometric_view, light, normal_strength, resolved)
		+ u_earthshine * light_response(n, geometric, photometric_view, earth_light, normal_strength, false);
	// SRGB8_ALPHA8 texture sampling already decodes color and filters in linear space.
	// Sample SRGB8_ALPHA8 color with explicit seam-safe derivatives; the GPU returns linear color.
	vec3 albedo = textureGrad(u_albedo, uv, gx, gy).rgb;
	// Use the documented mean reflectance in the unresolved cap; this is not a new terrain or color measurement.
	albedo = mix(albedo, vec3(0.4832), polar_fallback);
	if (u_live_shadows == 0)
	{
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
		albedo *= mix(vec3(1.0), satellite_factor, 0.85);
	}
	vec3 radiance = EXPOSURE * (albedo / 0.4832) * reflectance;
	// Compress positive linear radiance with the Reinhard operator before display encoding.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	// Encode once to sRGB and set alpha to one; egui_glow disables framebuffer sRGB encoding.
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
