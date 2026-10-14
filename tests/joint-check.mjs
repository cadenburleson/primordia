import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'The tidal garden',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:1000,origin:'seeded'};
const results=[];
for(const seed of [8042,1197,6219]){
  const world=new World(JSON.stringify({...config,seed}));world.start_joint_lab();
  const first=JSON.parse(world.save());assert.equal(first.organisms[0].cells,1);assert.ok(first.lab.jointed);
  // Save/resume during the one-cell physical phase, before any joint exists.
  world.step(1);const resumed=new World(JSON.stringify(config));resumed.load(world.save());world.step(79);resumed.step(79);assert.equal(world.save(),resumed.save());
  const baseline=JSON.parse(world.metadata(1)).lab.baseline;
  assert.ok(baseline.viable && baseline.work>0 && baseline.distance>.1 && baseline.joints===8);
  assert.ok(baseline.joint_error<.15);
  const passive=new World(JSON.stringify({...config,seed}));passive.start_joint_lab();
  const quiet=JSON.parse(passive.save());
  quiet.organisms[0].genes[12]=0;quiet.lab.program.genes[12]=0;quiet.lab.elite.genes[12]=0;
  passive.load(JSON.stringify(quiet));passive.step(80);const inactive=JSON.parse(passive.metadata(1)).lab.baseline;
  assert.equal(inactive.work,0);assert.ok(baseline.distance>inactive.distance+.1);
  const neutral=new World(JSON.stringify({...config,seed}));neutral.start_joint_lab();const staticControl=JSON.parse(neutral.save());
  for(const program of [staticControl.organisms[0],staticControl.lab.program,staticControl.lab.elite])program.anatomy.network=Array.from({length:2},()=>Array(8).fill(.5));
  neutral.load(JSON.stringify(staticControl));neutral.step(80);const uncontrolled=JSON.parse(neutral.metadata(1)).lab.baseline;
  assert.ok(baseline.distance>uncontrolled.distance+.1);
  for(let round=0;round<24;round++)world.step(400);
  const view=JSON.parse(world.metadata(1));assert.ok(view.lab.generation>=10);
  assert.ok(view.lab.champion.viable && view.lab.champion.score>baseline.score+.15);
  assert.ok(view.lab.progress.every((p,i,all)=>!i||p.score+1e-5>=all[i-1].score));
  const result={seed,baseline,inactive,constantController:uncontrolled,best:view.lab.champion,generations:view.lab.generation};
  results.push(result);console.log(JSON.stringify(result));
  resumed.load(world.save());world.step(37);resumed.step(37);assert.equal(world.save(),resumed.save());assert.deepEqual(world.joint_data(),resumed.joint_data());
  world.replay_best(true);assert.equal(JSON.parse(world.metadata(1)).selected.cells,1);world.step(52);
  const selected=JSON.parse(world.metadata(1)).selected;assert.equal(selected.cells,12);assert.equal(selected.body.joints.length,8);
  const packed=world.joint_data(),cellCount=world.cell_data().length/14;assert.equal(packed.length,8*9);
  for(let i=0;i<packed.length;i+=9){for(let k=0;k<3;k++)assert.ok(packed[i+k]>=0 && packed[i+k]<cellCount);assert.ok(packed[i+3]<packed[i+4]);}
  if(seed===8042)await writeFile(new URL('../examples/jointed-lab.json',import.meta.url),world.save()+'\n');
  const malformed=JSON.parse(world.save());malformed.organisms[0].body.joints[0].segments[0]=999;assert.throws(()=>resumed.load(JSON.stringify(malformed)));
  const invalidWeights=JSON.parse(world.save());invalidWeights.lab.program.anatomy.network[0][0]=2;assert.throws(()=>resumed.load(JSON.stringify(invalidWeights)));
  const count=JSON.parse(world.metadata(1)).lab.results.length;world.set_motors_enabled(false);world.step(100);
  assert.equal(JSON.parse(world.metadata(1)).lab.results.length,count);assert.equal(JSON.parse(world.metadata(1)).selected.body.work,0);
  resumed.load(world.save());world.step(13);resumed.step(13);assert.equal(world.save(),resumed.save());
  world.set_motors_enabled(true);world.replay_best(false);assert.ok(JSON.parse(world.metadata(1)).lab.motorsEnabled);
  for(const w of [world,resumed,passive,neutral])w.free();
}
const old=new World(JSON.stringify(config));old.load(await readFile(new URL('./fixtures/tissue-v3.json',import.meta.url),'utf8'));
let state=JSON.parse(old.save());assert.equal(state.version,4);assert.ok(state.organisms.every(o=>o.anatomy===null && o.body.joints.length===0));
old.step(17);state=old.save();old.load(state);assert.equal(old.save(),state);old.free();
await writeFile(new URL('../docs/joint-results.json',import.meta.url),JSON.stringify({protocol:'Supplied single-cell precursors, flat contact assay; growth/settling excluded from measured travel. Paired angular motors and bounded inherited recurrent controllers. Ten generations, three seeds, motor and constant-controller comparisons.',results},null,2)+'\n');
console.log('Bundled WASM joints: development, bounded angles, motor/controller comparisons, replicated selection, replay ablation, valid packed buffers, exact continuation and v3 migration passed.');
