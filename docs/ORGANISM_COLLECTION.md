# Selecting and preserving organisms

Life includes a searchable, paginated browser of every living connected body, ordered by size or ID. Clicking a body focuses it and opens its details. World picking uses projected cell positions, a minimum 12-pixel tolerance, terrain occlusion checks, and a visible bounding box for the selected body. Physical cell sizes are unchanged. Picking does not raycast each instanced cell mesh or add a per-frame search.

Name a selected body and Save to collection. Capture pauses the world. The Rust engine captures every member cell, primary and homologous chromosomes, gene promoters, chemicals, lifecycle, age, injury and material stores, relative 3D positions, bonds, mineral joints, and internal signaling contacts/weights. This is the connected body's current state, not a list of visual traits or a genetically uniform species. Existing gene-detail and specimen previews still have their display limits; collection capture is not truncated by them.

Snapshots are immutable and persist in a separate browser IndexedDB database, independent of world saves. Export and import use versioned `primordia-organism` JSON. Import goes through engine validation before storage. Unknown versions or invalid packets are rejected. A failed local capture save exports the packet instead of discarding it. Other live world tabs do not need to close for a world-database schema upgrade.

Collection → Place a copy pauses the world and enables a crosshair. Drag/zoom normally, then click a clear terrain location. Cancel or Escape leaves placement mode. Multiple copies can be placed before resuming. The source packet remains unchanged. Copies receive fresh cell, ancestral-root, and genome identities; they preserve geometry in a local surface frame, genes, age, current material state and learned signaling. They start at rest with ground contact and cumulative activity counters reset. Material and joint state may react differently on a different terrain or under different resources.

Placement is staged atomically. Bad locations, collisions, cell-budget overflow, invalid state, or exhausted identities reject the operation without changing the world. Collision checks use spatial buckets. Introduced organic matter, mineral, chemical energy, internal water and gas are added explicitly to the conservation baselines. Introductions are journaled as external material input, not fission, births, mutations, or fitness improvement. An insertion into the restored main state clears the old matched checkpoint; interventions in active comparison runs remain blocked.

Copies carry collection source names. Fission propagates them; sexual fusion combines up to 16 source names. These are labels for inspection, not mating rules. Place different snapshots near the same food patch to observe competition through the existing engine. Feeding, defenses, growth, movement and compatible gametic fusion still require the relevant genes, materials, contacts and resources; collection placement does not force combat or reproduction. Use Diet & exchanges and Evidence of function to inspect actual flux and paid movement.

Validation performed during development: Rust library/test-target compilation, single-job WASM compilation, TypeScript and production build. No simulation tests, benchmarks, or browser simulation were executed. Runtime checks below are for the user:

1. Click near a small cell or select an ID in Find an organism; confirm the highlighted whole body and inspected ID match.
2. Preserve a multicellular mutation, export it, refresh and import it; inspect cell count, chromosomes, bonds and material functions.
3. Place two different snapshots in clear nearby spots, then resume; inspect their actual feeding and offspring. Copies should have distinct identities and collections should remain frozen.
4. Attempt an occupied or over-budget placement; confirm the world stays unchanged and a useful message appears.
5. Save and restore the world after insertion; confirm copied cells, ancestry labels and conservation balances persist.
