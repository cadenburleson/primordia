/** Pointer-path and worker protocol checks only. No browser, WASM or simulation. */
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';
import {Vector3} from 'three';
const source=file=>ts.transpileModule(readFileSync(file,'utf8').replace(/^import .*;\n/gm,'').replace(/export /g,''),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
const math=vm.createContext({Vector3,noise:()=>0,setTimeout:()=>1,clearTimeout(){}});
vm.runInContext(source('src/sim/geometry.ts')+source('src/brush.ts')+';this.Path=BrushPath;this.Stroke=BrushStroke;',math);
const plane={worldSize:128,worldShape:'plane'},sphere={worldSize:128,worldShape:'sphere'};
const coarse=new math.Path(plane,1),fine=new math.Path(plane,1);
const expected=[...coarse.sample([0,0]),...coarse.sample([10,0])],actual=[];
for(let x=0;x<=10;x+=.25)actual.push(...fine.sample([x,0]));
assert.equal(JSON.stringify(actual),JSON.stringify(expected),'Dose must not depend on event rate');
const seam=new math.Path(sphere,.25);seam.sample([63,0]);const seamPoints=seam.sample([-63,0]);assert(seamPoints.length>=7&&seamPoints.length<=8);assert(seamPoints.every(([x,z])=>Math.abs(x)>62&&Math.abs(z)<1e-6),'Short seam crossing must not paint the other hemisphere');
const sent=[],notes=[],settings={tool:'flatten',radius:3,amount:.35,aspect:.5,angle:45};
const brush=new math.Stroke(c=>sent.push(c),()=>{},s=>notes.push(s));
assert(brush.begin({x:0,z:0,height:2},settings,plane));assert(!brush.begin({x:0,z:0,height:2},settings,plane));
brush.move({x:15,z:0,height:7});brush.end();assert.equal(sent.length,1,'Only one command may be in flight');
let handled=0;while(brush.busy){const c=sent[handled++];assert(c);assert(c.points.length<=12);assert.equal(c.settings.level,2);assert.equal(c.id,1);brush.accept({type:'brush-ack',id:c.id,done:c.end});assert(handled<30);}
assert.equal(sent.filter(c=>c.start).length,1);assert(sent.at(-1).end);assert(notes.length);
brush.begin({x:0,z:0,height:0},settings,plane);brush.accept({type:'brush-ack',id:2,done:true,error:'Invalid edit'});assert(!brush.busy);assert.equal(notes.at(-1),'Invalid edit');
const presetsContext=vm.createContext({});vm.runInContext(source('src/sim/types.ts')+';this.presets=PRESETS;',presetsContext);
const PRESETS=JSON.parse(JSON.stringify(presetsContext.presets));
class FakeWorld {
  constructor(json){this.state={config:JSON.parse(json),time:23,paint:0};this.saves=0;}
  start_physical(){} free(){} step(){throw new Error('Simulation forbidden in controller checks');}
  save(){this.saves++;return JSON.stringify(this.state);} load(json){this.state=JSON.parse(json);}
  intervene(json){const edit=JSON.parse(json);if(edit.points?.some(p=>p[0]===999))throw new Error('Outside world');this.state.paint+=edit.points?.length??1;return 'Edited';}
  metadata(){return JSON.stringify({config:this.state.config,stats:{time:this.state.time},autonomous:{physics:{}},selected:null});}
  display_world(){return {free(){},render_data:()=>new Float32Array(),ids:()=>new Uint32Array(),cell_data:()=>new Float32Array(),link_data:()=>new Float32Array(),joint_data:()=>new Float32Array()};}
  display_metadata(){return this.metadata();} field(){return new Float32Array();}
}
const messages=[],context=vm.createContext({PRESETS,World:FakeWorld,init:async()=>{},wasmUrl:'unused',performance:{now:()=>500},Float32Array,Uint32Array,self:{},postMessage:m=>messages.push(m),setTimeout(){},storeWorld:async()=>{},readWorld:async()=>null});
for(const file of ['src/sim/nursery-session.ts','src/sim/scheduler.ts','src/sim/assessment.ts','src/sim/worker.ts'])vm.runInContext(source(file),context);
const send=async cmd=>{context.self.onmessage({data:cmd});await vm.runInContext('commands',context);};
const state=()=>JSON.parse(vm.runInContext('JSON.stringify(world.state)',context));
await send({type:'init',config:PRESETS.tidal});const before=state();
const chunk=(id,start,end,points,extra={})=>({type:'brush-stroke',id,start,end,points,settings:{...settings,...extra}});
await send(chunk(1,true,false,[[0,0]]));await send(chunk(1,false,false,[[1,0],[2,0]]));
assert.equal(vm.runInContext('world.saves',context),1,'Serialize one undo checkpoint per stroke');
await send({type:'control',paused:false});assert.match(messages.at(-1).message,/Finish the brush/);assert(vm.runInContext('paused',context));
await send(chunk(1,false,true,[[3,0]]));assert.equal(state().paint,4);assert.equal(messages.at(-1).done,true);
await send({type:'editor',action:'undo'});assert.deepEqual(state(),before,'Undo must restore the entire drag');
await send(chunk(2,true,false,[[0,0]]));await send(chunk(2,false,true,[[999,0]]));assert.match(messages.findLast(m=>m.type==='brush-ack').error,/Outside/);assert.equal(state().paint,1,'Failed chunk must not erase successful chunks');
await send({type:'editor',action:'undo'});assert.deepEqual(state(),before,'Failed chunk retains the original undo');
await send(chunk(3,true,false,[[0,0]]));await send(chunk(3,false,true,[],{angle:90}));assert.match(messages.findLast(m=>m.type==='brush-ack').error,/settings changed/);
await send({type:'editor',action:'undo'});assert.deepEqual(state(),before);
console.log('Passed: event-rate-independent strokes, globe seam, backpressure, fixed flatten level, one checkpoint, whole-stroke undo, interrupted/invalid batches. Zero simulation steps.');
