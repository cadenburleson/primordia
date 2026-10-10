# Configure first, then observe evolution

Startup no longer sends an `init` command. It opens planet setup with no Rust world, no seeded cells, no worker simulation loop and no WASM initialization. Only **Create planet & begin** constructs the configured planet and starts evolution. **Open** and **Restore** explicitly load a saved world paused. Closing setup leaves an empty view; Configure planet / New world reopens setup.

Setup includes habitat, size, seed, geometry, founder placement, population counts, ecology, starting sunlight, baseline temperature, fuel and minerals. **Evolution pace** defaults to Max with Evolution first display priority. **Use a quick experiment** is an optional 128-unit spherical planet with 64 founders and a 4,096-cell ceiling. It uses the existing tidal parameters and four founder genes; it does not inject advanced functions or change mutation/physics rules. The 128-unit sphere has an 80×40 environmental field instead of the 160×80 capped grid used by larger worlds. That is fewer environmental sites, not a measured speed guarantee or proof that complex anatomy will evolve.

## What matters first

Overview is the default observation view. Its four signals are:

- Generation: maximum descent reached during this run, including ancestors that may no longer survive.
- Largest living body: the largest connected body in the current packed display, not its historical peak.
- Natural gene changes: accumulated promoter/strength edits, duplications, deletions and innovations. Manual gene edits and specimen introductions do not increment these counters. A large mutation count does not mean improvements survived.
- Functional regions: measured connected groups of at least three cells satisfying existing material/function thresholds. One cell can contribute to multiple kinds of region. These are candidate functional regions, not confirmed anatomical organs or useful walking.

Keep a reproducing population first. Then inspect whether descendants keep attachments, acquire materials/functions, and reproduce successfully. Life gives organism selection and preservation; Ecology gives measured uptake/output and food contacts. Evidence runs user-started mechanism-ablation comparisons when a body appears worth investigating. Ground contacts and ATP spent on contraction do not establish locomotion. Light-fed organisms can succeed while stationary; resource scarcity can produce extinction. No timetable or guaranteed progression to organs or intelligence is promised.

Shape has exclusive Climate, Surface and Life tools; only one tool panel is shown at a time. Gene/mechanics details and population history are also collapsed. Default startup no longer auto-selects an organism or opens the full gene inspector.

## Simulated time versus display refresh

Speed is requested model cycles per wall-clock second; **actual cycles / second** reports what the worker achieved. Biology always advances with the existing fixed 0.25-cycle step. Raising requested speed or selecting Max does not enlarge the physical step or force mutations.

**Let evolution run · Max** selects Max and Evolution first. Periodic snapshots are prepared every two seconds in Evolution first, compared with every 500 ms at Max in Live view (250 ms at other Live speeds). Commands such as pause, select, save, placement and editing still publish immediately. Physics, reproduction and mutation continue between snapshots. The worker retains its bounded batch and idle yield; a single long synchronous physics step can still overrun that budget.

The old FPS label counted completed redraws, including the renderer intentionally skipping unchanged frames. At Max a stationary scene commonly redrew twice per second to match snapshots, so “2 FPS” was not a measurement of camera frame capacity. The display now says **View on demand** and reports physical-step cost instead. Camera interaction may draw up to 30 times per second in either priority mode; actual interaction performance still depends on the device and scene. It is not rendered continuously to inflate a counter. Hovering the status gives physical-step, snapshot preparation and world display preparation costs measured during the user's run. Snapshot cost includes projection, metadata serialization/parsing and packed exports; display cost measures CPU preparation rather than GPU frame time.

## Avoided work

- The Rust body projection is reused for display metadata and packed render exports instead of being built twice per snapshot.
- Selected inspector nodes/links/joints are limited to the existing 256-node inspection scope; full-body observations and organism capture still include the whole organism.
- The world renderer uses numeric packed-node references and reusable position buffers instead of per-cell string maps and multiple vector objects. Body instance matrices are prepared once per snapshot.
- Seeded terrain coordinates, ground heights and radial normals are cached; terrain normals are recomputed after terrain changes rather than every unchanged spherical refresh.
- Hidden tabs skip the full inspector, lineage and journal HTML generation. The specimen preview updates once, only while Life is visible. Closed history does not draw its chart. Hidden specimen canvases do not render.

These are source-supported reductions in work, not benchmarked gains. No browser interaction, simulation, native/WASM test or performance benchmark was executed for this revision. Rust compilation, TypeScript checks, production bundling and static control checks are used; runtime evaluation remains with the user.
