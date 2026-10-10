import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const base={seed:8042,kind:'tidal',name:'Organism/ecosystem check',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,initialCells:64,capacity:4096,origin:'seeded',physicalAssay:'full'};
const results=[];
for(const [seed,temperature,kind] of [[8042,.48,'tidal'],[6219,.22,'alien'],[1197,.75,'volcanic']]){
 const w=new World(JSON.stringify({...base,seed,temperature,kind}));w.start_physical();
 const founder=JSON.parse(w.save());
 assert.equal(founder.autonomous.cells.length,64);
 assert.ok(founder.autonomous.cells.every(c=>c.material.proteins.length===38&&c.material.lipid===0&&c.genome.genes.every(g=>[0,1,2,7].includes(g.action))));
 const maxErrors=[0,0,0,0];let peakCells=64,peakMixed=0,peakJoints=0,peakStorage=0,peakExcited=0,elapsed=0;
 for(let sample=0;sample<25;sample++){
  const start=performance.now();w.step(40);elapsed+=performance.now()-start;
  const meta=JSON.parse(w.metadata(0)),a=meta.autonomous,p=a.physics,planet=a.planet;
  const errors=[p.massError,p.mineralError,p.energyError,planet.waterError];
  errors.forEach((e,k)=>{assert.ok(Number.isFinite(e)&&Math.abs(e)<1e-7,`Conservation ${seed}/${meta.stats.time}: ${errors}`);maxErrors[k]=Math.max(maxErrors[k],Math.abs(e));});
  assert.equal(a.actions.length,38);assert.equal(a.resources.detritus.length,1600);assert.equal(planet.relief.length,1600);
  assert.ok(a.resources.detritus.every(v=>Number.isFinite(v)&&v>=0));
  assert.ok(a.physiology.ledger.excreted>0);
  peakCells=Math.max(peakCells,meta.stats.cells);peakMixed=Math.max(peakMixed,a.physiology.mixedBodies);peakJoints=Math.max(peakJoints,p.joints);peakStorage=Math.max(peakStorage,a.physiology.lipid);peakExcited=Math.max(peakExcited,a.physiology.excited);
  const saved=w.save();w.load(saved);assert.equal(w.save(),saved);
  const display=w.display_world();const data=display.cell_data(),links=display.link_data(),joints=display.joint_data();
  assert.equal(data.length/9,meta.stats.cells);assert.ok([...data,...links,...joints].every(Number.isFinite));
  for(let k=0;k<links.length;k+=6)assert.ok(links[k]<data.length/9&&links[k+1]<data.length/9);
  assert.equal(w.save(),saved);display.free();
  if(sample%5===0)console.log(JSON.stringify({seed,cycles:meta.stats.time,cells:meta.stats.cells,divisions:a.divisions,weather:planet.weather,eruptions:planet.eruptions,mixed:a.physiology.mixedBodies}));
 }
 const meta=JSON.parse(w.metadata(0)),a=meta.autonomous;
 assert.ok(a.divisions>0&&a.innovations>0);
 assert.ok(a.planet.precipitation>0&&a.planet.runoff>0);
 const result={seed,temperature,kind,cycles:meta.stats.time,cells:meta.stats.cells,bodies:meta.stats.population,colonies:meta.stats.colonies,divisions:a.divisions,generation:a.peakGeneration,genomes:a.genomes,innovations:a.innovations,peakCells,peakMixed,peakJoints,peakStorage,peakExcited,detritus:a.detritus,physiology:a.physiology,planet:{...a.planet,relief:undefined,snowCover:undefined,heat:undefined},maxErrors,meanTickMs:elapsed/1000};
 results.push(result);console.log(JSON.stringify({...result,physiology:undefined,planet:undefined}));
 if(seed===8042)await writeFile(new URL('../examples/natural-ecosystem-8042.json',import.meta.url),w.save()+'\n');
 const copy=new World(JSON.stringify(base));copy.load(w.save());w.step(8);copy.step(8);assert.equal(w.save(),copy.save());
 for(const corrupt of [s=>s.autonomous.cells[0].state.excitation=2,s=>s.autonomous.physical.planet.water[0]+=1,s=>s.autonomous.physical.solutes[0][5]+=1]){
  const bad=JSON.parse(w.save());corrupt(bad);assert.throws(()=>copy.load(JSON.stringify(bad)));
 }
 copy.free();w.free();
}
// The two shared assays run the same new vocabulary and planetary/resource engine.
for(const physicalAssay of ['passive','scarcity']){const w=new World(JSON.stringify({...base,physicalAssay}));w.start_physical();w.step(4);const a=JSON.parse(w.metadata(0)).autonomous;assert.ok(a.planet&&a.physiology&&a.resources);if(physicalAssay==='passive')assert.equal(a.motorWork,0);w.free();}
// Existing physical cells and molecules migrate; no new specialist genes/stores.
const old=JSON.parse(await readFile(new URL('../examples/physical-world.json',import.meta.url),'utf8'));
const migrated=new World(JSON.stringify(base));migrated.load(JSON.stringify(old));const state=JSON.parse(migrated.save());
assert.equal(state.autonomous.cells.length,old.autonomous.cells.length);
state.autonomous.cells.forEach((c,i)=>{assert.equal(c.material.lipid,0);assert.deepEqual(c.material.proteins.slice(0,10),old.autonomous.cells[i].material.proteins);assert.ok(c.material.proteins.slice(10).every(x=>x===0));assert.equal(c.genome.genes.length,old.autonomous.cells[i].genome.genes.length);});
assert.deepEqual(state.autonomous.physical.solutes.map(s=>s.slice(0,5)),old.autonomous.physical.solutes);
migrated.step(4);assert.ok(JSON.parse(migrated.metadata(0)).autonomous.planet);migrated.free();
await writeFile(new URL('../docs/organisms-results.json',import.meta.url),JSON.stringify({protocol:'Three populations of 64 ordinary four-gene founders; 250 cycles each, unchanged mutation protocol, autonomous planetary forcing. Conservation, finite render buffers, no mutation during observation, exact continuation, shared assays and old-save migration. Observed functions are not claims of organs, gait, intelligence or stable trophic specialization.',results},null,2)+'\n');
console.log('Organism/ecosystem WASM checks passed. Mechanisms permit functional interaction; no guaranteed organ/locomotion outcome was asserted.');
