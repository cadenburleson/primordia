import init, { World } from '../wasm/primordia_engine';
import wasmUrl from '../wasm/primordia_engine_bg.wasm?url';
import { PRESETS, type Command, type Snapshot } from './types';
import {storeWorld,readWorld} from '../storage';
import {NurserySession,nurseryConfig} from './nursery-session';
import {AssessmentRecorder} from './assessment';
import {SimulationClock, requestedSpeed, BATCH_MS, MAX_BATCH_MS} from './scheduler';
const clock = new SimulationClock(); clock.reset(performance.now());

let world: World | null = null;
let paused = false, speed = 1, selected = 0, lastPublish = 0;
let computeMs = 0;
let visible = true, snapshotDirty = false;
let ecosystemBackup:string|null=null;
let previousWorld:string|null=null;
let editUndo:string|null=null,editUndoSelection=0;
let brushStroke:{id:number;settings:string}|null=null;
let checkpoint:string|null=null,checkpointSelection=0,assessmentDeadline:number|null=null;
const assessment=new AssessmentRecorder();
const nursery=new NurserySession();
let ready:ReturnType<typeof init>|null=null;
let displayPriority:'live'|'evolution'='live';
let pumpScheduled=false;
function publish() {
  if (!world) return;
  const preparationStart=performance.now();
  const display = world.display_world();
  let meta:Snapshot;
  try {
    meta=JSON.parse(world.display_metadata(selected,display)) as Snapshot;
    meta.type = 'snapshot'; meta.data = display.render_data(); meta.ids = display.ids(); meta.field = world.field();
    meta.cellData=display.cell_data();meta.linkData=display.link_data();meta.jointData=display.joint_data();
  } finally { display.free(); }
  meta.stats.computeMs = computeMs;
  meta.stats.snapshotMs=performance.now()-preparationStart;
  meta.stats.actualSpeed = paused ? 0 : clock.measure(performance.now());
  meta.stats.requestedSpeed = speed;
  meta.canUndoWorld = previousWorld !== null;
  meta.nursery=nursery.active?{parentName:nursery.parentName}:null;
  postMessage({type:'editor-state',canUndo:editUndo!==null});
  meta.selection=selected;meta.paused=paused;assessment.observe(meta);meta.assessment=assessment.view(checkpoint!==null,meta.stats.time);
  postMessage(meta, { transfer: [meta.data.buffer as ArrayBuffer, meta.ids.buffer as ArrayBuffer, meta.field.buffer as ArrayBuffer,meta.cellData.buffer as ArrayBuffer,meta.linkData.buffer as ArrayBuffer,meta.jointData.buffer as ArrayBuffer] });
  snapshotDirty=false;lastPublish=performance.now();
}
let commands = Promise.resolve();
self.onmessage = (event: MessageEvent<Command>) => {
  commands = commands.then(async () => {
    const cmd = event.data;
    try {
      if(cmd.type==='visibility'){
        visible=cmd.visible;clock.reset(performance.now());return;
      }
      if(cmd.type==='display-priority'){displayPriority=cmd.mode;return;}
      await (ready??=init({module_or_path:wasmUrl}));
      if(brushStroke&&cmd.type!=='brush-stroke'&&!['visibility','display-priority'].includes(cmd.type))throw new Error('Finish the brush stroke before another action.');
      if(cmd.type==='brush-stroke'){
        try {
          if(!world)throw new Error('Create a world first.');
          if(!Number.isSafeInteger(cmd.id)||cmd.points.length>12||(!cmd.points.length&&!cmd.end))throw new Error('Invalid brush batch.');
          const settings=JSON.stringify(cmd.settings);
          let before:string|null=null;
          if(cmd.start){
            if(brushStroke)throw new Error('A brush stroke is already active.');
            const meta=JSON.parse(world.metadata(selected)) as Snapshot;
            if(assessment.running||assessment.control||!meta.autonomous?.physics||(meta.config.physicalAssay??'full')!=='full'||meta.autonomous.observationControl)throw new Error('Return to the main cellular world before painting.');
            before=world.save();
          }else if(!brushStroke||brushStroke.id!==cmd.id||brushStroke.settings!==settings)throw new Error('Brush settings changed during the stroke.');
          if(cmd.points.length){
            const [x,z]=cmd.points[0];
            world.intervene(JSON.stringify({kind:'brush',...cmd.settings,x,z,points:cmd.points}));
            if(before!==null){editUndo=before;editUndoSelection=selected;}
          }
          paused=true;checkpoint=null;assessmentDeadline=null;assessment.reset();clock.reset(performance.now());
          brushStroke=cmd.end?null:{id:cmd.id,settings};
          // Render at most four previews per second; final state is always sent.
          if(cmd.end||performance.now()-lastPublish>250)publish();
          postMessage({type:'brush-ack',id:cmd.id,done:cmd.end});
        } catch(error){
          brushStroke=null;paused=true;clock.reset(performance.now());
          postMessage({type:'brush-ack',id:cmd.id,done:true,error:String(error instanceof Error?error.message:error)});
          if(world)publish();
        }
        return;
      }
      if(cmd.type==='nursery'){
        if(assessment.control||assessment.running)throw new Error('Return to the comparison checkpoint before changing worlds.');
        if(cmd.action==='enter'){
          if(!world||nursery.active)throw new Error('Open the nursery from your main planet.');
          const meta=JSON.parse(world.metadata(selected)) as Snapshot;
          if(!meta.autonomous?.physics||meta.lab||meta.study||meta.competition||(meta.config.physicalAssay??'full')!=='full')throw new Error('Return to your main cellular planet first.');
          const backup=world.save();paused=true;
          // Preserve the parent before replacing the active world. If storage
          // fails, the current planet remains available, paused and intact.
          await storeWorld(backup,'nursery-parent');
          const next=new World(JSON.stringify(nurseryConfig(meta.config)));
          try{
            if(nursery.savedHabitat)next.load(nursery.savedHabitat);
            else{
              next.start_physical();
              // A new nursery inherits the planet's star, oceans and pace of change.
              const dials=meta.autonomous.steering;
              if(dials)for(const action of [{kind:'light',scale:dials.light},{kind:'vents',scale:dials.vents},{kind:'mixing',scale:dials.mixing},{kind:'mutation',scale:dials.mutation}])next.steer(JSON.stringify(action));
            }
          }catch(error){next.free();throw error;}
          nursery.begin(backup,meta.config.name);world.free();world=next;speed=0;displayPriority='evolution';paused=false;
        }else{
          paused=true;
          const backup=cmd.action==='leave'?nursery.parent:(nursery.parent??await readWorld('nursery-parent'));
          if(!backup)throw new Error('No held planet is available.');
          const nurseryState=nursery.active&&world?world.save():null;
          if(nurseryState)await storeWorld(nurseryState,'nursery-latest');
          const next=new World(JSON.stringify({...PRESETS.tidal,origin:'chemistry',initialCells:1}));
          try{next.load(backup);}catch(error){next.free();throw error;}
          // Recovering a checkpoint outside a nursery should also be undoable.
          if(cmd.action==='recover'&&!nursery.active&&world){previousWorld=world.save();try{await storeWorld(previousWorld,'before-new-world');}catch{}}
          world?.free();world=next;
          if(cmd.action==='leave'&&nurseryState)nursery.finish(nurseryState);else nursery.reset();
          paused=true;
        }
        selected=0;checkpoint=null;assessmentDeadline=null;assessment.reset();editUndo=null;clock.reset(performance.now());
        postMessage({type:'nursery-changed',active:nursery.active});publish();schedulePump();return;
      }
      if(nursery.active&&['init','load','undo-world','physical-study','return-physical','competition','lab','autonomous'].includes(cmd.type))throw new Error('Return to your planet before opening another world or lab. Saved organisms stay in your collection.');
      if(cmd.type==='load'&&!world){
        const next=new World(JSON.stringify({...PRESETS.tidal,origin:'chemistry',initialCells:1}));
        try{next.load(JSON.stringify(cmd.data));}catch(error){next.free();throw error;}
        world=next;selected=0;paused=true;clock.reset(performance.now());postMessage({type:'loaded'});publish();schedulePump();return;
      }
      if(assessment.running&&['config','seed','pulse','cell-budget','specimen','editor','steer'].includes(cmd.type))throw new Error('Resource and population changes are disabled during a matched comparison.');
      if(cmd.type==='steer'){
        if(!world)throw new Error('Create a world first.');
        if(assessment.running||assessment.control)throw new Error('Return to the checkpoint before steering evolution.');
        // Disasters can be undone from World → Undo; dials and breeding goals leave time running.
        const destructive=cmd.action.kind==='meteor'||cmd.action.kind==='dying';
        const before=destructive?world.save():null;
        const message=world.steer(JSON.stringify(cmd.action));
        if(before!==null){editUndo=before;editUndoSelection=selected;}
        checkpoint=null;assessmentDeadline=null;assessment.reset();
        postMessage({type:'steered',kind:cmd.action.kind,message});
        publish();schedulePump();return;
      }
      if(cmd.type==='editor'){
        if(!world)throw new Error('Create a world first.');
        const meta=JSON.parse(world.metadata(selected)) as Snapshot;
        if(assessment.control||!meta.autonomous?.physics||meta.config.physicalAssay!=='full'||meta.autonomous.observationControl)throw new Error('Return to the main cellular world before editing.');
        if(cmd.action==='copy'||cmd.action==='founder'){
          const json=cmd.action==='copy'?world.capture_specimen(cmd.id,cmd.name):world.founder_specimen();
          const summary=JSON.parse(world.inspect_specimen(json));paused=true;
          postMessage({type:'editor-specimen',json,summary});
        }else if(cmd.action==='undo'){
          if(!editUndo)throw new Error('No editor checkpoint is held.');
          world.load(editUndo);selected=editUndoSelection;editUndo=null;paused=true;
          checkpoint=null;assessmentDeadline=null;assessment.reset();
          postMessage({type:'edited',message:'World restored to just before the last edit, including its time and organisms.'});
        }else{
          const before=world.save(),priorSelection=selected;
          const message=world.intervene(JSON.stringify(cmd.edit));
          editUndo=before;editUndoSelection=priorSelection;paused=true;
          checkpoint=null;assessmentDeadline=null;assessment.reset();
          postMessage({type:'edited',message});
        }
        clock.reset(performance.now());publish();return;
      }
      if(cmd.type==='specimen'){
        if(!world)throw new Error('Create a world first.');
        if(cmd.action==='inspect'){
          const summary=JSON.parse(world.inspect_specimen(cmd.json));postMessage({type:'specimen',action:'inspect',json:cmd.json,summary});return;
        }
        if(assessment.control)throw new Error('Return to the main-world checkpoint before capturing or introducing organisms.');
        if(cmd.action==='capture'){
          const meta=JSON.parse(world.metadata(cmd.id)) as Snapshot;
          if(!meta.selected?.observation||meta.autonomous?.observationControl)throw new Error('Select a living organism in the main cellular world.');
          const json=world.capture_specimen(cmd.id,cmd.name),summary=JSON.parse(world.inspect_specimen(json));paused=true;selected=cmd.id;
          postMessage({type:'specimen',action:'capture',json,summary});
        }else{
          const before=world.save(),priorSelection=selected;
          selected=world.place_specimen(cmd.json,cmd.x,cmd.z);paused=true;editUndo=before;editUndoSelection=priorSelection;
          checkpoint=null;assessmentDeadline=null;assessment.reset();
          postMessage({type:'specimen-placed',id:selected});
        }
        clock.reset(performance.now());publish();return;
      }
      if(cmd.type==='save'&&assessment.control)throw new Error('Return to the checkpoint before saving a main world. Export the Evidence report to keep trial results.');
      if(['init','load','undo-world','physical-study','return-physical','competition','lab'].includes(cmd.type)){checkpoint=null;assessmentDeadline=null;assessment.reset();editUndo=null;}
      if(cmd.type==='assessment'){
        if(!world)throw new Error('Create a world first.');
        const meta=JSON.parse(world.metadata(selected)) as Snapshot;
        if(cmd.action==='capture'){
          editUndo=null;
          if(assessment.running||meta.autonomous?.observationControl)throw new Error('Return to the original main-world checkpoint before capturing.');
          if(!meta.autonomous?.physics||meta.config.physicalAssay!=='full'||!meta.selected?.observation)throw new Error('Select a living body in the main physical world.');
          checkpoint=world.save();checkpointSelection=selected;assessment.reset();assessmentDeadline=null;paused=true;
        }else{
          if(!checkpoint)throw new Error('Capture a checkpoint first.');
          const duration=cmd.duration??20;
          if(cmd.action==='run'&&(![5,20,50].includes(duration)||!['baseline','motors','transport','signals'].includes(cmd.control??'')))throw new Error('Choose a valid comparison and duration.');
          world.load(checkpoint);selected=checkpointSelection;assessmentDeadline=null;paused=true;assessment.running=false;
          if(cmd.action==='restore'){assessment.control=null;assessment.start=null;assessment.last=null;}
          if(cmd.action==='run'){world.observation_control(cmd.control!);const start=JSON.parse(world.metadata(selected)) as Snapshot;assessment.begin(start,cmd.control!,duration);assessmentDeadline=world.simulation_time()+duration;paused=false;}
        }
        clock.reset(performance.now());publish();return;
      }
      if (cmd.type === 'init') {
        const next = new World(JSON.stringify(cmd.config)); next.start_physical();
        // Starting conditions chosen at creation, such as the star's brightness.
        try{for(const action of cmd.steering??[])next.steer(JSON.stringify(action));}catch(error){next.free();throw error;}
        paused=true;
        if (world) { previousWorld=world.save(); try { await storeWorld(previousWorld,'before-new-world'); } catch {} }
        else { try { previousWorld=await readWorld('before-new-world'); } catch {} }
        world?.free(); world = next; selected = 0; clock.reset(performance.now()); paused = false;
        ecosystemBackup=null;nursery.reset();postMessage({type:'world-created'});
      } else if (cmd.type === 'control') {
        if (cmd.paused !== undefined) paused = cmd.paused;
        if (cmd.speed !== undefined) speed = requestedSpeed(cmd.speed);
        clock.reset(performance.now());
      } else if (world) {
        if (cmd.type === 'undo-world') {
          if (!previousWorld) throw new Error('No previous world is held');
          world.load(previousWorld); nursery.reset();previousWorld=null; paused=true; selected=0; clock.reset(performance.now());
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
        if (cmd.type === 'select'&&!assessment.running) selected = cmd.id ?? 0;
        if (cmd.type === 'save') postMessage({ type: 'save', json: world.save() });
        if (cmd.type === 'load') {
          const previous=world.save();world.load(JSON.stringify(cmd.data));nursery.reset();
          const loadedMeta=JSON.parse(world.metadata(0)),previousState=JSON.parse(previous);
          if((loadedMeta.lab||loadedMeta.study||loadedMeta.competition||loadedMeta.autonomous)&&!previousState.lab&&!previousState.study&&!previousState.competition){
            let held:string|null=null;
            if(cmd.restore){try{held=await readWorld('ecosystem-before-lab');}catch{}}
            ecosystemBackup=held??previous;
            if(!held){try{await storeWorld(previous,'ecosystem-before-lab');}catch{}}
          }
          selected=0;paused=true;clock.reset(performance.now());postMessage({type:'loaded'});
        }
      }
      publish();schedulePump();
    } catch (error) { postMessage({ type: 'error', message: String(error) });if(cmd.type==='nursery'&&world)publish(); }
  }).catch(error => postMessage({ type: 'error', message: `Could not start Rust engine: ${error}` }));
};
function pump() {
  const now = performance.now();
  if (world && !paused && visible) {
    clock.accrue(now, speed);
    const start = performance.now(), budget = speed === 0 ? MAX_BATCH_MS : BATCH_MS; let steps = 0;
    // Yield between batches to handle pause/save/config, even in Max mode. One
    // slow Rust tick cannot be interrupted, so measured speed is always reported.
    while (steps < 256 && performance.now() - start < budget && clock.take(speed)) {
      world.step(1); steps++; clock.record();
      if(assessmentDeadline!==null&&world.simulation_time()>=assessmentDeadline){
        const meta=JSON.parse(world.metadata(selected)) as Snapshot;assessment.finish(meta);assessmentDeadline=null;paused=true;break;
      }
    }
    if (steps) {computeMs = (performance.now() - start) / steps;snapshotDirty=true;}
  }
  if (world && visible && snapshotDirty && performance.now() - lastPublish > (displayPriority==='evolution' ? 2000 : speed === 0 ? 500 : 250)) {
    publish();
  }
  // Max, or a paced speed that is still behind, continues immediately: the
  // worker thread is dedicated to evolution. Otherwise wait for the next frame.
  const busy = speed === 0 || clock.pending();
  schedulePump(paused || !visible ? 250 : busy ? 0 : 16);
}
// A message to ourselves queues behind pending commands without the 4 ms
// minimum that nested timers impose.
const yieldChannel = typeof MessageChannel === 'undefined' ? null : new MessageChannel();
if (yieldChannel) yieldChannel.port1.onmessage = () => { pumpScheduled = false; pump(); };
function schedulePump(delay=16){
  if(!world||pumpScheduled)return;
  pumpScheduled=true;
  if(delay===0&&yieldChannel)yieldChannel.port2.postMessage(0);
  else setTimeout(()=>{pumpScheduled=false;pump();},delay);
}
