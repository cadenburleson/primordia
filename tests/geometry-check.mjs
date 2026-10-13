import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const base={seed:8042,kind:'tidal',name:'Curved habitat check',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,initialCells:16,capacity:4096,origin:'seeded',physicalAssay:'full',founderSpread:'global'};
const results=[];
function inspect(w){
 const m=JSON.parse(w.metadata(0)),a=m.autonomous,p=a.physics,planet=a.planet;
 const errors=[p.massError,p.mineralError,p.energyError,planet.waterError];
 errors.forEach(e=>assert.ok(Number.isFinite(e)&&Math.abs(e)<1e-7,`Conservation: ${errors}`));
 const display=w.display_world();
 assert.ok([...display.cell_data(),...display.link_data(),...display.joint_data()].every(Number.isFinite));display.free();
 return {m,a,p,planet,errors};
}
for(const worldShape of ['planar','sphere'])for(const worldSize of [64,128,256]){
 const config={...base,worldSize,worldShape},w=new World(JSON.stringify(config));w.start_physical();
 const initial=JSON.parse(w.save()),g=initial.autonomous.physical.domain;
 assert.equal(g.grid,worldSize/1.6);assert.equal(g.shape,worldShape);
 const sites=g.grid*g.grid/(worldShape==='sphere'?2:1);
 assert.equal(w.field().length,sites);
 assert.ok(initial.autonomous.cells.every(c=>c.genome.genes.length===5));
 if(worldShape==='sphere'){
  const r=worldSize/(2*Math.PI);
  assert.ok(initial.autonomous.cells.every(c=>Math.hypot(c.p.x,c.p.y,c.p.z)>r-5));
  assert.ok(initial.autonomous.cells.some(c=>c.p.y<0)&&initial.autonomous.cells.some(c=>c.p.y>0));
 }
 const start=performance.now();w.step(24);const elapsed=performance.now()-start;
 const {m,a,p,planet,errors}=inspect(w);
 assert.equal(a.resources.detritus.length,sites);assert.equal(planet.relief.length,sites);
 assert.equal(p.domain.grid,g.grid);assert.equal(p.domain.sites,sites);
 assert.equal(w.field().length,sites);
 const save=w.save(),copy=new World(JSON.stringify(config));copy.load(save);assert.equal(copy.save(),save);
 w.step(4);copy.step(4);assert.equal(copy.save(),w.save());
 const bad=JSON.parse(save);bad.config.worldSize=worldSize===64?128:64;
 assert.throws(()=>copy.load(JSON.stringify(bad)));
 const result={worldShape,worldSize,sites,cycles:m.stats.time,cells:m.stats.cells,divisions:a.divisions,meanTickMs:elapsed/24,errors,area:p.domain.area};results.push(result);console.log(JSON.stringify(result));
 copy.free();w.free();
}
// A longer ordinary population checks the combined weather/flow/growth engine,
// not just isolated transport fixtures. No movement/anatomy genes are supplied.
const config={...base,worldShape:'sphere',worldSize:128,initialCells:64};
const w=new World(JSON.stringify(config));w.start_physical();let peakCells=64;const maxErrors=[0,0,0,0];
for(let sample=0;sample<10;sample++){
 w.step(20);const {m,a,errors}=inspect(w);peakCells=Math.max(peakCells,m.stats.cells);errors.forEach((e,k)=>maxErrors[k]=Math.max(maxErrors[k],Math.abs(e)));
 console.log(JSON.stringify({sphereCycles:m.stats.time,cells:m.stats.cells,divisions:a.divisions,generation:a.peakGeneration}));
}
const {m,a,p,planet}=inspect(w);assert.ok(a.divisions>0&&a.innovations>0);assert.ok(planet.precipitation>0&&planet.runoff>0);
const ecosystem={cycles:m.stats.time,cells:m.stats.cells,colonies:m.stats.colonies,divisions:a.divisions,peakGeneration:a.peakGeneration,innovations:a.innovations,peakCells,maxErrors,waterTransported:JSON.parse(w.save()).autonomous.physical.fluid.surface.water_transported};
await writeFile(new URL('../examples/spherical-ecosystem.json',import.meta.url),w.save()+'\n');w.free();
for(const physicalAssay of ['passive','scarcity']){
 const w=new World(JSON.stringify({...config,initialCells:8,physicalAssay}));w.start_physical();w.step(4);const {a,p}=inspect(w);assert.equal(p.domain.shape,'sphere');if(physicalAssay==='passive')assert.equal(a.motorWork,0);w.free();
}
// Existing ecosystem files have no domain or sun direction. Migration must retain
// their inventories and their original 64-unit planar geometry.
const old=JSON.parse(await readFile(new URL('../examples/natural-ecosystem-8042.json',import.meta.url),'utf8'));
const migrated=new World(JSON.stringify(base));migrated.load(JSON.stringify(old));
const state=JSON.parse(migrated.save());assert.equal(state.autonomous.physical.domain.shape,'planar');assert.equal(state.autonomous.physical.domain.grid,40);
assert.deepEqual(state.autonomous.cells,old.autonomous.cells);assert.deepEqual(state.autonomous.physical.solutes,old.autonomous.physical.solutes);
migrated.step(4);inspect(migrated);migrated.free();
await writeFile(new URL('../docs/geometry-results.json',import.meta.url),JSON.stringify({protocol:'Six size/shape populations of 16 ordinary founders for six cycles; finite buffers, four chemical/water budgets, geometry dimensions, exact save continuation and invalid geometry rejection. A 64-founder spherical population for 50 cycles, shared assays and pre-globe ecosystem migration. Native tests separately verify seam advection, polar diffusion, hydrostatic rest and radial gravity. No organ, intelligence or directed movement outcome asserted.',results,ecosystem},null,2)+'\n');
console.log('Curved habitat WASM checks passed.');
