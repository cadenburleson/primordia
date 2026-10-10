import init, { World } from '../wasm/primordia_engine';
import wasmUrl from '../wasm/primordia_engine_bg.wasm?url';
import { PRESETS, type Command, type Snapshot } from './types';
import {storeWorld,readWorld} from '../storage';
import {SimulationClock, requestedSpeed, BATCH_MS} from './scheduler';
const clock = new SimulationClock(); clock.reset(performance.now());

let world: World | null = null;
let paused = false, speed = 1, selected = 0, lastPublish = 0;
let computeMs = 0;
let ecosystemBackup:string|null=null;
let previousWorld:string|null=null;
const ready = init({ module_or_path: wasmUrl });
function publish() {
  if (!world) return;
  const meta = JSON.parse(world.metadata(selected)) as Snapshot;
  const display = world.display_world();
  try {
    meta.type = 'snapshot'; meta.data = display.render_data(); meta.ids = display.ids(); meta.field = world.field();
    meta.cellData=display.cell_data();meta.linkData=display.link_data();meta.jointData=display.joint_data();
  } finally { display.free(); }
  meta.stats.computeMs = computeMs;
  meta.stats.actualSpeed = paused ? 0 : clock.measure(performance.now());
  meta.stats.requestedSpeed = speed;
  meta.canUndoWorld = previousWorld !== null;
  postMessage(meta, { transfer: [meta.data.buffer as ArrayBuffer, meta.ids.buffer as ArrayBuffer, meta.field.buffer as ArrayBuffer,meta.cellData.buffer as ArrayBuffer,meta.linkData.buffer as ArrayBuffer,meta.jointData.buffer as ArrayBuffer] });
}
let commands = Promise.resolve();
self.onmessage = (event: MessageEvent<Command>) => {
  commands = commands.then(async () => {
    await ready;
    const cmd = event.data;
    try {
      if (cmd.type === 'init') {
        const next = new World(JSON.stringify(cmd.config)); next.start_physical();
        if (world) { previousWorld=world.save(); try { await storeWorld(previousWorld,'before-new-world'); } catch {} }
        else { try { previousWorld=await readWorld('before-new-world'); } catch {} }
        world?.free(); world = next; selected = 0; clock.reset(performance.now()); paused = false;
        ecosystemBackup=null;
      } else if (cmd.type === 'control') {
        if (cmd.paused !== undefined) paused = cmd.paused;
        if (cmd.speed !== undefined) speed = requestedSpeed(cmd.speed);
        clock.reset(performance.now());
      } else if (world) {
        if (cmd.type === 'undo-world') {
          if (!previousWorld) throw new Error('No previous world is held');
          world.load(previousWorld); previousWorld=null; paused=true; selected=0; clock.reset(performance.now());
          postMessage({type:'loaded'});
        }
        if (cmd.type === 'cell-budget') world.set_cell_budget(cmd.capacity);
        if (cmd.type === 'physical-study') {
          const next = new World(JSON.stringify(cmd.config)); next.start_physical();
          const backup = world.save();
          try { await storeWorld(backup,'ecosystem-before-lab'); } catch {}
          ecosystemBackup=backup; world.free(); world=next; selected=0; clock.reset(performance.now());
        }
        if (cmd.type === 'return-physical') {
          let backup=ecosystemBackup;
          if(!backup) { try { backup=await readWorld('ecosystem-before-lab'); } catch {} }
          if (!backup) throw new Error('No held habitat is available');
          world.load(backup); ecosystemBackup=null; selected=0; clock.reset(performance.now());
        }
        if(cmd.type==='autonomous'){
          ecosystemBackup=world.save();
          try{await storeWorld(ecosystemBackup,'ecosystem-before-lab');}catch{}
          world.start_physical();selected=0;clock.reset(performance.now());
        }
        if(cmd.type==='competition'){
          if(cmd.action==='start'){
            ecosystemBackup=world.save();
            try{await storeWorld(ecosystemBackup,'ecosystem-before-lab');}catch{}
            world.start_population(cmd.count);selected=0;
          }else if(cmd.action==='introduce') world.seed_competitors(cmd.count);
          else world.set_directional_sensing(cmd.action==='sense-on');
          clock.reset(performance.now());
        }
        if(cmd.type==='lab'){
          if(cmd.action==='start'||cmd.action==='study'||cmd.action==='joints') {
            ecosystemBackup=world.save();
            try{await storeWorld(ecosystemBackup,'ecosystem-before-lab');}catch{}
            if(cmd.action==='study')world.start_tissue_study();else if(cmd.action==='joints')world.start_joint_lab();else world.start_lab();selected=1;
          }else if(cmd.action==='motors-on'||cmd.action==='motors-off')world.set_motors_enabled(cmd.action==='motors-on');
          else if(cmd.action==='replay'||cmd.action==='evolve')world.replay_best(cmd.action==='replay');
          else{
            let backup=ecosystemBackup;
            if(!backup){try{backup=await readWorld('ecosystem-before-lab');}catch{}}
            if(backup)world.load(backup);
            else{const next=new World(JSON.stringify(PRESETS.tidal));world.free();world=next;}
            ecosystemBackup=null;selected=0;
          }
          clock.reset(performance.now());
        }
        if (cmd.type === 'config') world.configure(JSON.stringify(cmd.patch));
        if (cmd.type === 'seed') world.seed(cmd.count);
        if (cmd.type === 'pulse') world.pulse();
        if (cmd.type === 'select') selected = cmd.id ?? 0;
        if (cmd.type === 'save') postMessage({ type: 'save', json: world.save() });
        if (cmd.type === 'load') {
          const previous=world.save();world.load(JSON.stringify(cmd.data));
          const loadedMeta=JSON.parse(world.metadata(0)),previousState=JSON.parse(previous);
          if((loadedMeta.lab||loadedMeta.study||loadedMeta.competition||loadedMeta.autonomous)&&!previousState.lab&&!previousState.study&&!previousState.competition){
            let held:string|null=null;
            if(cmd.restore){try{held=await readWorld('ecosystem-before-lab');}catch{}}
            ecosystemBackup=held??previous;
            if(!held){try{await storeWorld(previous,'ecosystem-before-lab');}catch{}}
          }
          selected=0;clock.reset(performance.now());postMessage({type:'loaded'});
        }
      }
      publish();
    } catch (error) { postMessage({ type: 'error', message: String(error) }); }
  }).catch(error => postMessage({ type: 'error', message: `Could not start Rust engine: ${error}` }));
};
function pump() {
  const now = performance.now();
  if (world && !paused) {
    clock.accrue(now, speed);
    const start = performance.now(); let steps = 0;
    // Yield regularly to handle pause/save/config, even in Max mode. One slow
    // Rust tick cannot be interrupted, so measured speed is always reported.
    while (steps < 256 && performance.now() - start < BATCH_MS && clock.take(speed)) {
      world.step(1); steps++; clock.record();
    }
    if (steps) computeMs = (performance.now() - start) / steps;
  }
  if (world && performance.now() - lastPublish > (speed === 0 ? 500 : 250)) {
    lastPublish = performance.now(); publish();
  }
  setTimeout(pump, paused ? 25 : speed === 0 ? 0 : 8);
}
setTimeout(pump, 25);
