/** Shipped-WASM check for creator powers: the UI depends on this exact API. */
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, { World } from '../src/wasm/primordia_engine.js';
await init({ module_or_path: await readFile(new URL('../src/wasm/primordia_engine_bg.wasm', import.meta.url)) });
const config={seed:8042,kind:'tidal',name:'The tidal garden',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:4096,initialCells:64,worldSize:128,worldShape:'sphere',founderSpread:'local',physicalAssay:'full',resourceLayout:'patchy',origin:'seeded'};
const steering=w=>JSON.parse(w.metadata(0)).autonomous.steering;
const send=(w,action)=>w.steer(JSON.stringify(action));
const plain=new World(JSON.stringify(config)),steered=new World(JSON.stringify(config));plain.start_physical();steered.start_physical();
// Untouched dials leave the world exactly as it was.
const start=steering(plain);
assert.deepEqual([start.mutation,start.light,start.vents,start.mixing,start.goal,start.favoured],[1,1,1,1,'',0]);assert.equal(start.diet.length,7);assert.equal(start.captures,0);
assert.equal(start.ladder.length,24);assert.equal(start.kinds.length,4);assert.deepEqual(start.ladder.map(r=>r.key).slice(0,3),['cells','colonies','bodies']);
// Every command the Powers panel can send is accepted; bad ones are refused and change nothing.
for(const action of [{kind:'light',scale:6},{kind:'vents',scale:4},{kind:'mixing',scale:3},{kind:'sharing',scale:4},{kind:'seed',species:'hunter',count:6,sustain:true},{kind:'sustain',on:false},{kind:'goal',goal:'swimming',strength:.25,period:30},{kind:'goal',goal:'defence',strength:.4,period:20},{kind:'mutation',scale:3},{kind:'goal',goal:'movement',strength:.25,period:30},{kind:'storm',scale:8,duration:60},{kind:'nourish',amount:.6}])assert.equal(typeof send(steered,action),'string');
for(const bad of [{kind:'light',scale:99},{kind:'goal',goal:'wings',strength:.25,period:30},{kind:'mutation'},{kind:'unknown'}]){const before=steered.save();assert.throws(()=>send(steered,bad));assert.equal(steered.save(),before);}
// One call advances at most 400 ticks (100 cycles).
for(let i=0;i<2;i++){plain.step(400);steered.step(400);}
const a=steering(plain),b=steering(steered);
assert.equal(b.goal,'movement');assert.ok(b.rounds>=5,'breeding rounds ran');assert.equal(a.rounds,0);
assert.equal(b.light,6);assert.equal(b.mutationFactor,3);assert.equal(b.storm,0,'the storm has passed');
for(const view of [a,b]){assert.ok(view.ladder.every(r=>r.stage>=0&&r.stage<=3&&r.progress>=0&&r.progress<=1));assert.equal(view.ladder[0].stage,3);assert.equal(view.losses.length,3);}
// The gene vocabulary the UI lists: 48 abilities, 32 promoter inputs.
const meta=JSON.parse(steered.metadata(JSON.parse(steered.metadata(0)).lineages[0].representative));
assert.equal(meta.autonomous.actions.length,48);assert.ok(meta.selected.programCells.every(c=>c.expression.length===48&&c.genes.every(g=>g.promoter.length===32)));
// Favour, disasters, and exact continuation from a save.
const body=JSON.parse(steered.metadata(0)).lineages[0].representative;
send(steered,{kind:'favour',body,pressure:.03});assert.ok(steering(steered).favoured>0);
send(steered,{kind:'winter',depth:.25,duration:120});assert.throws(()=>send(steered,{kind:'winter',depth:.25,duration:120}));
send(steered,{kind:'meteor',x:0,z:0,radius:6});send(steered,{kind:'dying',fraction:.5});
const resumed=new World(JSON.stringify(config));resumed.load(steered.save());steered.step(120);resumed.step(120);
assert.equal(resumed.save(),steered.save());
send(steered,{kind:'unfavour'});assert.equal(steering(steered).favoured,0);
// Creatures: record-holders are reported, kept as snapshots, and painted back onto the map.
{
  const w=new World(JSON.stringify(config));w.start_physical();
  assert.deepEqual(JSON.parse(w.champions()),[]);
  // A young planet has no free oxygen; the player can set the air, and floods and droughts change the seas.
  assert.equal(steering(w).oxygen,0);send(w,{kind:'oxygen',level:1});assert.equal(steering(w).oxygen,1);
  assert.throws(()=>send(w,{kind:'oxygen',level:2}));
  // Matter cycles: the books name where every element is, and moving nitrogen between air and water makes none.
  const books=()=>steering(w).cycles,sum=n=>n.air+n.dissolved+n.living+n.remains;
  const b0=books();assert.equal(b0.levels.length,5);assert.equal(b0.efficiency.length,3);assert.ok(b0.carbon.dioxide>0&&b0.nitrogen.air>b0.nitrogen.dissolved&&b0.nitrogen.dissolved>0);
  send(w,{kind:'nitrogen',change:.1});const b1=books();assert.ok(b1.nitrogen.dissolved>b0.nitrogen.dissolved*1.3&&Math.abs(sum(b1.nitrogen)-sum(b0.nitrogen))<1e-6*sum(b0.nitrogen));
  send(w,{kind:'nitrogen',change:-.5});const b2=books();assert.ok(Math.abs(b2.nitrogen.dissolved-b1.nitrogen.dissolved*.5)<1e-6*b1.nitrogen.dissolved&&Math.abs(sum(b2.nitrogen)-sum(b0.nitrogen))<1e-6*sum(b0.nitrogen));
  assert.throws(()=>send(w,{kind:'nitrogen',change:0}));assert.throws(()=>send(w,{kind:'nitrogen',change:2}));
  const sea=()=>JSON.parse(w.metadata(0)).autonomous.planet.water;
  const dry=sea();send(w,{kind:'water',change:.3});const flooded=sea();assert.ok(flooded>dry*1.05);send(w,{kind:'water',change:-.5});assert.ok(Math.abs(sea()-flooded*.5)<1e-6*flooded);
  assert.throws(()=>send(w,{kind:'water',change:5}));
  const hunter=w.species_specimen('hunter');assert.equal(JSON.parse(w.inspect_specimen(hunter)).name,'Hunter');
  assert.throws(()=>w.species_specimen('dragon'));
  const before=JSON.parse(w.metadata(0)).stats.cells;
  const placed=w.paint_life(hunter,new Float32Array([4,2,7,2,10,2]),3,4);assert.ok(placed>=6&&placed<=12,`painted ${placed}`);
  assert.equal(JSON.parse(w.metadata(0)).stats.cells,before+placed);
  assert.throws(()=>w.paint_life(hunter,new Float32Array([4]),3,4));assert.throws(()=>w.paint_life('{}',new Float32Array([4,2]),3,4));
  send(w,{kind:'keep',species:'hunter'});assert.deepEqual(steering(w).sustained,['hunter']);
  // Survival is the default; an automatic goal is an optional experiment that follows the ladder.
  send(w,{kind:'mutation',scale:2});send(w,{kind:'sharing',scale:1});
  assert.equal(steering(w).goal,'');assert.equal(steering(w).auto,false);
  send(w,{kind:'goal',goal:'auto',strength:.15,period:40});assert.equal(steering(w).auto,true);assert.equal(steering(w).goal,'bodies');
  send(w,{kind:'goal',goal:'',strength:.15,period:40});assert.equal(steering(w).auto,false);
  w.step(400);w.step(400);w.step(400);
  const best=JSON.parse(w.champions());assert.ok(best.length>0&&best.every(c=>c.body>0&&c.cells>0&&c.score>0&&c.note));
  const kept=w.capture_specimen(best[0].body,best[0].title);assert.equal(JSON.parse(w.inspect_specimen(kept)).cells,best[0].cells);
  const chosen=JSON.parse(w.metadata(best[0].body)).selected,bearing=chosen.bearing;assert.ok(bearing.submerged>=0&&bearing.submerged<=1&&Number.isFinite(bearing.weight));
  assert.ok(Number.isFinite(bearing.load)&&bearing.dry>=0&&bearing.dry<=1&&typeof chosen.feeding.level==='string'&&chosen.feeding.steps>=0);
  w.free();
}
console.log('Creator powers passed on shipped WASM: defaults, every Powers command, refused input, breeding rounds, ladder, favour, disasters, creatures and exact continuation.');
console.log(`After ${JSON.parse(steered.metadata(0)).stats.time} cycles: ${b.rounds} breeding rounds, ${b.culled} cells removed; ladder stages ${b.ladder.map(r=>r.stage).join('')} (unsteered ${a.ladder.map(r=>r.stage).join('')}).`);
plain.free();steered.free();resumed.free();
