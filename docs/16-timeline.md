# UTC timeline

The timeline and statistics period are shared across sites. Open **Time > Period**, enter start and end as `YYYY-MM-DD`, then select **Apply period**. Valid limits are 2027-01-01 and 2035-01-01, with start strictly before end. The end is an exclusive duration boundary but remains selectable for display. Invalid input leaves the previously applied period unchanged.

**Step** selects 1 hour, 1 day or 1 calendar month, for both navigation and statistics sampling. Compact `<` and `>` buttons share one short help icon immediately to their right; the timeline has no other help popups. Month arithmetic preserves UTC time of day and clamps to the destination month's last day, including February 29. Repeated month-end steps therefore use the resulting clamped date, not an invisible original day. A held button starts repeating after 400 ms, then repeats at 10 Hz without an extra release step. Both directions clamp to the applied period.

The slider has second-resolution time. Current Sun/Earth positions, statuses, globe rays and numerical markers interpolate normalized local sky-direction vectors between hourly samples. This avoids an incorrect long interpolation through 180 degrees when crossing north. No direction is extrapolated beyond available sky data. Paths remain hourly, clipped to the chosen period and the separate Paths window.

Statistics use samples anchored at the selected start, followed by the selected step, with the exact end appended when needed. The core integrates left-held states over actual durations, not sample counts. Any visible Sun and Earth clearance retain their previous definitions. The cards label the applied start, end and sampling step. Daily and especially monthly results can alias visibility, miss short events and distort longest night; use hourly sampling for comparisons. These are sampled estimates, not continuous event solutions.

The original seven site tracks end at 2030-01-01. A period reaching beyond existing data lazily requests the 2,244,152-byte `moon-ephemeris.bin`, shared by all sites. No complete ephemeris request occurs at startup. Each site appends at most 512 epochs per frame using its stored terrain altitude plus observer height; existing reference samples remain unchanged. Missing or failed sky data shows an explicit unavailable/loading state and does not publish partial-period statistics. Previously available periods remain usable. A failed complete-sky request can be retried by reloading the application.

The zoom pair now has one help icon. Texture quality and its status each use a nonwrapping row with their help icon. The step pair likewise has one help icon immediately to its right.

`d7de796` and `64ae30f` provided the earlier redesign and help changes, not the selectable step, calendar arithmetic, repeat behavior or selected-period statistics. PDF and full source-quotation regeneration are deferred.
