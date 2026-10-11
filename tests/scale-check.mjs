import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const base={seed:8042,kind:'tidal',name:'Scale check',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,initialCells:256,capacity:8192,origin:'seeded',physicalAssay:'full'};
const results=[];
for(const initialCells of [256,1536,4096,8192]) {
 const world=new World(JSON.stringify({...base,initialCells,capacity:64000}));world.start_physical();
 let state=JSON.parse(world.save());assert.equal(state.autonomous.cells.length,initialCells);
 assert.equal(JSON.parse(world.metadata(0)).stats.cellBudget,64000);
 const start=performance.now();world.step(8);const stepMs=(performance.now()-start)/8;
 const meta=JSON.parse(world.metadata(0));assert.ok(meta.autonomous.physics);
 for(const k of ['massError','mineralError','energyError'])assert.ok(Math.abs(meta.autonomous.physics[k])<1e-7);
 const saved=world.save();world.load(saved);assert.equal(saved,world.save());
 const savedBio=JSON.stringify(JSON.parse(saved).autonomous);
 world.set_cell_budget(64000);assert.equal(savedBio,JSON.stringify(JSON.parse(world.save()).autonomous));
 assert.throws(()=>world.set_cell_budget(initialCells-1));assert.throws(()=>world.set_cell_budget(64001));
 const before=world.save();const display=world.display_world();
 assert.ok([...display.render_data(),...display.cell_data(),...display.link_data(),...display.joint_data()].every(Number.isFinite));
 assert.equal(display.ids().length,meta.stats.population);assert.equal(world.save(),before);display.free();
 const clone=new World(JSON.stringify(base));clone.load(world.save());world.step(4);clone.step(4);assert.equal(world.save(),clone.save());clone.free();
 results.push({initialCells,ceiling:64000,cycles:3,finalCells:JSON.parse(world.metadata(0)).stats.cells,stepMs,actualSpeed:.25/(stepMs/1000)});
 world.free();console.log(JSON.stringify(results.at(-1)));
}
// Same engine, chemistry and inheritance in passive/full; only actuation differs.
for(const physicalAssay of ['full','passive','scarcity']){
 const w=new World(JSON.stringify({...base,physicalAssay}));w.start_physical();w.step(4);
 const meta=JSON.parse(w.metadata(0));assert.ok(meta.autonomous.physics);assert.equal(meta.stats.cellBudget,8192);
 assert.equal(meta.autonomous.physics.assay,physicalAssay==='passive'?'passive':'full');w.free();
}
// An older physical save must still restore its exact chemical/genome state.
const older=await readFile(new URL('../examples/physical-world.json',import.meta.url),'utf8');
const migrated=new World(JSON.stringify(base));migrated.load(older);const old=JSON.parse(older),now=JSON.parse(migrated.save());
assert.equal(now.autonomous.cells.length,old.autonomous.cells.length);assert.deepEqual(now.autonomous.physical.solutes.map(s=>s.slice(0,5)),old.autonomous.physical.solutes.map(s=>s.slice(0,5)));
assert.ok(now.autonomous.physical.solutes.every(s=>s.length===8&&s[5]===0));
migrated.free();
await writeFile(new URL('../docs/scale-results.json',import.meta.url),JSON.stringify({protocol:'Short eight-tick throughput samples plus four-tick exact continuation; all use a 64000 cell budget. Upper capacity is a guardrail, not a long-run 64000-cell performance claim.',results},null,2)+'\n');
console.log('Scale WASM checks passed: configurable founders, ceiling, conservation, budget-only edits, fixed-state rendering, exact continuation, shared assays and old saves.');
// A sharply folded deposited contact with no contractile material must not
// spend ATP on a hidden minimum-angle target or report motor activity.
const sharp=new World(JSON.stringify({...base,initialCells:3}));sharp.start_physical();
const fixture=JSON.parse(sharp.save());
fixture.autonomous.cells.forEach((c,i)=>{c.p=[{x:0,y:5,z:0},{x:.5,y:5,z:0},{x:0,y:5,z:.0001}][i];c.material.mineral=.08;});
fixture.autonomous.physical.initial_mineral+=.24;
fixture.autonomous.bonds=[{a:1,b:2,rest:.5,integrity:1,activation:0,stiffness:0},{a:2,b:3,rest:.5,integrity:1,activation:0,stiffness:0}];
sharp.load(JSON.stringify(fixture));sharp.step(1);assert.equal(JSON.parse(sharp.save()).autonomous.motor_work,0);sharp.free();
console.log('No-material sharp-angle regression passed in the distributed WASM.');
