# Atomic loading: never publish half a Moon

## 1. Why successful downloads can still produce a broken image

The loading fixes span `c8b68fe` and `5e4d5b7`. This chapter describes the
final behavior at main `5e4d5b7`, not the incremental loaders quoted in older
chapters. Start with
[assets.rs](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/assets.rs),
[surface uploads](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/heights.rs),
[deep selection](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/deep.rs),
and [the GPU atlas](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/deep.rs).

A texture can be valid in isolation but incompatible with the other textures
currently being drawn. A higher-resolution global normal arriving before its
matching polar normal can change one part of the globe first. Enabling shadows
before their height maps or mask are ready can similarly divide the image into
different shading regimes. The user sees a split, even though no triangle is
missing and every individual request eventually succeeds.

The useful abstraction is a transaction: collect, validate and upload a new
resource set privately, then publish the set at one draw boundary. Atomic here
means logically indivisible to the renderer. It does not mean every byte is
transferred in one GPU command or that no temporary extra memory is needed.

## 2. A resource has more than two states

A robust mental model distinguishes:

1. Requested: network work exists.
2. Decoded: CPU/browser image data is available.
3. Staged: a GPU texture exists but is not referenced by the active surface.
4. Published: the active draw references a complete compatible set.
5. Retired: no active snapshot needs the old GPU object.

`Result` and `Option` encode different questions. `None` can mean work has not
finished; `Some(Err(...))` means it finished unsuccessfully. Treating either as
"ready enough" is how partial loading becomes visible.

Likewise, upload completion is not publication. OpenGL commands execute in order
on the current context, but Rust must still avoid replacing active handles one
at a time as independent responses arrive.

## 3. Publish a complete live surface

For a requested tier, the Realistic surface set contains global color, matching
global normals, global heights, polar heights and matching polar normals.
Height tier is capped at 4 even when the overview is 8k. Browser lossless WebP
normals and native PNG normals are alternative encodings, not different terrain.

The loader keeps these results together. Renderer staging retains the previous
active surface while uploads progress. Only a complete compatible replacement
may become active; a matching visibility mask must also be ready before dynamic
lighting is shown. An incomplete framebuffer or failed upload must leave the
consistent photographic/overview fallback rather than enable half the effect.

A compact teaching version of the policy is:

```text
if replacement has an error:
    keep active surface; report the error
else if replacement has every required texture uploaded:
    commit the compatible texture set
    invalidate/build its visibility mask
    enable dynamic lighting only with that mask
else:
    keep active surface
```

This is pseudocode, not a new Rust API. Read the actual staging and readiness
predicates before modifying the order. In particular, "selected Realistic" is
user intent; it is not proof the current GPU resources can render Realistic.

## 4. Tier changes are transactions too

Suppose an active 2k surface is complete and the user requests 4k. Replacing
global color immediately, then replacing polar normals a second later, creates
an intermediate material that was never a valid tier. The fix leaves the 2k set
active until the 4k set can replace it consistently.

This costs temporary memory for both generations. It also means a failed
upgrade can leave the old image looking softer than the selected target. That
is intentional: a complete old image is preferable to a sharper broken one.
The UI should preserve a diagnostic rather than silently claim the target is
active merely because its downloads started.

The same principle applies to fine normals and heights. The DEM-derived pair
shares one window specification. The mask is invalidated when the pair changes;
the image must not combine new fine normals with old fine heights and call the
result date-correct shadowing.

## 5. First-generation mosaic gating was not enough

The split fix initially hid WAC imagery until every currently requested tile
was resident. That prevented the initial partial mosaic, but introduced another
problem during navigation. If a new camera view wanted one missing tile,
`coverage_ready()` became false and the whole mosaic contribution switched off.
When the tile arrived it switched back on.

Because photographic contrast differs from overview albedo, this was a visible
brightness flash, not merely a sharpened crater. Exact float-bit view keys
increased the frequency of new selections during tiny camera changes.

Controlled reproduction reached the Fast photographic path and recorded
`u_deep_ready` changing 1 → 0 → 1. It required no NAC or height requests, and
the GL error was zero. Thus DEM shadows, NAC replacement and GL failure were
not necessary causes of that reproduced flash. Idle frames were already stable
before the fix; it would be inaccurate to claim persistent idle oscillation was
the verified root cause.

## 6. Separate published pages from staged tiles

The final atlas has an explicit `published: Vec<Tile>`. Its page table refers
only to that published set. New images can upload into spare texture-array
layers without modifying those references.

`replacement_slot(entries, wanted, published)` first chooses an empty layer.
Otherwise it chooses the least-recently-used entry that belongs to neither the
new wanted set nor the active published set. An old layer is not safe to evict
merely because it is no longer wanted: the active page table may still use it.

The transaction can be summarized as:

```text
upload replacement tiles into unreferenced layers
leave the published page table unchanged
when every wanted tile is resident:
    generate pages from wanted tiles only
    upload the complete replacement page table
    remember wanted as the published snapshot
```

`published_pixels` filters resident entries before constructing page-table
bytes. Otherwise a cached/staged child could silently become visible through
ancestor lookup before the replacement set was complete.

The table is 576×256 RGBA8: 512 columns describe global finest-level cells and
two 32-column strips describe polar cells. The shader obtains a layer and level
with one table lookup, then samples the appropriate 256² texture-array layer.
Ancestor lookup keeps the table compact without searching a CPU-style tree in
the fragment shader.

## 7. Budget two complete snapshots

The requested tile budget is 80. Two completely disjoint snapshots can therefore
need 160 layers. The previous 96-layer array could not guarantee keeping all
old layers while staging all new ones.

For 256² RGBA8 layers with a complete mip chain, the dominant storage is

$$
M=N\,256^2\,4\left(1+\frac14+\frac1{16}+\cdots\right)
\approx N\,256^2\,4\frac43.
$$

At 96 layers this is 32 MiB; at 160 it is about 53.33 MiB, excluding the page
table and driver overhead. The extra 21.33 MiB is a deliberate streaming-memory
tradeoff, not a free fix. It adds no startup dataset download.

Download concurrency remains four; decoded pending plus in-flight work is
bounded at six; uploads remain limited to four tiles per frame. Increasing
capacity did not remove queue limits or turn navigation into an unbounded
prefetcher.

## 8. Hysteresis stops boundary chatter

If a decision uses exactly the same enter and leave threshold, tiny noise can
toggle it every frame. A thermostat solves this with hysteresis; imagery needs
the same idea.

`State::needs_selection` compares against the last **accepted selection view**:

- changed viewport/aspect triggers a selection;
- $|\ln(z/z_0)|>0.14$ triggers one;
- maximum model-element difference times maximum zoom above 0.15 triggers one;
- a pan-component difference above 0.1 triggers one.

The model test is a practical screen-motion proxy, not a geodesic angle in
radians. Scaling by zoom makes small angular motion matter sooner near the
surface. Comparing against the accepted view accumulates movement: many tiny
steps eventually cross the threshold instead of being ignored forever.

An inactive WAC mosaic enters at 2×. Once active, it can remain enabled down to
1.7×. Small reversals near 2× no longer alternate whole shading paths.

Photographic detail has a second hysteresis policy: round the four-tile window
origin, retain the old window while the center is inside its central half, and
tolerate an adjacent level while log zoom remains within 0.14 of the accepted
request. Different sources/poles, non-photo windows and larger level changes
do not qualify. This is bounded reuse, not permission to freeze a photograph
over unrelated terrain.

## 9. Measure the transition, not just the final image

The controlled Chrome test held replacement WAC requests during a drag, saved
the pending image, released requests and saved the complete image. Each pair
had identical camera, projection and Sun uniforms. UI status text was excluded
from the Moon-region crop statistics.

| Recorded measurement | Main before fix | Fixed |
|---|---:|---:|
| Deep-ready pending / complete | 0 / 1 | 1 / 1 |
| Deep-ready transitions in draw trace | 2 | 0 |
| Mean absolute RGB difference, 0–255 | 11.379028 | 0.065835 |
| RMS RGB difference, 0–255 | 18.256353 | 0.493718 |
| Distinct idle hashes, five captures | 1 | 1 |

The mean discontinuity dropped 99.42% in this scenario. It is not a general
frame-rate benchmark or proof every possible temporal artifact is gone.
Alternating 1350×/1500× on a loaded polar NAC window retained its bounds and
made zero additional NAC requests in the recorded interaction.

## 10. Regression tests encode ownership rules

The native atlas tests include
`staged_tiles_cannot_change_published_pages_or_evict_active_layers` and
`full_disjoint_replacement_fits_without_overwriting_published_tiles`.
Other tests cover camera hysteresis and photographic-window retention. Empty
tables, no replaceable layer, different sources and large zoom changes matter
as much as the successful overlap case.

Browser checks still matter for actual uploads and draw ordering. Delay one
required texture, fail a replacement, then rotate while data is held. A page
that simply finishes loading cannot test atomicity. Keep the old surface visible
in the screenshots and inspect the published-page fingerprint while pending.

Finally, atomic publication does not remove real gaps or brightness strips in
source imagery. A complete NAC window with transparent NoData is rejected in
favor of WAC; a static WAC seam was observed but not isolated as source versus
filtering. Temporal consistency must not be advertised as complete photographic
calibration or universal seam removal.
