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
assert.equal(start.ladder.length,19);assert.deepEqual(start.ladder.map(r=>r.key).slice(0,3),['cells','colonies','bodies']);
// Every command the Powers panel can send is accepted; bad ones are refused and change nothing.
for(const action of [{kind:'light',scale:6},{kind:'vents',scale:4},{kind:'mixing',scale:3},{kind:'goal',goal:'swimming',strength:.25,period:30},{kind:'goal',goal:'defence',strength:.4,period:20},{kind:'mutation',scale:3},{kind:'goal',goal:'movement',strength:.25,period:30},{kind:'storm',scale:8,duration:60},{kind:'nourish',amount:.6}])assert.equal(typeof send(steered,action),'string');
for(const bad of [{kind:'light',scale:99},{kind:'goal',goal:'wings',strength:.25,period:30},{kind:'mutation'},{kind:'unknown'}]){const before=steered.save();assert.throws(()=>send(steered,bad));assert.equal(steered.save(),before);}
// One call advances at most 400 ticks (100 cycles).
for(let i=0;i<2;i++){plain.step(400);steered.step(400);}
const a=steering(plain),b=steering(steered);
assert.equal(b.goal,'movement');assert.ok(b.rounds>=5,'breeding rounds ran');assert.equal(a.rounds,0);
assert.equal(b.light,6);assert.equal(b.mutationFactor,3);assert.equal(b.storm,0,'the storm has passed');
for(const view of [a,b]){assert.ok(view.ladder.every(r=>r.stage>=0&&r.stage<=3&&r.progress>=0&&r.progress<=1));assert.equal(view.ladder[0].stage,3);assert.equal(view.losses.length,3);}
// The gene vocabulary the UI lists: 38 abilities, 32 promoter inputs.
const meta=JSON.parse(steered.metadata(JSON.parse(steered.metadata(0)).lineages[0].representative));
assert.equal(meta.autonomous.actions.length,38);assert.ok(meta.selected.programCells.every(c=>c.expression.length===38&&c.genes.every(g=>g.promoter.length===32)));
// Favour, disasters, and exact continuation from a save.
const body=JSON.parse(steered.metadata(0)).lineages[0].representative;
send(steered,{kind:'favour',body,pressure:.03});assert.ok(steering(steered).favoured>0);
send(steered,{kind:'winter',depth:.25,duration:120});assert.throws(()=>send(steered,{kind:'winter',depth:.25,duration:120}));
send(steered,{kind:'meteor',x:0,z:0,radius:6});send(steered,{kind:'dying',fraction:.5});
const resumed=new World(JSON.stringify(config));resumed.load(steered.save());steered.step(120);resumed.step(120);
assert.equal(resumed.save(),steered.save());
send(steered,{kind:'unfavour'});assert.equal(steering(steered).favoured,0);
console.log('Creator powers passed on shipped WASM: defaults, every Powers command, refused input, breeding rounds, ladder, favour, disasters and exact continuation.');
console.log(`After ${JSON.parse(steered.metadata(0)).stats.time} cycles: ${b.rounds} breeding rounds, ${b.culled} cells removed; ladder stages ${b.ladder.map(r=>r.stage).join('')} (unsteered ${a.ladder.map(r=>r.stage).join('')}).`);
plain.free();steered.free();resumed.free();
