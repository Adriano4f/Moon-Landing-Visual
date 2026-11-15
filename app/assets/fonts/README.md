# Interface fonts

The browser embeds renamed Latin, punctuation and geometric-symbol subsets of Ubuntu Light and Hack Regular from `epaint_default_fonts` 0.37.0. Glyph outlines, layout metrics and hinting are retained for the supported English interface. Hack supplies the geometric symbols and arrows; the unchanged 3.5 KB egui icon font preserves framework icons. Native builds keep the original full default fonts.

The modified fonts are named KarayaUI and KarayaMono, respecting the original font-name restrictions. Their licenses and the unchanged egui icon license are included in this directory. These fonts are presentation assets, not scientific data.

Regenerate with `python3 subset.py PATH/TO/epaint_default_fonts-0.37.0/fonts`. Python fontTools is required. No external font is fetched by the application. Browser tests check interface text, degree signs, bullets, squares, arrows, empty text and an unsupported code point.
