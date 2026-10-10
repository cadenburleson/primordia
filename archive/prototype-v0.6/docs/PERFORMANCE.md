# CPU scheduling and GPU compute

## Current changes

The biology already runs in one Web Worker, outside the UI thread. Three.js renders through the GPU. Async work keeps controls responsive; additional CPU threads would not by themselves reduce total CPU use.

The worker now checks a six-millisecond compute deadline between individual fixed WASM steps, then waits at least 35 milliseconds after its work completes. At most 32 steps are pending per batch, so overloaded acceleration does not accumulate unlimited wall-time debt. Each completed biological step still uses the existing fixed timestep and rules. Under load, model time advances more slowly than the requested acceleration.

The worker stops periodic stepping and snapshots when the document is hidden. Returning resumes without simulating the hidden interval. Pausing keeps the last snapshot instead of rebuilding unchanged Rust metadata/render buffers ten times per second. Commands such as selection and load still publish a fresh snapshot.

These are scheduling bounds, not an OS-level CPU percentage limit. One synchronous WASM step and snapshot packing cannot be interrupted, and rendering also consumes CPU. The measured CPU reduction is unverified: only a production build/static checking was performed for this change. Runtime and performance testing are reserved for the user; do not run simulation tests, browser evolution runs or benchmarks unless explicitly requested.

## GPU path (not implemented)

WebGPU compute shaders can execute bulk numerical operations on the GPU. [WebKit documents GPU compute and its Metal mapping](https://webkit.org/blog/17333/webkit-features-in-safari-26-0/#webgpu); the [WGSL specification](https://gpuweb.github.io/gpuweb/wgsl/) defines the shader language. GPU support must be detected in the user's actual browser rather than assumed.

A hybrid backend would keep variable genomes, ordered RNG, birth/death events, ancestry and save ownership in Rust while offloading repeated particle forces/integration and local signal calculations. Resource uptake must preserve a defined allocation rule where cells compete for the same finite patch. Parallel writes to shared forces/stores require explicit accumulation rather than racing neighboring cells.

Implementation requires packed simulation buffers, WGSL kernels, synchronization with the Rust biological update and a CPU fallback. Keep GPU data resident through physics substeps and rendering where feasible; avoid reading all cell state back after every dispatch. Save/export must capture a completed update, including the GPU-owned state. Floating-point differences must be disclosed; exact CPU/GPU trajectories are not promised.

Before changing backends, remove repeated full-Core clones from render-buffer projection and measure where time is spent on the user's machine. The 1,024-cell bound and 40×40 field are small enough that GPU dispatch/readback overhead may outweigh some kernels. Acceptance requires user-run comparisons of CPU use, completed cycles per second, responsiveness and valid biological saves; a successful build does not establish a speedup.
