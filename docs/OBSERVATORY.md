# Planet-first observatory

The world canvas fills the screen. Guide, Shape, Inspect and Collection are floating translucent buttons; only one drawer opens at a time. Clicking an organism opens inspection. Closing a drawer or pressing Escape restores the unobstructed view. Brush and organism placement close the drawer so the planet is clickable. Stats, history, resource layers and mechanism details appear on request. This is CSS transparency without backdrop blur or extra WebGL effects.

Planet creation remains separate. No world is simulated before Create or an explicit saved-world load. The first view shows the whole planet; View living cells and organism inspection take the camera closer. Save the current world before refreshing into this build.

## Guide

A compact suggestion opens the guide. Choose one question: Build bodies, Seek food, Survive rivals or Changing world. Only its three-step recipe is expanded. Compact readings show peak generation, divisions and connected bodies, without inventing a complexity score. Advice uses available observations: extinction, population ceiling, pause state, cell divisions, connected bodies, selected-body fuel sources, and population change over at least ten sampled model cycles. A decline is reported as an observation, not diagnosed as starvation. Imports and manual interventions can affect these counts.

The recipe explains when to inspect resources, preserve a checkpoint, change one variable, investigate a body, and introduce saved copies. Links open the relevant tools. The guide never changes genes or applies environmental edits on its own. Largest-body selection is a starting point for inspection, not a complexity score. Function tests help distinguish actuation from drift, growth or gravity. No optimal recipe or guaranteed time to organs is claimed.

See [the research basis and anatomical limits](EVOLUTION_PRESSURES.md) and [drag painting and seasonal controls](WORLD_TOOLS.md).

## Evolution nursery

Open from the guide after creating a main cellular planet. The worker saves the exact parent world to `nursery-parent` in browser storage before swapping worlds. Only the nursery advances; the parent world is held. If parent storage fails, entry aborts and the planet remains intact, paused.

The initial nursery is a 128-unit spherical tidal habitat with 64 local founders, patchy resources and a 4,096-cell ceiling. It uses the same four founder genes, physical steps, mutation and inheritance mechanisms. It starts at Max with Evolution first display priority. Smaller environmental fields reduce the amount of work; this is not a benchmarked acceleration or a promise of more complex anatomy.

Return restores the exact parent and pauses it. The nursery is retained in memory for later visits during this session; its most recent return checkpoint is also stored under `nursery-latest`. Save a nursery world explicitly before refreshing if you want a portable file for further evolution. More → Recover last held planet restores the durable parent after a refresh; this checkpoint is shared by tabs on the same origin. Recovery outside a nursery also holds the current world for undo.

While a nursery is active, world replacement and legacy lab commands are blocked until return. Checkpoint comparisons require returning to their checkpoint before switching worlds. Capture, inheritance, mutations, ecological selection, terrain and resources continue using the normal cellular engine.

## Preserve and populate

Select a body, name it and Save to collection. Captures retain all cells and chromosomes, material stores, physical structure and signaling state. Collection entries are shared between the nursery and planet. In a nursery, **Return & place on planet** restores the parent first and arms placement only after the parent snapshot arrives. Click a surface site, place more copies if desired, then resume. **Place in nursery** keeps the current experiment active. Placement adds external material and starts copies at rest, as in the existing collection system.

## Verification

`npm run build` checks TypeScript and bundles the app. `npm run check:nursery-ui` uses fake world objects to check parent preservation, return and resume, recovery after a reload, blocked replacement, storage failure handling and guide routing. It never loads WASM, renders, or executes a physical step. Expanded template and control-reference checks cover the new drawers and guide. No simulation, browser interaction or performance benchmark was run for this change; interactive evaluation is left to the user.
