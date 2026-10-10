/** Small controller checks. Fake worlds only: no WASM, rendering or physical steps. */
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';
const source=file=>ts.transpileModule(readFileSync(file,'utf8').replace(/^import .*;\n/gm,'').replace(/export /g,''),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
const presetsContext=vm.createContext({});vm.runInContext(source('src/sim/types.ts')+';this.presets=PRESETS;',presetsContext);
const PRESETS=JSON.parse(JSON.stringify(presetsContext.presets));
class FakeWorld {
  constructor(json){this.state={config:JSON.parse(json),time:0,marker:'original'};}
  start_physical(){this.state.marker='founders';}
  save(){return JSON.stringify(this.state);}
  load(json){this.state=JSON.parse(json);}
  free(){}
  step(){throw new Error('A simulation step must never run in this check');}
  metadata(){return JSON.stringify({config:this.state.config,stats:{time:this.state.time},autonomous:{physics:{}},selected:null});}
  display_world(){return {free(){},render_data:()=>new Float32Array(),ids:()=>new Uint32Array(),cell_data:()=>new Float32Array(),link_data:()=>new Float32Array(),joint_data:()=>new Float32Array()};}
  display_metadata(){return this.metadata();}
  field(){return new Float32Array();}
}
function controller(storage=new Map()){
  let failStore=false;const messages=[];
  const context=vm.createContext({PRESETS,World:FakeWorld,init:async()=>{},wasmUrl:'unused',performance:{now:()=>0},Float32Array,Uint32Array,self:{},postMessage:m=>messages.push(m),setTimeout(){},storeWorld:async(json,key)=>{if(failStore)throw new Error('Storage unavailable');storage.set(key,json);},readWorld:async key=>storage.get(key)??null});
  for(const file of ['src/sim/nursery-session.ts','src/sim/scheduler.ts','src/sim/assessment.ts','src/sim/worker.ts'])vm.runInContext(source(file),context);
  return {context,messages,storage,setFail:()=>{failStore=true;},async send(cmd){context.self.onmessage({data:cmd});await vm.runInContext('commands',context);},saved:()=>vm.runInContext('world.save()',context),snapshot:()=>messages.filter(m=>m.type==='snapshot').at(-1)};
}
const c=controller();const parent={...PRESETS.tidal,initialCells:4000,worldSize:8192,capacity:64000};
await c.send({type:'init',config:parent});vm.runInContext('world.state.time=47;world.state.marker="evolved parent"',c.context);const original=c.saved();
await c.send({type:'nursery',action:'enter'});
assert.equal(c.storage.get('nursery-parent'),original);assert.equal(c.snapshot().nursery.parentName,parent.name);assert.equal(c.snapshot().config.worldSize,128);assert.equal(c.snapshot().config.initialCells,64);assert.equal(c.snapshot().paused,false);
vm.runInContext('world.state.time=31;world.state.marker="nursery descendant"',c.context);const nursery=c.saved();
await c.send({type:'init',config:parent});assert.match(c.messages.at(-1).message,/Return to your planet/);assert.equal(c.saved(),nursery);
await c.send({type:'nursery',action:'leave'});assert.equal(c.saved(),original);assert.equal(c.snapshot().paused,true);assert.equal(c.snapshot().nursery,null);assert.equal(c.storage.get('nursery-latest'),nursery);
await c.send({type:'nursery',action:'enter'});assert.equal(c.saved(),nursery);await c.send({type:'nursery',action:'leave'});
const reopened=controller(c.storage);await reopened.send({type:'nursery',action:'recover'});assert.equal(reopened.saved(),original);assert.equal(reopened.snapshot().paused,true);
c.setFail();await c.send({type:'nursery',action:'enter'});assert.equal(c.saved(),original);assert.equal(c.snapshot().paused,true);assert.equal(c.snapshot().nursery,null);assert(c.messages.some(m=>m.type==='error'&&m.message.includes('Storage unavailable')));
const adviceContext=vm.createContext({STRIDE:12});vm.runInContext(source('src/evolution-guide.ts')+';this.advice=advise;',adviceContext);
const advise=adviceContext.advice;
const sample={config:parent,stats:{time:0,cells:64,cellBudgetReached:false},autonomous:{physics:{},divisions:0},ids:new Uint32Array([1]),data:new Float32Array(12),selected:null};
assert.equal(advise(null,'structure',0).action,'create');assert.equal(advise(sample,'structure',0).action,'nursery');
assert.equal(advise({...sample,stats:{...sample.stats,cellBudget:4096,cellBudgetReached:true}},'structure',0).action,'capacity');
assert.equal(advise({...sample,stats:{...sample.stats,cellBudget:64000,cellBudgetReached:true}},'structure',0).action,'inspect');
assert.equal(advise({...sample,stats:{...sample.stats,cells:0}},'structure',0).action,'founder');
assert.equal(advise({...sample,paused:true},'structure',0).action,'resume');
assert.equal(advise(sample,'structure',0,true).action,'resources');
console.log('Passed: parent preservation, nursery resume, recover-after-reload, blocked world replacement, storage failure and guide routing. Zero simulation steps.');
