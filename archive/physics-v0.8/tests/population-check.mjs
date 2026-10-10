import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'The tidal garden',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:1000,origin:'seeded'};
const report=[];
for(const seed of [8042,1197,6219]){
 const w=new World(JSON.stringify({...config,seed}));w.start_population(48);
 let state=JSON.parse(w.save());assert.equal(state.lab,null);assert.equal(state.study,false);
 assert.equal(state.organisms.length,48);assert.ok(state.organisms.every(o=>o.cells===1&&o.parent===0&&o.anatomy&&o.regulation));
 assert.equal(state.competition.families.length,6);
 let maxActiveBodies=0,foodPoorFraction=0,checkpoint='',checkpointBodies=0;
 for(let sample=0;sample<20;sample++){
  w.step(40);const saved=w.save();state=JSON.parse(saved);const meta=JSON.parse(w.metadata(0));
  assert.ok(state.organisms.length<=1000);
  assert.equal(state.competition.families.reduce((s,f)=>s+f.births,0),state.births);
  assert.equal(state.competition.families.reduce((s,f)=>s+f.meals,0),state.feedings);
  assert.equal(state.competition.families.reduce((s,f)=>s+f.exchanges,0),state.mating);
  assert.equal(meta.competition.families.reduce((s,f)=>s+f.living,0),state.organisms.length);
  const active=state.organisms.filter(o=>o.body.joints.some(j=>j.enabled&&j.work>0)).length;
  maxActiveBodies=Math.max(maxActiveBodies,active);foodPoorFraction=Math.max(foodPoorFraction,meta.competition.foodPoorFraction);
  for(const o of state.organisms){assert.ok(o.anatomy&&o.family>0);if(o.body.regulated){assert.ok(Math.abs(o.body.nodes.reduce((s,n)=>s+n.reserve,0)-o.energy)<.01);}}
  if(seed===8042&&active>checkpointBodies){checkpoint=saved;checkpointBodies=active;}
  w.load(saved);assert.equal(w.save(),saved);
  if(sample%5===0)console.log(JSON.stringify({seed,cycles:state.time,population:state.organisms.length,births:state.births,mating:state.mating,feedings:state.feedings,active}));
 }
 const meta=JSON.parse(w.metadata(0));
 const result={seed,cycles:meta.stats.time,survivors:meta.stats.population,cells:meta.stats.cells,births:meta.stats.births,mating:meta.stats.mating,feedings:meta.stats.feedings,generation:meta.competition.peakGeneration,peakJointed:meta.competition.peakJointed,largestBody:meta.competition.largestBody,maxActiveBodies,foodPoorFraction,families:meta.competition.families};
 console.log(JSON.stringify(result));report.push(result);
 assert.ok(result.births>0);assert.ok(result.mating>0);assert.ok(result.generation>=2);assert.ok(result.peakJointed>=2);assert.ok(result.maxActiveBodies>=2);
 const resumed=new World(JSON.stringify(config));resumed.load(w.save());w.step(17);resumed.step(17);assert.equal(w.save(),resumed.save());assert.deepEqual(w.joint_data(),resumed.joint_data());
 const invalid=JSON.parse(w.save());invalid.organisms[0].family=99999;assert.throws(()=>resumed.load(JSON.stringify(invalid)));
 const before=JSON.parse(w.save());w.seed_competitors(12);const after=JSON.parse(w.save());
 assert.equal(after.organisms.length,Math.min(before.organisms.length+12,1000));assert.ok(after.organisms.slice(before.organisms.length).every(o=>o.cells===1&&o.parent===0));
 w.load(JSON.stringify(after));
 if(seed===8042&&checkpoint)await writeFile(new URL('../examples/population-ecosystem.json',import.meta.url),checkpoint+'\n');
 w.free();resumed.free();
}
assert.ok(report.every(r=>r.feedings>0));assert.ok(report.some(r=>r.foodPoorFraction>0));
await writeFile(new URL('../docs/population-results.json',import.meta.url),JSON.stringify({protocol:'48 supplied single cells, six founder biases, shared ecological resources and actual survival/reproduction; 200 cycles, three seeds. No movement-scored trials or automatic rescue.',results:report},null,2)+'\n');
console.log('Competing population WASM: multiple active jointed bodies, ecological births/mating/feeding, family accounting, ancestry, finite stores, valid saves, exact continuation and manual introductions passed.');
