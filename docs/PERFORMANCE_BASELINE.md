# Performance Baseline

## Scenario

`survival_spawn_v1` is the fixed initial-spawn streaming workload.

- Build: `cargo run --release`
- Resolution: 1280x720
- VSync: disabled
- World seed: `3273072678` (`0xC3172026`)
- Render and mesh distance: 8 chunks
- Data vertical radius: 2 chunks above, 3 below
- Warmup: 5 seconds
- Sample window: 15 seconds
- World state: new isolated survival world, player idle at the default spawn

Run it from the repository root:

```powershell
.\tools\performance_baseline.ps1
```

The script recreates only `target/perf-run`, runs the scenario, and writes the machine-readable report to `target/perf/survival_spawn_v1.json`.

## Baseline

Recorded on 2026-07-15 with the Rust release profile on Windows.

| Metric | Baseline |
| --- | ---: |
| Frame time mean | 7.03 ms |
| Frame time P95 | 8.71 ms |
| Frame time max | 12.67 ms |
| Chunk tasks mean total | 1.41 |
| Chunk tasks max total | 7 |
| Terrain tasks max | 4 |
| Structure tasks max | 1 |
| Mesh tasks max | 4 |
| Process memory last/max | 0.845 GiB |
| Expected/loaded chunks | 1518 / 1518 |
| Rendered chunks | 911 |

All terrain, structure, and mesh task counts were zero at the end of the sample window. Treat these values as a same-machine regression baseline; hardware and driver differences make cross-machine comparisons unreliable.
