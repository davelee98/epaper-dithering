# Test fixture images

Photographs used for regression testing and benchmarking.

## Copyright

All images © Gabriel. Included in this repository for testing purposes only.

## Images

| File | Content | Primary test coverage |
|---|---|---|
| `frankfurt_nacht.png` | Frankfurt skyline at night | Auto tone compression — dark scene |
| `unicorn.png` | Unicorn sculpture, Tollwood festival Munich | Gamut compression — vivid colors |
| `cat_orange.png` | Orange cat in grass, golden hour | Fine detail, warm tones |
| `cat.png` | Calico cat on tiles | High contrast B&W + color patches |
| `olympiapark.png` | Olympic Park, Munich | Outdoor scene, sky gradients |
| `river.png` | River/waterway with sky | Daylight outdoor, sky gradients |
| `ubahn_station.png` | Marienplatz U-Bahn station | Graphic/saturated, architecture |
| `seeed_opendisplay.png` | Synthetic color chart | Pure primaries, checkerboard, text |

## Benchmark-only

Images in `benchmark_only/` are full-resolution camera files used only for performance
benchmarks. They are not included in regression tests.

| File | Content |
|---|---|
| `cat.png` | Calico cat — full resolution |
| `cat_orange.png` / `orange_cat.png` | Orange cat — full resolution |
| `marienplatz.png` | Marienplatz U-Bahn — full resolution |
| `river.png` | River/waterway — full resolution |

## Adding more images

Drop any `.png` or `.jpg` file into this directory and run:

```bash
UPDATE_FIXTURES=1 cargo test --test regression
```

This generates reference outputs for all three regression suites and locks them in.
No code changes needed.