# Earlier Primordia prototypes

These are preserved source snapshots from earlier development stages. They are not separate components needed by the current engine. All new work belongs in the parent directory's `src/` and `engine/`.

| Folder | Former location | Stage | Historical port |
| --- | --- | --- | --- |
| `prototype-v0.6/` | `repos/primordia` | Initial organism/evolution prototype | 5190 |
| `physics-v0.8/` | `repos/primordia-physics` | Physical cellular habitat rewrite | 5192 |
| Parent directory | `repos/primordia-organisms` | Current integrated app, v0.12 | 5194 |

## Consolidation, 2026-10-09

Source, documents, examples, built assets and build caches were moved without deleting either prototype. Source/document hashes were verified immediately after relocation. The pre-migration manifest is in `../.build-backups/consolidation-2026-10-09.json`.

The original empty Git repository was retained at the project root. No commits, branches or remote configuration were changed. There were no tracked files or commits at the time of this move.

The current app now owns a real `node_modules/` directory. The physics archive reuses it through a relative `../../node_modules` symlink; the original prototype retains its own dependencies. Archive READMEs retain historical names and paths as part of the snapshot. Their relative launchers still use their old ports.

The main development watcher ignores this archive and Rust build caches. The active static server uses port 5194, so the browser origin and its saved collection/world storage stay the same. The move did not reload the user's browser or run a simulation.
