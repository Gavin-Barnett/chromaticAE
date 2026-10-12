# Performance measurements - 2.2.0

Measured on 12 October 2026, Windows x64, Intel Core i7-8700K at its configured system settings. These are single-thread CPU **kernel** timings, not end-to-end After Effects project render times.

| 1920 x 1080 input | 2.1 | 2.2 | Speedup | 2.2 with Centre Colour Restore at 100% |
| --- | ---: | ---: | ---: | ---: |
| Integer channel split | 154.85 ms | 66.87 ms | 2.32x | 93.31 ms |
| Fractional channel split | 151.87 ms | 97.60 ms | 1.56x | 120.84 ms |
| Horizontal softness | 313.63 ms | 211.71 ms | 1.48x | 236.99 ms |
| Horizontal and vertical softness | 549.19 ms | 266.25 ms | 2.06x | 294.15 ms |
| Both-axis softness, 256 x 256 requested output region | 393.03 ms | 10.63 ms | 36.97x | 11.29 ms |

The comparison uses the frozen 2.1 engine and current engine in the same executable, compiled with `rustc -O`. Each case has one warm-up for each variant, followed by five timed runs with rotating execution order. The table reports the median. The input contains varying colour and alpha; the channels use -8/0/+8 pixel offsets, with 0.37 pixels added for fractional cases. Softness is 12 pixels in each active axis, Mix is 100% and Repeat Edge Pixels is on. Centre restoration is off in the direct version comparison and on in the final column. The small output region begins at [800, 400].

## Changes

- Integer translations read one source pixel instead of four; fractional translations confined to one axis read two.
- Vertical blur accumulates contiguous rows, improving memory locality on larger frames.
- Single-axis blur skips unused-axis passes and their image copies.
- Blur intermediates cover the requested output region plus full convolution support, instead of processing the complete input for every cropped request.

The kernel retains the previous filter and edge behaviour. Regression tests compare the optimized result with the frozen 2.1 engine at a per-component tolerance of 0.00003 across blur modes, cropped regions, outside-source sampling, mixed results and both edge policies.

Centre restoration adds a smooth spatial blend with the original input. It is fully active inside 35% of the layer's elliptical radius and fades to zero at 90%. It is centred on the original layer bounds, even when input buffers have been expanded or cropped.

## Reproduce

Run `native\benchmark.ps1` from PowerShell with stable Rust/MSVC installed. The script also supports the workspace-local Rust toolchain used by `native\build.ps1`.

Host input checkout, upstream effects, pixel-format conversion, compositing, cache state and export encoding are outside this benchmark. The effect still requests the full input from AE to preserve upstream-effect and repeat-edge behaviour. Gains for an actual composition depend on its content and requested region; Centre Colour Restore also has its own blending cost. This remains a CPU effect with no GPU acceleration.
