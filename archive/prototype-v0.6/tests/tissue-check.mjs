import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'The tidal garden',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:1000,origin:'seeded'};
const results=[];let best=0,checkpoint='';
for(const seed of [8042,1197,6219]){
  const w=new World(JSON.stringify({...config,seed}));w.start_tissue_study();
  const initial=JSON.parse(w.save());assert.equal(initial.study,true);
  assert.equal(initial.organisms.length,80);assert.ok(initial.organisms.every(o=>o.cells===1));
  let largest=1,changes=0,repair=0,motors=false,support=false;
  for(let sample=0;sample<12;sample++){
    w.step(40);const saved=w.save(),state=JSON.parse(saved);
    for(const o of state.organisms){
      largest=Math.max(largest,o.cells);changes=Math.max(changes,o.body.differentiations);repair=Math.max(repair,o.body.repair_work);
      motors ||= o.body.links.some(l=>l.kind===2)||o.body.joints.some(j=>j.enabled&&j.work>0);support ||= o.body.links.some(l=>l.kind===1);
      if(o.body.regulated){
        const total=o.body.nodes.reduce((sum,n)=>sum+n.reserve,0);
        assert.ok(Math.abs(total-o.energy)<.005,'cell stores must match the total energy budget');
      }
      if(seed===8042&&o.lineage===9&&o.cells>best){best=o.cells;checkpoint=saved;}
    }
    w.load(saved);assert.equal(w.save(),saved);
  }
  const meta=JSON.parse(w.metadata(0));assert.ok(largest>=4);assert.ok(changes>0);assert.ok(support);assert.ok(meta.stats.births>0);
  const result={seed,cycles:meta.stats.time,largestSampledBody:largest,maxSampledFateChanges:changes,maxSampledRepairEnergy:repair,muscleObserved:motors,supportObserved:support,births:meta.stats.births,feedings:meta.stats.feedings};
  results.push(result);console.log(JSON.stringify(result));w.free();
}
assert.ok(checkpoint);
await writeFile(new URL('../examples/tissue-study.json',import.meta.url),checkpoint+'\n');
const a=new World(JSON.stringify(config)),b=new World(JSON.stringify(config));a.load(checkpoint);b.load(checkpoint);a.step(37);b.step(37);assert.equal(a.save(),b.save());
const healthy=JSON.parse(a.save());const o=healthy.organisms.find(o=>o.cells>=4&&o.regulation);assert.ok(o);
o.body.nodes[0].health=-1;assert.throws(()=>b.load(JSON.stringify(healthy)));
// Existing physical save imports stay in their explicitly simpler tissue mode.
const legacy=JSON.parse(await readFile(new URL('../examples/tidal-world-100.json',import.meta.url),'utf8'));b.load(JSON.stringify(legacy));
assert.equal(JSON.parse(b.save()).version,4);assert.ok(JSON.parse(b.save()).organisms.every(o=>o.regulation===null));
await writeFile(new URL('../docs/tissue-results.json',import.meta.url),JSON.stringify({protocol:'Supplied single-cell precursor; ecosystem growth, local regulation, transport and survival rules; samples every 10 cycles.',results},null,2)+'\n');
console.log('Bundled WASM tissue studies: real growth, regulation, support, cell-energy consistency, valid sampled saves, exact continuation and legacy migration passed.');
a.free();b.free();
