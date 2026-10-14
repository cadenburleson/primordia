import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {performance} from 'node:perf_hooks';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'Physical cells',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:1000,origin:'seeded'};
const results=[];
for(const seed of [8042,1197,6219]){
 const w=new World(JSON.stringify({...config,seed}));w.start_physical();
 const first=JSON.parse(w.save());assert.equal(first.autonomous.cells.length,64);
 assert.ok(first.autonomous.cells.every(c=>c.material&&c.genome.genes.every(g=>[0,1,2,7,48].includes(g.action))));
 let maxErrors=[0,0,0],elapsed=0,steps=0,peakCells=64;
 for(let sample=0;sample<30;sample++){
  const start=performance.now();w.step(40);elapsed+=performance.now()-start;steps+=40;
  const saved=w.save(),state=JSON.parse(saved),meta=JSON.parse(w.metadata(0)),p=meta.autonomous.physics;
  assert.ok(p&&p.flow.length===400&&p.flow.every(v=>v.every(Number.isFinite)));
  const errors=[p.massError,p.mineralError,p.energyError];
  errors.forEach((v,i)=>{assert.ok(Math.abs(v)<1e-7,`Conservation ${seed}, ${state.time}: ${errors}`);maxErrors[i]=Math.max(maxErrors[i],Math.abs(v));});
  peakCells=Math.max(peakCells,state.autonomous.cells.length);
  assert.ok(state.autonomous.cells.every(c=>c.material.biomass>0&&c.material.proteins.every(v=>v>=0&&Number.isFinite(v))));
  const cells=w.cell_data(),links=w.link_data();assert.equal(cells.length%14,0);assert.equal(links.length%6,0);
  assert.ok([...cells,...links].every(Number.isFinite));
  for(let i=0;i<links.length;i+=6){assert.ok(links[i]<cells.length/14&&links[i+1]<cells.length/14);}
  w.load(saved);assert.equal(w.save(),saved);
  if(sample%10===0)console.log(JSON.stringify({seed,cycle:state.time,cells:state.autonomous.cells.length,divisions:state.births,peakBody:meta.autonomous.peakBody,maxErrors}));
 }
 const meta=JSON.parse(w.metadata(0));
 assert.ok(meta.autonomous.divisions>0&&meta.autonomous.innovations>0);
 const copy=new World(JSON.stringify(config));copy.load(w.save());w.step(12);copy.step(12);assert.equal(w.save(),copy.save());
 const bad=JSON.parse(w.save());bad.autonomous.physical.solutes[0][0]+=1;assert.throws(()=>copy.load(JSON.stringify(bad)));
 const result={seed,cycles:meta.stats.time,cells:meta.stats.cells,peakCells,bodies:meta.stats.population,divisions:meta.autonomous.divisions,
  peakGeneration:meta.autonomous.peakGeneration,peakBody:meta.autonomous.peakBody,genomes:meta.autonomous.genomes,
  innovations:meta.autonomous.innovations,meals:meta.stats.feedings,motorWork:meta.autonomous.motorWork,
  physics:{...meta.autonomous.physics,flow:undefined},maxErrors,meanStepMs:elapsed/steps};
 results.push(result);console.log(JSON.stringify(result));
 if(seed===8042){await mkdir(new URL('../examples/',import.meta.url),{recursive:true});await writeFile(new URL('../examples/physical-world.json',import.meta.url),w.save()+'\n');}
 w.free();copy.free();
}
await writeFile(new URL('../docs/physical-results.json',import.meta.url),JSON.stringify({protocol:'64 identical supplied replicators. Finite initial pools, tracked light input, no resource renewal. 300 cycles per seed; 30 conservation/save/render samples per run. Exact continuation. Model units, not calibrated real chemistry.',results},null,2)+'\n');
console.log('Physical WASM checks passed: chemistry accounting, finite material stores, variable inherited genomes, render buffers, exact continuation, and corruption rejection. No organ or walking claim.');
