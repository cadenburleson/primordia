import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const conditions=[
  {seed:8042,kind:'tidal',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09},
  {seed:1197,kind:'volcanic',sunlight:.2,nutrients:.55,minerals:.94,temperature:.81,radiation:.32,solvent:.65,mutation:.11},
  {seed:6219,kind:'alien',sunlight:.42,nutrients:.45,minerals:.74,temperature:.25,radiation:.64,solvent:.63,mutation:.14},
];
const report=[];
let mostComplex=0;
for(const config of conditions){
  const a=new World(JSON.stringify({...config,name:config.kind,capacity:1000,origin:'seeded'}));
  const start=performance.now();
  let largest=1,contractile=false,support=false,work=0;
  for(let sample=0;sample<30;sample++){
    a.step(400);
    const saved=a.save(),state=JSON.parse(saved);
    for(const o of state.organisms){
      assert.ok(Number.isFinite(o.energy));
      largest=Math.max(largest,o.cells);
      contractile ||= o.body.links.some(l=>l.kind===2)||o.body.joints.some(j=>j.enabled&&j.work>0);
      support ||= o.body.links.some(l=>l.kind===1);
      work=Math.max(work,o.body.work);
      if(o.cells>mostComplex){
        mostComplex=o.cells;
        await writeFile(new URL('../examples/evolved-ecosystem.json',import.meta.url),saved+'\n');
      }
    }
    // Import validates numeric state, per-body graphs and indices every 100 cycles.
    a.load(saved);assert.equal(a.save(),saved);
  }
  const {stats}=JSON.parse(a.metadata(0));
  assert.ok(stats.births>0);assert.ok(stats.population<=1000);
  const result={world:config.kind,seed:config.seed,cycles:stats.time,survivors:stats.population,births:stats.births,feedings:stats.feedings,mating:stats.mating,largestSampledBody:largest,contractileObserved:contractile,supportObserved:support,maxSampledWork:work,elapsedMs:Math.round(performance.now()-start)};
  report.push(result);console.log(JSON.stringify(result));a.free();
}
assert.ok(report.some(r=>r.feedings>0),'ordinary founders should produce actual ingestion events');
await writeFile(new URL('../docs/ecology-soak.json',import.meta.url),JSON.stringify(report,null,2)+'\n');
console.log('Three distributed-WASM ecological runs passed; all sampled saves remained valid. Example genomes and body counts were not edited.');
