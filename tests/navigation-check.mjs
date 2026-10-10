import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'Navigation comparison',sunlight:.72,nutrients:.45,minerals:.65,temperature:.48,radiation:.08,solvent:1,mutation:.09,capacity:1000,origin:'seeded'};
const results=[];
for(const seed of [8042,1197,6219]){
 const source=new World(JSON.stringify({...config,seed}));source.start_population(48);
 const initial=source.save();source.free();
 const auditOn=new World(JSON.stringify(config)),auditOff=new World(JSON.stringify(config));
 auditOn.load(initial);auditOff.load(initial);auditOff.set_directional_sensing(false);
 const before=JSON.parse(auditOff.save());assert.deepEqual(before.organisms,JSON.parse(initial).organisms);
 auditOn.step(1);auditOff.step(1);
 const firstOn=JSON.parse(auditOn.save()),firstOff=JSON.parse(auditOff.save());
 assert.deepEqual(firstOn.organisms.map(o=>o.energy),firstOff.organisms.map(o=>o.energy),'cue removal preserves first-update maintenance and uptake');
 assert.deepEqual(firstOn.field,firstOff.field);assert.equal(firstOn.competition.food_energy,firstOff.competition.food_energy);
 auditOn.free();auditOff.free();
 const runs=[];
 for(const enabled of [true,false]){
  const w=new World(JSON.stringify(config));w.load(initial);w.set_directional_sensing(enabled);
  let previous=0;
  for(let sample=0;sample<12;sample++){
   w.step(40);const saved=w.save();const m=JSON.parse(w.metadata(0));
   assert.equal(m.competition.directionalSensing,enabled);
   assert.ok(Number.isFinite(m.competition.foodEnergy)&&m.competition.foodEnergy>=previous);
   previous=m.competition.foodEnergy;w.load(saved);assert.equal(w.save(),saved);
  }
  const m=JSON.parse(w.metadata(0));runs.push({directionalSensing:enabled,cycles:m.stats.time,foodEnergy:m.competition.foodEnergy,births:m.stats.births,mating:m.stats.mating,feedings:m.stats.feedings,survivors:m.stats.population,cells:m.stats.cells,generation:m.competition.peakGeneration});
  const copy=new World(JSON.stringify(config));copy.load(w.save());w.step(8);copy.step(8);assert.equal(w.save(),copy.save());copy.free();
  const bad=JSON.parse(w.save());bad.competition.food_energy=-1;assert.throws(()=>w.load(JSON.stringify(bad)));
  // Additive fields preserve previously exported population checkpoints.
  const legacy=JSON.parse(initial);delete legacy.competition.directional_sensing;delete legacy.competition.food_energy;w.load(JSON.stringify(legacy));const old=JSON.parse(w.metadata(0));assert.equal(old.competition.directionalSensing,true);assert.equal(old.competition.foodEnergy,0);
  w.free();
 }
 assert.notEqual(runs[0].foodEnergy,runs[1].foodEnergy,'directional cue removal must affect ecological trajectories');
 const row={seed,runs,foodEnergyDelta:runs[0].foodEnergy-runs[1].foodEnergy,birthDelta:runs[0].births-runs[1].births};results.push(row);console.log(JSON.stringify(row));
}
await writeFile(new URL('../docs/navigation-results.json',import.meta.url),JSON.stringify({protocol:'Paired identical 48-single-cell founder saves, 120 cycles, three seeds; directional food/prey/mate cues on versus off. Genotypes, sensing maintenance costs, random steering, local tissue regulation and mechanical contact/angle feedback are retained. No optimizer or reward score. Total energy from environmental uptake excludes ingestion.',results},null,2)+'\n');
console.log('Paired navigation comparisons, cumulative uptake, valid sampled saves, exact continuation, corruption rejection and additive migration passed. Outcomes do not imply evolved controllers outperform founders.');
