import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('..', import.meta.url));
const targetDir = process.env.CARGO_TARGET_DIR || `${root}/engine/target`;
const cli = process.env.WASM_BINDGEN || (existsSync(`${root}/.tools/wasm-bindgen`) ? `${root}/.tools/wasm-bindgen` : 'wasm-bindgen');
execFileSync('cargo', ['build', '--release', '--target', 'wasm32-unknown-unknown', '--manifest-path', 'engine/Cargo.toml'], { cwd: root, stdio: 'inherit' });
execFileSync(cli, [`${targetDir}/wasm32-unknown-unknown/release/primordia_engine.wasm`, '--target', 'web', '--out-dir', 'src/wasm', '--out-name', 'primordia_engine'], { cwd: root, stdio: 'inherit' });
