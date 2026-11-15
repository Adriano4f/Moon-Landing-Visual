# The Moon renderer, from zero

## 1. What this image means

This is a visual globe, not the landing-site simulation. The independent `globe` crate uses NASA color and relief products to draw an illustrative Moon. It does not query Horizons, calculate solar power, determine Earth visibility, ingest a scientific DEM into `lunar_core`, or predict terrain shadows. The camera and Sun are deliberately fixed for presentation. The globe turns once per 60 seconds. The model includes a measured lunar photometric parameter set, but its RGB modulation and display exposure are modeling choices, not a calibrated spectral inversion.

Read this guide first, then [the Rust and complete code guide](04-globe-code.md). Its annotated listings quote every globe source and both app entry files exactly. [Asset provenance](../globe/assets/README.md) provides downloads, dimensions, units, sizes, and hashes. The earlier [scientific walkthrough](02-code.md) describes the original placeholder app, not the current renderer.

## 2. Pixels, vectors, and surface directions

A pixel is one addressable image location. RGB assigns it red, green, and blue channel values. A framebuffer is the image storage into which rendering writes. A physical display pixel is not necessarily one UI coordinate: egui uses logical points and converts its paint rectangle to physical pixels. The callback's viewport supplies that conversion. Implementation: [globe/src/lib.rs:135](../globe/src/lib.rs#L135).

A three-component vector is an ordered triple. Addition and scalar multiplication act componentwise. Its Euclidean length is sqrt(x*x+y*y+z*z). A unit vector has length one; normalization divides by length and requires a nonzero vector. A point is a location, while subtracting two points gives a displacement vector. Our units for rendering positions are lunar reference radii.

The dot product a dot b = ax*bx+ay*by+az*bz equals length(a)*length(b)*cos(angle). For unit directions it directly gives a cosine. For example, (0,0,1) dot (1,0,0) = 0 means a right angle; (0,0,1) dotted with itself is 1. The shader uses dot products to ask whether a surface faces the Sun and camera. Implementation: [globe/src/moon.frag:174](../globe/src/moon.frag#L174).

The cross product a cross b = (ay*bz-az*by, az*bx-ax*bz, ax*by-ay*bx) is perpendicular to both vectors. Its sign distinguishes triangle orientations. For a=(1,0,0) and b=(0,1,0), a cross b=(0,0,1). The mesh test uses the z component of the cross product of two triangle edges to check outward winding. Implementation: [globe/src/math.rs:112](../globe/src/math.rs#L112).

Latitude measures angle north of the equator. Longitude increases eastward. In radians, latitude phi and longitude lambda become the unit direction

```text
p = (cos(phi) sin(lambda), sin(phi), cos(phi) cos(lambda)).
```

Thus north is +y, longitude zero faces +z, and longitude 90 degrees east faces +x. Tycho belongs in the southern hemisphere and Mare Crisium to the right of the near-side center. These are orientation checks, not an astronomical camera model. Implementation: [globe/src/math.rs:5](../globe/src/math.rs#L5).

## 3. Vertices, triangles, and the GPU

A vertex is a mesh corner with numerical attributes. Here it stores five floats: position x,y,z and texture coordinates u,v. A triangle references three vertices. An index buffer stores these references, allowing adjacent triangles to share vertices instead of repeating all their floats. A vertex buffer stores the floats themselves. A vertex array object records how these bytes should be interpreted. Each vertex occupies 20 bytes: position starts at byte 0 and UV starts at byte 12. Implementation: [globe/src/lib.rs:304](../globe/src/lib.rs#L304).

The sphere has 256 longitude cells and 128 latitude cells. Its duplicated boundary vertices give 257*129 = 33,153 vertices, 65,536 triangles, and 196,608 indices. A cell with upper-left index a and lower-left index b=a+257 uses (a,b,a+1) and (a+1,b,b+1). The two seam copies must have identical positions even though u is 0 on one and 1 on the other. The poles use exact (0,+1,0) and (0,-1,0), not tiny cosine residuals. Implementation: [globe/src/math.rs:13](../globe/src/math.rs#L13).

The graphics processing unit, or GPU, transforms and shades many vertices/fragments in parallel. OpenGL is the command API that supplies data and configures this pipeline. `glow` exposes that API in Rust and also maps it to browser WebGL2. GLSL is the language of the small GPU programs called shaders. The vertex shader processes one vertex. Rasterization converts projected triangles into covered sample locations. A fragment shader computes a color for a covered location before depth testing decides whether it survives.

A uniform is a value held constant for one draw, such as the rotation angle or projection matrix. A vertex attribute varies per vertex. Shader `in`/`out` declarations connect these values between stages. `gl_Position` is the vertex shader's special homogeneous clip-coordinate output. Implementation: [globe/src/moon.vert:19](../globe/src/moon.vert#L19) and [globe/src/moon.frag:144](../globe/src/moon.frag#L144).

## 4. Matrices and coordinate spaces

A matrix is a rectangular array defining a linear transformation by row-dot-column multiplication. A 4 by 4 matrix lets us combine rotations, translations, and perspective using homogeneous coordinates. A point (x,y,z) is represented as (x,y,z,1); a displacement uses fourth component zero so translation does not affect it. After a projective transformation, (x,y,z,w) and any nonzero scalar multiple represent the same point. Dividing x,y,z by w recovers its three ordinary coordinates.

With column vectors, translating by (0,0,-3.3) uses

```text
T = [1 0 0  0  ]
    [0 1 0  0  ]
    [0 0 1 -3.3]
    [0 0 0  1  ].
T * (1,0,0,1) = (1,0,-3.3,1).
T * (1,0,0,0) = (1,0,0,0).
```

Multiplication order matters: P*T*p translates first, then projects. OpenGL accepts our matrices in column-major memory order: consecutive groups of four floats are columns, not rows. The upload passes transpose=false. Implementation: [globe/src/lib.rs:494](../globe/src/lib.rs#L494).

Object space is the unrotated Moon's coordinate system. World space is object space after the model rotation. For angle theta around +y, world x=cos(theta)*x+sin(theta)*z, world y=y, and world z=-sin(theta)*x+cos(theta)*z. At theta=90 degrees, (0,0,1) becomes (1,0,0). A rigid rotation transforms normals by this same matrix. General nonuniform scaling would require an inverse-transpose normal matrix, which we do not need.

View space measures positions relative to the camera. Here the camera is C=(0,0,3.3), looking along -z without any camera rotation, so view position is world position minus C. The vertex shader computes this translation directly. Implementation of object, world, and view transformations: [globe/src/moon.vert:26](../globe/src/moon.vert#L26).

Clip space is the homogeneous output of the projection matrix. Let a be viewport width/height, n=0.1 the near distance, F=10 the far distance, and f=min(a,1)/tan(pi/8). Then

```text
P = [f/a 0       0               0       ]
    [0   f       0               0       ]
    [0   0 (F+n)/(n-F)     2*F*n/(n-F)    ]
    [0   0      -1               0       ].
```

The horizontal scale divides by aspect ratio so a sphere remains circular. The min(a,1) factor preserves a 45-degree field of view on the shorter viewport axis. The z rows map the near plane to normalized depth -1 and the far plane to +1. Implementation: [globe/src/math.rs:73](../globe/src/math.rs#L73).

For a square viewport, f=2.414214. A world point (1,0,0,1) becomes view (1,0,-3.3,1), then clip (2.414214,0,3.164646,3.3). Perspective divide produces normalized device coordinates, or NDC: approximately (0.73158,0,0.95898). This division by w makes equally sized distant objects appear smaller. Clipping removes geometry outside -w <= x,y,z <= w before the divide.

Window space maps NDC into the viewport. In an 800 by 800 viewport, x_window=(x_NDC+1)*400 gives approximately 692.63; y=400. Depth maps (z_NDC+1)/2 into [0,1], approximately 0.97949. OpenGL window y grows upward; UI and screenshot coordinates often grow downward, which egui handles for the callback. The projection tests check the near/far endpoints and both aspect orientations. Implementation: [globe/src/math.rs:152](../globe/src/math.rs#L152).

## 5. Rasterization, interpolation, and visibility

Rasterization finds which pixel samples lie inside each projected triangle. Barycentric coordinates are three triangle weights summing to one. They interpolate vertex quantities across a triangle. Under perspective, an attribute a uses

```text
a_fragment = sum(weight_j * a_j / w_j) / sum(weight_j / w_j).
```

This is perspective-correct interpolation. GLSL uses it by default for our floating-point outputs. Interpolated positions and normals must be normalized again where a unit direction is required. The shader reconstructs texture UV from interpolated object direction instead of interpolating a wrapped longitude directly. Implementation: [globe/src/moon.frag:147](../globe/src/moon.frag#L147).

A depth buffer stores the closest surviving depth at each sample. Without it, draw order could put a rear triangle in front. The callback enables depth testing with LESS, permits writes, and clears depth to 1. A fragment with a smaller depth replaces the stored value. Counterclockwise triangles are front faces; back-face culling discards the rear surfaces of this closed opaque sphere. Blending is disabled because alpha is always one. Implementation: [globe/src/lib.rs:474](../globe/src/lib.rs#L474).

Multisample anti-aliasing, or MSAA, tests multiple coverage locations per pixel and resolves them into a final color. Four samples soften the geometric silhouette; this does not mean four independent texture or lighting evaluations everywhere, and it does not replace mipmaps. Native `NativeOptions.multisampling=4` remains enabled. The browser uses its context's available antialiasing behavior rather than claiming native's exact sample count. Implementation: [app/src/main.rs:11](../app/src/main.rs#L11).

## 6. Textures, filtering, and a continuous longitude seam

A texture is an image addressed by normalized coordinates u,v. Our equirectangular maps use u=lambda/(2*pi)+0.5 and v=0.5-phi/pi: top is north, middle longitude is zero. Each stored pixel is called a texel. A sampler combines the image with rules for filtering and coordinates outside its bounds. Longitude repeats; latitude clamps to the edge. Implementation: [globe/src/lib.rs:417](../globe/src/lib.rs#L417).

Bilinear filtering blends four neighboring texels at one resolution. A mipmap is a sequence of smaller images, each approximately half as wide and high. When many texels fit under one screen pixel, sampling an appropriate smaller level suppresses shimmering. LINEAR_MIPMAP_LINEAR also blends between two levels, often called trilinear filtering. The level chain costs about one third beyond the base image because 1+1/4+1/16+...=4/3. Implementation: [globe/src/lib.rs:419](../globe/src/lib.rs#L419).

At a grazing angle the projected footprint is elongated, not square. Anisotropic filtering samples along that footprint. We enable it only when an extension name ends in `texture_filter_anisotropic`, query enum 0x84FF (MAX_TEXTURE_MAX_ANISOTROPY_EXT), and set 0x84FE (TEXTURE_MAX_ANISOTROPY_EXT) to min(8,max). Eight is an anisotropy ratio, not a promise of exactly eight texture reads. Without the extension, mipmapped trilinear filtering remains the fallback. Implementation: [globe/src/lib.rs:430](../globe/src/lib.rs#L430).

Ordinary automatic UV derivatives see u jump from almost 1 to almost 0 at the longitude boundary. They can incorrectly select a very small mip level and produce a visible seam. The shader instead differentiates the continuous unit direction p. For a differential dp, du=(pz*dpx-px*dpz)/(2*pi*(px*px+pz*pz)), and dv=-dpy/(pi*sqrt(px*px+pz*pz)). It obtains dp with dFdx and dFdy, bounds the polar denominator, and passes these explicit gradients to textureGrad for both maps. The texture still repeats, but the footprint never differentiates the wrapped u value. Implementation: [globe/src/moon.frag:158](../globe/src/moon.frag#L158).

## 7. Relief and normal maps

A normal is a unit vector perpendicular to the surface. A smooth sphere has radial normal r. A normal map stores perturbed directions that change illumination without moving the silhouette. These are object-space normals, not tangent-space normals: RGB directly encodes directions in the unrotated Moon coordinates. The shader rotates them with the geometry and renormalizes after filtering. Implementation: [globe/src/moon.frag:162](../globe/src/moon.frag#L162).

LOLA is the Lunar Orbiter Laser Altimeter. The offline script uses NASA's 16-pixels-per-degree unsigned displacement product. If q is an unsigned sample, height h=0.5*q-10,000 meters relative to reference radius R=1,737,400 m. This iteration uses exaggeration 1.0. Height is relief, not an already normalized display value. The source is 5,760 by 2,880: an 8k normal texture resamples this source and does not invent 8k scientific relief measurements. Implementation: [globe/assets/prepare.py:49](../globe/assets/prepare.py#L49).

At latitude phi, neighboring longitudes are separated by distance (R+h)*cos(phi)*d_lambda. Neighboring latitudes are separated by (R+h)*d_phi. Centered differences therefore give east slope p=(h_east-h_west)/(2*d_lambda*(R+h)*cos(phi)) and north slope q=(h_north-h_south)/(2*d_phi*(R+h)). The script wraps east/west neighbors, uses one-sided differences at the top/bottom rows, and tapers both slopes over the last degree around each pole because longitude is singular there.

The local east basis is E=(cos(lambda),0,-sin(lambda)). The local north basis is N=(-sin(phi)*sin(lambda),cos(phi),-sin(phi)*cos(lambda)). To first order, the unnormalized surface normal is r-p*E-q*N. For example, an eastward slope of 0.1 on an equatorial longitude-zero surface changes (0,0,1) to normalize((-0.1,0,1)), approximately (-0.099504,0,0.995037). The normal tilts away from rising terrain. Implementation: [globe/assets/prepare.py:24](../globe/assets/prepare.py#L24).

The bake encodes each component as round((component*0.5+0.5)*255) into RGB8 PNG, a lossless encoding of quantized vectors. Component quantization error is at most 1/255 after decoding. The shader decodes with 2*rgb-1 and renormalizes. Phones decode the two final images but do no height differentiation. Python tests check a flat sphere, a known metric slope, periodic neighbor influence, and invalid grids. This is not displacement tessellation or terrain shadow mapping.

## 8. Radiometry and the Hapke model

Radiometry measures electromagnetic energy. Irradiance E is incident power per area, measured in W/m^2. Radiance L is power per projected area per solid angle in a direction, measured in W/(m^2 sr). Reflectance is a dimensionless reflected fraction, but the word alone does not identify a directional law. A bidirectional reflectance distribution function, or BRDF, is outgoing radiance divided by incident irradiance for specified incoming/outgoing directions and has units sr^-1.

For a collimated beam with perpendicular irradiance J, our modeled quantity is radiance factor I/F=pi*L/J. It is dimensionless. The ISIS expression below is I/F, not a BRDF to multiply again by cos(i). Adding that cosine again would incorrectly darken the full Moon's limb. Our display uses a relative Sun intensity, not measured J. Implementation: [globe/src/moon.frag:47](../globe/src/moon.frag#L47).

Let S be the unit direction toward the Sun, V toward the camera, and n the local mapped normal. Incidence i satisfies mu0=cos(i)=n dot S. Emission e satisfies mu=cos(e)=n dot V. Phase g satisfies cos(g)=S dot V. Opposition means g=0: observer and Sun are in the same direction. A locally unilluminated or invisible facet returns zero. A separate geometric-normal test prevents mapped normals from lighting the back side. There is no ambient or earthshine term.

The verified implementation source is USGS ISIS3 commit `fadd004da2f733525209ee4f66ffdb3a9a9d193e`, [HapkeLROC.cpp](https://github.com/DOI-USGS/ISIS3/blob/fadd004da2f733525209ee4f66ffdb3a9a9d193e/isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp). Its HGfunc and photometry functions define the equations and sign convention. The companion [HapkeLRO.cpp](https://github.com/DOI-USGS/ISIS3/blob/fadd004da2f733525209ee4f66ffdb3a9a9d193e/isis/src/lro/apps/lrowacphomap/HapkeLRO.cpp) was also read, but we specifically follow HapkeLROC's signed mixing convention. [lrowacphomap.cpp](https://github.com/DOI-USGS/ISIS3/blob/fadd004da2f733525209ee4f66ffdb3a9a9d193e/isis/src/lro/apps/lrowacphomap/lrowacphomap.cpp) dispatches these models. This verifies an open implementation; direct Wiley access to Sato's appendix was denied, so we do not claim to have read it.

The scientific reference is Sato et al. (2014), "Resolved Hapke parameter maps of the Moon", JGR Planets, DOI [10.1002/2013JE004580](https://doi.org/10.1002/2013JE004580). Parameters come directly from the PDS [WAC_HAPKEPARAMMAP_566NM.IMG v1.1](https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP/WAC_HAPKEPARAMMAP_566NM.IMG), whose attached label was read. Its nine bands are w,b,c,Bc0,hc,Bs0,hs,theta,phi. We average all 50,400 valid one-degree tiles with cos(latitude) weights over 70 degrees south to 70 degrees north. These are means of fitted parameters over the mapped band, not a measured whole-Moon parameter set and not the mean of the nonlinear reflectance function.

| Parameter | Value | Meaning |
|:--|:--|:--|
| w | 0.3261142672 | Single-scattering albedo at 566 nm. |
| b | 0.2348928321 | Double-lobed Henyey-Greenstein shape. |
| c | 0.3574639778 | Signed backward/forward mixing. |
| Bs0 | 1.7707997700 | Shadow-hiding amplitude. |
| hs | 0.0622193166 | Shadow-hiding width in tan(g/2). |
| theta | 23.6566009521 degrees | Unresolved mean roughness angle. |
| Bc0, hc | 0, 1 | Coherent-backscatter amplitude and width. |
| phi | 0 | Porosity parameter; ISIS's porosity factor K is 1. |

Implementation constants: [globe/src/moon.frag:21](../globe/src/moon.frag#L21). Reproduce the means and numeric checks with `python3 globe/assets/photometry.py`.

### 8.1 Single-particle scattering and opposition

The single-particle phase function is

```text
P(g) = (1+c)/2 * (1-b*b)/(1-2*b*cos(g)+b*b)^(3/2)
     + (1-c)/2 * (1-b*b)/(1+2*b*cos(g)+b*b)^(3/2).
```

Positive c favors the first, backward lobe. It is not a [0,1] forward-lobe mixture weight. Interchanging those weights changes the opposition behavior and would mismatch these map values. Implementation: [globe/src/moon.frag:60](../globe/src/moon.frag#L60).

Shadow-hiding brightens near opposition because particle shadows disappear behind the particles casting them. Its normalized profile Bs(g)=1/(1+tan(g/2)/hs) enters as 1+Bs0*Bs(g). Thus Bs(0)=1. This is a separate physical mechanism from the microscopic phase function.

Coherent backscatter enhances reciprocal light paths very near opposition. ISIS uses x=tan(g/2)/hc and Bc(g)=[1+(1-exp(-x))/x]/[2*(1+x)^2], with continuous limit Bc(0)=1. Its multiplier is 1+Bc0*Bc(g). Every valid map tile has Bc0=0, so our shader omits this identically one multiplier rather than evaluating a numerically singular unused profile. Likewise phi=0 gives porosity K=1. Implementation: [globe/src/moon.frag:64](../globe/src/moon.frag#L64).

### 8.2 Multiple scattering

A photon may scatter more than once between grains. The Chandrasekhar H-function describes this aggregate contribution. We use the logarithmic approximation in HapkeLROC::HGfunc:

```text
gamma = sqrt(1-w)
r0 = (1-gamma)/(1+gamma)
H(x) = 1 / [1-w*x*(r0+(1-2*r0*x)/2*ln((1+x)/x))].
```

Here H(0)=1 by continuity. The shader bounds positive arguments away from zero to avoid log(1/0). The multiple-scattering term is H(mu0e)*H(mue)-1, where the e suffix denotes roughness-adjusted cosines defined next. This is not the simpler (1+2*x)/(1+2*gamma*x) rational approximation. Implementation: [globe/src/moon.frag:34](../globe/src/moon.frag#L34).

### 8.3 Macroscopic roughness

Unresolved tilted facets change the average incidence and emission and hide one another. This is distinct from the resolved LOLA normal slopes. We implement HapkeLROC's two roughness branches. Let t=tan(theta), chi=1/sqrt(1+pi*t*t), E1(x)=exp(-2*cot(x)/(pi*t)), and E2(x)=exp(-cot(x)^2/(pi*t*t)). Set

```text
eta_i = chi*(mu0+sin(i)*t*E2(i)/(2-E1(i)))
eta_e = chi*(mu +sin(e)*t*E2(e)/(2-E1(e)))
cos(psi) = [cos(g)-mu0*mu]/[sin(i)*sin(e)].
```

psi is the relative azimuth between the incidence and emission planes, in [0,pi]. If one plane is undefined at a normal direction, use psi=0, the limiting value for the terms that matter. Define z=sin(psi/2)^2, f=exp(-2*tan(psi/2)), with f(pi)=0.

For i<=e, D=2-E1(e)-(psi/pi)*E1(i),

```text
mu0e = chi*(mu0+sin(i)*t*(cos(psi)*E2(e)+z*E2(i))/D)
mue  = chi*(mu +sin(e)*t*(E2(e)-z*E2(i))/D)
q = mu0/eta_i.
```

For i>e, D=2-E1(i)-(psi/pi)*E1(e),

```text
mu0e = chi*(mu0+sin(i)*t*(E2(i)-z*E2(e))/D)
mue  = chi*(mu +sin(e)*t*(cos(psi)*E2(i)+z*E2(e))/D)
q = mu/eta_e.
```

The final roughness shadowing factor is S=mue*mu0*chi/[eta_e*eta_i*(1-f+f*chi*q)]. Shader denominator guards protect floating-point grazing limits. The complete radiance factor is

```text
I/F = (w/4) * mu0e/(mu0e+mue)
    * [P(g)*(1+Bs0*Bs(g))+H(mu0e)*H(mue)-1] * S.
```

Implementation: [globe/src/moon.frag:67](../globe/src/moon.frag#L67) through [globe/src/moon.frag:133](../globe/src/moon.frag#L133). There is no additional Lambert cosine multiplication.

The independent Python implementation checks 1,296 valid geometries, a dark facet, and opposition. At g=0, the center gives 0.19113042 and a location with mu=mu0=0.2 gives 0.18828614, only 1.49% darkening. At phase 0, 1, 5, 30, and 90 degrees with i=e=g/2, I/F is 0.19113042, 0.17695949, 0.14318869, 0.08717426, and 0.04459440. This checks the formula independently; it is not GPU readback or a calibration against lunar observations. Normals and the fixed off-axis light mean the displayed screenshot is not a full-Moon test image.

## 9. Linear light, color maps, and tone mapping

sRGB is a nonlinear encoding of display RGB. A stored channel c decodes to c/12.92 when c<=0.04045, otherwise ((c+0.055)/1.055)^2.4. Multiplication and interpolation for physical light belong in linear values, not encoded values. We upload albedo as SRGB8_ALPHA8, so texture sampling decodes it before filtering; normal maps use ordinary RGB8 and must not be color-decoded. Mipmap generation uses the texture's color-space rules. Implementation: [globe/src/lib.rs:396](../globe/src/lib.rs#L396).

The NASA map is aesthetic RGB imagery, not a per-pixel inversion of 566 nm Hapke parameters. Our explicit modeling choice is to multiply the common photometric response by each linear RGB channel divided by 0.4832, a rounded common area-weighted color-map luminance reference. Individual tier pre-JPEG means are recorded in manifest.json and are approximately 0.483. This preserves relative spatial brightness and color but does not convert RGB to measured spectral single-scattering albedo. It applies the mapped-band average outside its original 70-degree latitude coverage too, including the poles, solely for presentation.

Exposure multiplies this relative radiance by 4.0. For exposed x>=0, the simple per-channel Reinhard tone map y=x/(1+x) compresses values smoothly below one. For example, x=0.5 gives 1/3 and x=2 gives 2/3. This is the global compression associated with Reinhard et al. (2002), "Photographic Tone Reproduction for Digital Images", DOI [10.1145/566654.566575](https://doi.org/10.1145/566654.566575), not the complete local adaptation method. Per-channel compression can alter hue. Implementation: [globe/src/moon.frag:187](../globe/src/moon.frag#L187).

Finally, linear y encodes to 12.92*y for y<=0.0031308, otherwise 1.055*y^(1/2.4)-0.055. The shader writes sRGB with alpha one. egui_glow disables automatic framebuffer sRGB conversion for custom callbacks, so this explicit encoding is required and not applied twice. The [Khronos sRGB specification](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_sRGB.txt) defines these transfers. Implementation: [globe/src/moon.frag:137](../globe/src/moon.frag#L137).

## 10. Runtime quality, deployment, and limits

A quality tier selects one albedo and one normal map, never all six. First read GL_MAX_TEXTURE_SIZE. On native, default to requested width 8,192. On web, lowercase navigator.userAgent and consider it mobile if it contains android, iphone, ipad, or mobile, or if window.innerWidth is <=600 CSS pixels. Mobile requests 2,048; other browsers request 8,192. This is a conservative heuristic, not a measurement of available GPU memory. A phone in desktop mode with a wide viewport can be misclassified; use the override. Implementation: [globe/src/assets.rs:59](../globe/src/assets.rs#L59).

Native MOON_TIER=2, 4, or 8 and web ?moon_tier=2, 4, or 8 override the preference. The k-suffixed forms also work. Choose the largest of 8,192, 4,096, 2,048 not exceeding both preference and hardware limit. Thus forced 8k on a 4,096-limited GPU becomes 4k. A limit below 2,048 or invalid override produces an error, not an untested smaller tier. Implementation: [globe/src/assets.rs:18](../globe/src/assets.rs#L18).

| Tier | Dimensions of each texture | Two RGB8 images decoded on CPU | Conservative GPU estimate |
|:--|:--|:--|:--|
| 2k | 2,048 by 1,024 | 12 MiB | 21.33 MiB, about 22.37 MB |
| 4k | 4,096 by 2,048 | 48 MiB | 85.33 MiB, about 89.48 MB |
| 8k | 8,192 by 4,096 | 192 MiB | 341.33 MiB, about 357.91 MB |

A MiB is 1,048,576 bytes; an MB is 1,000,000 bytes. GPU estimates assume two four-byte-per-texel allocations and all mipmaps, even though RGB is supplied. Drivers can use different internal storage. Decoder temporaries, compressed source bytes, MSAA color/depth, the window framebuffer, and driver overhead add memory. GL_MAX_TEXTURE_SIZE does not guarantee sufficient free memory. An upload error is displayed on black; there is no hidden retry that downloads another tier. The approximate 2k footprint is the phone default.

Native resolution order is MOON_ASSET_DIR if set; otherwise assets/globe beside current_exe if that directory exists; otherwise the crate's assets directory recorded by CARGO_MANIFEST_DIR at compile time. The last is a developer fallback, not a promise that an installed binary finds deleted source files. To distribute, place the six files in the executable-relative directory or supply just the desired tier and force it. If an existing directory lacks a selected file, show its error rather than silently searching elsewhere. Two worker threads read and decode only the chosen pair. Implementation: [globe/src/assets.rs:116](../globe/src/assets.rs#L116).

Web uses ehttp 0.7.3, a callback-based HTTP client. It fetches relative assets/globe/albedo_Nk.jpg and normal_Nk.png. Both must be served with the application. Trunk copies all available tiers into its distribution, but copying to server storage does not download them to each browser: runtime requests select exactly two files. There is no include_bytes! for imagery, service-worker precache, or scientific JSON fetch. The WASM file contains the small shaders through include_str!. The scheme works on a static host without threads, SharedArrayBuffer, or cross-origin isolation headers. HTTP failure or decoder failure becomes a readable error. On WASM image decoding runs in the callback on the main thread; it avoids expensive normal generation but is not an off-main-thread decoder.

```sh
cargo run -p app --release
MOON_TIER=2 cargo run -p app --release
MOON_ASSET_DIR=/path/to/assets/globe MOON_TIER=4 ./target/release/app
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cd app
trunk serve --release
```

Open the local URL with ?moon_tier=2 to force the phone-sized tier. For a subdirectory deployment, set Trunk's public URL to that subdirectory and serve the generated index and assets together. The browser still requests only the relative chosen pair. Upload also temporarily allocates an opaque RGBA albedo copy (8, 32, or 128 MiB). Both startup/loading and the surrounding panel are black; a short label distinguishes pending work from a failed asset. No web hosting provider is required for native use.

## 11. Checking and extending the implementation

Run cargo build -p app --release, cargo check -p app --target wasm32-unknown-unknown, cargo test --workspace, and cargo clippy --workspace. Format Rust with cargo +nightly fmt --all. Run python3 -m unittest discover -s globe/assets -p 'test_*.py' and python3 globe/assets/photometry.py for the production bake and numeric model. The core crate remains independent.

A real visual check must wait for "Moon textures ready", inspect near-side orientation, then inspect the antimeridian during rotation. Check both poles and the terminator at high and forced low tier. A real browser check must use mobile emulation, record the selected-tier console message and both successful image requests, inspect the rendered screenshot, and collect page/console errors. A successful WASM compile is not a phone-rendering test.

These approximations remain: a smooth spherical silhouette, no terrain occlusion or resolved cast shadows, a map-band mean parameter set rather than spatial Hapke fitting, one monochromatic parameter set applied to RGB, presentation camera/light/exposure, and heuristic quality selection. The black facets of steep mapped terrain near the limb/terminator are local shading, not a scientific horizon result. Future scientific work must keep those limitations separate from the existing terrain-ray computations.
