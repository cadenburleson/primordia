import init, { World } from '../wasm/primordia_engine';
import wasmUrl from '../wasm/primordia_engine_bg.wasm?url';
import { PRESETS, type Command, type Snapshot } from './types';
import {storeWorld,readWorld} from '../storage';

let world: World | null = null;
let paused = false, speed = 1, selected = 0, last = performance.now(), accumulator = 0, lastPublish = 0;
let computeMs = 0;
let visible = true, snapshotDirty = false;
// Yield between bounded CPU slices instead of chasing acceleration continuously.
// A synchronous WASM step cannot be interrupted; the deadline is checked between steps.
const COMPUTE_SLICE_MS = 6, IDLE_MS = 35, MAX_BATCH_STEPS = 32;
let ecosystemBackup:string|null=null;
const ready = init({ module_or_path: wasmUrl });
function publish() {
  if (!world) return;
  const meta = JSON.parse(world.metadata(selected)) as Snapshot;
  meta.type = 'snapshot'; meta.data = world.render_data(); meta.ids = world.ids(); meta.field = world.field();
  meta.cellData=world.cell_data();meta.linkData=world.link_data();meta.jointData=world.joint_data();
  meta.stats.computeMs = computeMs;
  postMessage(meta, { transfer: [meta.data.buffer as ArrayBuffer, meta.ids.buffer as ArrayBuffer, meta.field.buffer as ArrayBuffer,meta.cellData.buffer as ArrayBuffer,meta.linkData.buffer as ArrayBuffer,meta.jointData.buffer as ArrayBuffer] });
  snapshotDirty = false;
  lastPublish = performance.now();
}
let commands = Promise.resolve();
self.onmessage = (event: MessageEvent<Command>) => {
  commands = commands.then(async () => {
    await ready;
    const cmd = event.data;
    try {
      if (cmd.type === 'visibility') {
        visible = cmd.visible; accumulator = 0; last = performance.now();
        return;
      }
      if (cmd.type === 'init') {
        const next = new World(JSON.stringify(cmd.config)); next.start_autonomous();
        world?.free(); world = next; selected = 0; accumulator = 0; paused = false; last = performance.now();
        ecosystemBackup=null;
      } else if (cmd.type === 'control') {
        if (cmd.paused !== undefined) paused = cmd.paused;
        if (cmd.speed !== undefined) speed = Math.max(1, Math.min(100, cmd.speed));
        accumulator = 0; last = performance.now();
      } else if (world) {
        if(cmd.type==='autonomous'){
          ecosystemBackup=world.save();
          try{await storeWorld(ecosystemBackup,'ecosystem-before-lab');}catch{}
          world.start_autonomous();selected=0;accumulator=0;last=performance.now();
        }
        if(cmd.type==='competition'){
          if(cmd.action==='start'){
            ecosystemBackup=world.save();
            try{await storeWorld(ecosystemBackup,'ecosystem-before-lab');}catch{}
            world.start_population(cmd.count);selected=0;
          }else if(cmd.action==='introduce') world.seed_competitors(cmd.count);
          else world.set_directional_sensing(cmd.action==='sense-on');
          accumulator=0;last=performance.now();
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
          accumulator=0;last=performance.now();
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
          selected=0;accumulator=0;postMessage({type:'loaded'});
        }
      }
      publish();
    } catch (error) { postMessage({ type: 'error', message: String(error) }); }
  }).catch(error => postMessage({ type: 'error', message: `Could not start Rust engine: ${error}` }));
};
function runSlice() {
  const now = performance.now();
  const elapsed = Math.min((now - last) / 1000, .25); last = now;
  try {
    if (world && !paused && visible) {
      // Discard wall-time debt under load; never skip or enlarge biological steps.
      accumulator = Math.min(MAX_BATCH_STEPS, accumulator + elapsed * speed * 4);
      const due = Math.floor(accumulator), start = performance.now();
      let steps = 0;
      while (steps < due && performance.now() - start < COMPUTE_SLICE_MS) {
        world.step(1); steps++;
      }
      accumulator -= steps;
      if (steps > 0) {
        computeMs = (performance.now() - start) / steps; snapshotDirty = true;
      }
    }
    // Paused worlds keep their last snapshot. Hidden worlds do no periodic work.
    if (world && visible && snapshotDirty && performance.now() - lastPublish >= 100) publish();
  } catch (error) {
    paused = true;
    postMessage({type:'error',message:`Simulation stopped: ${error}`});
  } finally {
    // Schedule after work completes, guaranteeing idle time even for a slow step/publish.
    setTimeout(runSlice, visible && !paused ? IDLE_MS : 250);
  }
}
setTimeout(runSlice, IDLE_MS);
