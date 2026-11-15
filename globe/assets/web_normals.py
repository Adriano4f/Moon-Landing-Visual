"""Create browser-native, lossless normal textures; never quantize directional data."""
from pathlib import Path
import json
from PIL import Image

ROOT = Path(__file__).resolve().parent


def main():
    """Keep native PNGs and emit exactly equivalent browser WebP pixels."""
    results = []
    for tier in (2, 4, 8):
        source = ROOT / f"normal_{tier}k.png"
        target = ROOT / f"normal_{tier}k.webp"
        with Image.open(source) as image:
            rgb = image.convert("RGB")
            rgb.save(target, format="WEBP", lossless=True, quality=100, method=4)
            with Image.open(target) as decoded:
                if decoded.convert("RGB").tobytes() != rgb.tobytes():
                    raise ValueError(f"Normal pixels changed: {target}")
        results.append(dict(tier=tier, png_bytes=source.stat().st_size,
                            webp_bytes=target.stat().st_size,
                            changed_channels=0, angular_error_deg=0))
        print(json.dumps(results[-1]), flush=True)
    print("All normal textures are pixel-identical; angular error 0 degrees.")


if __name__ == "__main__":
    main()
