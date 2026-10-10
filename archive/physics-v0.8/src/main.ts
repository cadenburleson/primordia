import { createIcons, Orbit, Dna, Sun, Droplets, Mountain, Thermometer, Zap, Play, Pause, RotateCcw, Plus, Focus, Layers, ArrowUpRight, Download, Upload, X, FlaskConical, Leaf, Settings2, CircleHelp, Volume2, VolumeX, ChevronRight, Activity, Bone, Microscope, Maximize2 } from 'lucide';
import { WorldView } from './render/world';
import { SpecimenView } from './render/specimen';
import { PRESETS, GI, GENES, TISSUES, type WorldConfig, type WorldKind, STRIDE, type Snapshot, type Command } from './sim/types';
import './style.css';
import {storeWorld,readWorld} from './storage';

const icons={Orbit,Dna,Sun,Droplets,Mountain,Thermometer,Zap,Play,Pause,RotateCcw,Plus,Focus,Layers,ArrowUpRight,Download,Upload,X,FlaskConical,Leaf,Settings2,CircleHelp,Volume2,VolumeX,ChevronRight,Activity,Bone,Microscope,Maximize2};
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const icon=(name:string)=>`<i data-lucide="${name}"></i>`;
const esc=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
const fmt=(n:number)=>Math.floor(n).toLocaleString();
let config:WorldConfig={...PRESETS.tidal};
let snapshot:Snapshot|null=null,paused=false,speed=1,selectedId:number|null=null,tab='life',autoselect=true;
let view:WorldView, specimen:SpecimenView;
let lastUI=0,lastSpecimen=0,pendingLoad=false;
const worker=new Worker(new URL('./sim/worker.ts',import.meta.url),{type:'module'});
const send=(command:Command)=>worker.postMessage(command);

document.querySelector('#app')!.innerHTML=`
  <header class="topbar">
    <a class="brand" href="/" aria-label="Primordia home"><span class="brand-mark">${icon('orbit')}</span>primordia<span class="brand-beta">LAB / 08</span></a>
    <div class="header-center"><span class="live-dot"></span> AN ARTIFICIAL LIFE EXPERIMENT</div>
    <div class="header-actions"><button id="help" class="icon-button" title="How this world works" aria-label="How this world works">${icon('circle-help')}</button><button id="restore" class="text-button" title="Restore the last world saved in this browser" aria-label="Restore last save" disabled>${icon('rotate-ccw')}<span>Restore</span></button><button id="load" class="text-button" aria-label="Open a world file">${icon('upload')}<span>Open</span></button><button id="save" class="save-button" aria-label="Save world">${icon('download')}<span>Save world</span></button></div>
  </header>
  <div class="workspace">
    <aside class="world-panel">
      <div class="panel-intro"><span class="eyebrow">THE POSSIBILITY OF LIFE</span><h1>Every beginning<br>is different.</h1><p>Shape the conditions.<br>Let life find its own way.</p></div>
      <section class="panel-section"><div class="section-label"><span>01</span> YOUR WORLD</div>
        <div class="world-options">
          <button class="world-option active" data-world="tidal"><span class="planet tidal"></span><span><strong>Tidal garden</strong><small>Water-rich · abundant light</small></span>${icon('chevron-right')}</button>
          <button class="world-option" data-world="volcanic"><span class="planet volcanic"></span><span><strong>Ember sea</strong><small>Mineral-rich · dim light</small></span>${icon('chevron-right')}</button>
          <button class="world-option" data-world="alien"><span class="planet alien"></span><span><strong>Violet frontier</strong><small>Less solvent · moderate light</small></span>${icon('chevron-right')}</button>
        </div>
      </section>
      <section class="panel-section run-setup" id="run-setup"><div class="section-label"><span>02</span> EXPERIMENT SETUP</div>
        <div class="capacity-row"><label for="initial-cells">Starting organisms <small>Each begins as one cell</small></label><input id="initial-cells" type="number" min="1" max="8192" step="1" value="256"></div>
        <div class="capacity-row"><label for="cell-budget">Total living-cell ceiling</label><input id="cell-budget" type="number" min="100" max="64000" step="1" value="8192"></div>
        <button id="apply-budget" class="observe-button">Apply ceiling to this run</button>
        <div class="capacity-row"><label for="setup-seed">World seed</label><input id="setup-seed" type="number" min="0" max="4294967295" value="8042"></div>
        <button id="create-physical" class="primary-button">Create physical world</button><button id="undo-world" class="observe-button" disabled>Restore previous world</button>
        <p class="lab-disclosure">Starting count and seed apply to new runs. The ceiling can change during a run without changing genes or habitat resources. More cells share the same finite habitat.</p>
        <div class="section-label">SHARED ENGINE EXPERIMENTS</div>
        <button id="shared-passive" class="observe-button">Open passive mechanics assay</button>
        <button id="shared-scarcity" class="observe-button">Open scarce-resource assay</button>
        <button id="return-physical" class="observe-button" hidden>Return to held physical world</button>
        <p class="lab-disclosure">Your current physical world is saved and held unchanged; Return restores it. These use the main physical engine, selected population and ceiling. Passive mechanics disables actuation; scarcity reduces initial fuel. Neither awards fitness scores or supplies adult bodies.</p>
      </section>
      <section class="panel-section mechanics-entry"><div class="section-label"><span>02</span> EMBODIED EVOLUTION</div><h3>From cells to motion.</h3><p id="mechanics-description">Attached cells can digest smaller lifeforms, grow support segments and paired muscles, and inherit movement controllers. Explore their development in the lab.</p><button id="start-lab" class="observe-button">${icon('activity')} Legacy locomotion comparison ${icon('arrow-up-right')}</button><button id="start-joints" class="observe-button">${icon('bone')} Legacy joint comparison ${icon('arrow-up-right')}</button><button id="start-autonomous" class="observe-button">Open autonomous evolution</button><div id="competition-entry"><label for="competing-founders">Competing founders</label><select id="competing-founders"><option value="24">24 cells</option><option value="48" selected>48 cells</option><option value="96">96 cells</option></select><button id="start-population" class="observe-button">${icon('dna')} Legacy competing ecosystem ${icon('arrow-up-right')}</button></div><button id="start-tissue" class="observe-button">${icon('microscope')} Legacy tissue comparison ${icon('arrow-up-right')}</button><button id="leave-lab" class="observe-button" hidden>${icon('arrow-up-right')} Return to ecosystem</button><p class="lab-disclosure">Studies supply a precursor with stronger tissue potentials. Tissue ecology uses survival and reproduction; the locomotion lab selects measured movement. Your ecosystem is held while you explore.</p></section>
      <section class="panel-section conditions"><div class="section-label"><span>03</span> INITIAL CONDITIONS</div><div id="sliders"></div>
        <div class="capacity-row"><label for="capacity">Population budget</label><select id="capacity"><option value="1000">1,000</option><option value="4000" selected>4,000</option><option value="12000">12,000</option></select></div>
      </section>
      <section class="panel-section origin-section"><div class="section-label"><span>04</span> THE FIRST SPARK</div>
        <label class="check-row"><input id="chemistry" type="checkbox"><span>Begin with chemistry alone<small>A speculative origin model</small></span></label>
        <button id="new-world" class="primary-button">${icon('plus')} Create a new world ${icon('arrow-up-right')}</button>
        <div class="seed-row"><label for="seed">World seed</label><input id="seed" type="number" value="8042" min="0" max="4294967295" aria-label="World seed"><button id="random-seed" class="icon-button" aria-label="Randomize seed" title="Randomize seed">${icon('rotate-ccw')}</button></div>
      </section>
      <div class="panel-foot">NO DESTINATION. ONLY POSSIBILITY.</div>
    </aside>
    <main class="experiment">
      <div class="experiment-heading"><div><span class="eyebrow" id="experiment-label">EXPERIMENT 001 / SEED 8042</span><h2>A world becoming alive<span>.</span></h2></div><button id="settings" class="icon-button" title="Graphics quality" aria-label="Toggle graphics quality">${icon('settings-2')}</button></div>
      <div class="stats-bar"><div><span>Organisms</span><strong id="population">—</strong><small id="budget-note">of 4,000 capacity</small></div><div><span>Living cells</span><strong id="cells">—</strong><small id="colony-note">waiting for life</small></div><div><span id="diversity-label">Lineages</span><strong id="diversity">—</strong><small id="diversity-note">distinct genetic branches</small></div><div><span>Generation</span><strong id="generation">—</strong><small>maximum descent</small></div></div>
      <div class="world-stage"><div id="world-canvas" aria-label="Interactive 3D ecosystem. Drag to orbit; scroll to zoom; click a cell to inspect."></div>
        <div class="stage-top"><div><span class="stage-badge"><span class="live-dot"></span><span id="run-state">EVOLVING</span></span><span class="world-name" id="world-name">TIDAL GARDEN</span></div><div class="view-tools"><button id="resource-layer" class="stage-button" title="Show nutrient field" aria-label="Show nutrient field">${icon('layers')}</button><button id="reset-camera" class="stage-button" title="Reset camera" aria-label="Reset camera">${icon('maximize-2')}</button></div></div>
        <div id="loading" class="loading"><span class="loading-orbit"></span><strong>Preparing a possibility</strong><span>Initializing the Rust life engine</span></div>
        <div class="stage-bottom"><span class="orbit-hint">${icon('orbit')} Drag to orbit <b>·</b> Scroll to explore <b>·</b> Click to inspect</span><span id="perf">RUST + WASM</span></div>
        <div class="world-coordinate">LOCAL BIOSPHERE<br><span>Illustrative cellular scale</span></div>
      </div>
      <section class="timeline"><div class="timeline-top"><div class="playback"><button id="pause" class="play-button" aria-label="Pause evolution">${icon('pause')}</button><div class="speed-controls"><label for="simulation-speed">Speed</label><select id="simulation-speed">${[1,10,50,100,500,1000,10000,0].map(s=>`<option value="${s}">${s===0?'Max':`${s.toLocaleString()}×`}</option>`).join('')}</select><span id="achieved-speed">0.0× actual</span></div></div><span class="cycle"><strong id="time">0.0</strong> cycles elapsed</span><button id="pulse" class="text-button" title="Add nutrients to the world">${icon('droplets')} Nutrient pulse</button></div>
        <div class="chart-heading"><span>LIFE THROUGH TIME</span><div><span class="chart-legend"><b></b> Organisms</span><span class="chart-legend colony"><b></b> Colonies</span></div></div>
        <canvas id="history-chart" aria-label="Population and colony counts through time"></canvas>
        <div class="chart-axis"><span id="chart-start">0 cycles</span><span>Natural selection has no finish line.</span><span id="chart-end">now</span></div>
      </section>
    </main>
    <aside class="life-panel"><div class="panel-tabs"><button data-tab="life" class="active">Lifeforms</button><button data-tab="notes">Field notes <span id="event-dot"></span></button></div>
      <div id="life-content"><div class="specimen-heading"><span class="eyebrow">UNDER THE LENS</span><button id="focus" class="icon-button" title="Follow this lifeform" aria-label="Follow this lifeform">${icon('focus')}</button></div>
        <div id="specimen-canvas"></div><div class="specimen-info"><span class="specimen-type" id="specimen-type">THE FIRST CELL</span><h3 id="specimen-name">Waiting for life</h3><p id="specimen-description">A small beginning. An open future.</p></div>
        <div class="specimen-stats"><div><span>Cells</span><strong id="organism-cells">—</strong></div><div><span>Generation</span><strong id="organism-gen">—</strong></div><div><span>Energy</span><strong id="organism-energy">—</strong></div></div>
        <div class="traits-heading">INHERITED FUNCTIONS <span id="organism-id"></span></div><div id="traits"></div>
        <div class="roles" id="roles"></div><div id="body-mechanics" class="body-mechanics"></div><section id="joint-mechanics" class="cell-ecology" hidden></section><section id="cell-ecology" class="cell-ecology" hidden></section>
        <section id="autonomous-results" class="cell-ecology" hidden></section><section id="population-results" class="cell-ecology" hidden><span class="eyebrow">COMPETING ECOSYSTEM / ECOLOGICAL SELECTION</span><button id="toggle-sensing" class="observe-button" data-sensing>Disable directional sensing</button><div id="population-details"></div></section><section id="lab-results" class="lab-results" hidden><span class="eyebrow">CONTROLLED LOCOMOTION ASSAY</span><h3 id="lab-phase">Developing</h3><p id="lab-trial"></p><progress id="lab-progress" max="20" value="0"></progress><div class="lab-metrics"><div><span>Net travel</span><strong id="lab-distance">0.00</strong></div><div><span>Motor work</span><strong id="lab-work">0.00</strong></div><div><span>Champion travel</span><strong id="lab-best">—</strong></div><div><span>Best score</span><strong id="lab-score">—</strong></div></div><p id="lab-improvement"></p><small>Each candidate grows from one cell. Travel is measured after development and settling. Selection rewards net travel and penalizes work and collapse. Units are model units.</small><div id="lab-rounds"></div><button id="replay-best" class="observe-button">Watch best body</button><button id="toggle-motors" class="observe-button" hidden>Disable joint motors</button><button id="continue-selection" class="observe-button" hidden>Continue selection</button></section><button id="observe" class="observe-button">${icon('microscope')} Find a complex lifeform ${icon('arrow-up-right')}</button>
        <div class="lineage-heading"><span>LIVING LINEAGES</span><span id="lineage-count">0</span></div><div id="lineages"></div>
        <button id="seed-life" class="seed-button">${icon('plus')} Introduce 80 protocells</button>
      </div>
      <div id="notes-content" hidden><div class="notes-intro"><span class="eyebrow">AN EVOLUTIONARY JOURNAL</span><h3>Small changes.<br>New possibilities.</h3><p>Milestones record what actually happened in this world.</p></div><div id="events"></div><div class="journal-stats" id="journal-stats"></div></div>
      <div class="science-note">${icon('flask-conical')} A model of possibility.<br><span>Not a prediction of extraterrestrial life.</span></div>
    </aside>
  </div>
  <div id="toast" role="status"></div><input id="file-input" type="file" accept="application/json,.json" hidden>
  <dialog id="help-dialog"><button id="close-help" class="icon-button" aria-label="Close help">${icon('x')}</button><span class="eyebrow">PHYSICAL HABITAT PREVIEW</span><h2>Materials before traits.</h2><p>Your chosen number of supplied replicating cells inhabits finite chemical pools. Genes control synthesis of catalysts, adhesive and contractile polymers. Material stores and mineral deposition affect the physical body.</p><div class="help-grid"><div><h3>Fluid and diffusion</h3><p>A low-speed horizontal fluid solver transports solutes. Cell drag transfers momentum back to the fluid. The layers button shows measured currents and nutrients.</p></div><div><h3>Inheritance</h3><p>Fission splits existing matter and copies genes with random changes. Survival and reproduction select descendants; there is no tissue unlock or movement score.</p></div><div><h3>Conserved chemistry</h3><p>Fuel, waste, signals and minerals have finite stores. Light supplies energy. Synthesis, maintenance and contraction spend it. Death recycles material.</p></div><div><h3>Model limits</h3><p>The reaction network and catalyst functions are designed approximations. This is not atomistic chemistry or molecular abiogenesis. Tissue, organs and walking have not been demonstrated.</p></div></div><p class="help-small">Physics uses nondimensional model units. The solvent is a 40×40 horizontal lattice, while cell mechanics uses 3D positions. Numerical mechanics and chemical energy accounting are separate. Older labs retain their supplied organism rules.</p><button id="start-exploring" class="primary-button">Explore the habitat ${icon('arrow-up-right')}</button></dialog>
`;

const sliderDefs:[keyof WorldConfig,string,string,string][]=[
  ['sunlight','Starlight','sun','Energy for light-harvesting cells'],['nutrients','Nutrients','droplets','Regeneration of consumable resources'],['minerals','Minerals','mountain','Energy for mineral-metabolizing cells'],['temperature','Temperature','thermometer','Selects for inherited thermal tolerance'],['radiation','Radiation','zap','Raises damage and mutation pressure'],['solvent','Solvent availability','droplets','Controls the efficiency of all metabolisms'],['mutation','Mutation rate','dna','Probability of a change per gene per birth'],
];
$('sliders').innerHTML=sliderDefs.map(([key,label,ico,tip])=>`<div class="slider-row"><label for="${key}" title="${tip}"><span>${icon(ico)}${label}</span><output id="${key}-value">${Math.round((config[key] as number)*100)}%</output></label><input id="${key}" type="range" min="0" max="100" value="${(config[key] as number)*100}" aria-label="${label}" style="--value:${(config[key] as number)*100}%"></div>`).join('');
createIcons({icons});
function toast(message:string){$('toast').textContent=message;$('toast').classList.add('show');setTimeout(()=>$('toast').classList.remove('show'),3500);}
try { view=new WorldView($('world-canvas'),id=>select(id)); specimen=new SpecimenView($('specimen-canvas')); }
catch(error){$('loading').innerHTML=`<strong>3D graphics unavailable</strong><span>This world needs a browser with WebGL 2 enabled.</span>`;throw error;}
let experiment=1;
function syncControls(){
  sliderDefs.forEach(([key])=>{const value=Math.round((config[key] as number)*100);$(key) instanceof HTMLInputElement&&($(key) as HTMLInputElement).setAttribute('value',String(value));$(key+'-value').textContent=`${value}%`;($(key) as HTMLInputElement).value=String(value);$(key).style.setProperty('--value',`${value}%`);});
  $<HTMLInputElement>('initial-cells').value=String(config.initialCells??64);$<HTMLInputElement>('cell-budget').value=String(config.capacity);$<HTMLInputElement>('setup-seed').value=String(config.seed);
  $<HTMLInputElement>('seed').value=String(config.seed);$<HTMLSelectElement>('capacity').value=String(config.capacity);$<HTMLInputElement>('chemistry').checked=config.origin==='chemistry';
  document.querySelectorAll<HTMLElement>('[data-world]').forEach(el=>el.classList.toggle('active',el.dataset.world===config.kind));
  $('world-name').textContent=config.name.replace(/^The /,'').toUpperCase();$('experiment-label').textContent=`EXPERIMENT ${String(experiment).padStart(3,'0')} / SEED ${config.seed}`;
}
function createWorld(){
  const seed=Number($<HTMLInputElement>('seed').value);
  if(!Number.isInteger(seed)||seed<0||seed>4294967295){toast('Choose a seed from 0 to 4,294,967,295.');return;}
  config.initialCells=Number($<HTMLInputElement>('initial-cells').value);config.capacity=Number($<HTMLInputElement>('cell-budget').value);config.physicalAssay='full';
  if(!validSetup(config))return;
  config.seed=seed;config.origin=$<HTMLInputElement>('chemistry').checked?'chemistry':'seeded';experiment++;
  selectedId=null;autoselect=true;paused=false;view.select(null);specimen.update(null);send({type:'init',config});send({type:'control',speed,paused:false});syncControls();updatePause();toast('A new world. A different possibility.');
}
function validSetup(c:WorldConfig){
  if(!Number.isInteger(c.initialCells)||c.initialCells!<1||c.initialCells!>8192){toast('Starting organisms must be a whole number from 1 to 8,192.');return false;}
  if(!Number.isInteger(c.capacity)||c.capacity<100||c.capacity>64000||c.capacity<c.initialCells!){toast('Choose a cell ceiling from 100 to 64,000 that fits your starting population.');return false;}
  if(!Number.isInteger(c.seed)||c.seed<0||c.seed>4294967295){toast('Choose a valid world seed.');return false;}
  return true;
}
function physicalSetup():WorldConfig {
  return {...config, initialCells:Number($<HTMLInputElement>('initial-cells').value),capacity:Number($<HTMLInputElement>('cell-budget').value),seed:Number($<HTMLInputElement>('setup-seed').value),origin:'seeded',physicalAssay:'full'};
}
function createPhysical(){const next=physicalSetup();if(!validSetup(next))return;config=next;selectedId=null;autoselect=true;paused=false;view.select(null);send({type:'init',config});send({type:'control',paused:false,speed});updatePause();toast(`${config.initialCells} basic cells; ${fmt(config.capacity)} living-cell ceiling.`);}
$('create-physical').addEventListener('click',createPhysical);
$('undo-world').addEventListener('click',()=>{paused=true;selectedId=null;autoselect=true;view.select(null);send({type:'undo-world'});updatePause();});
$('apply-budget').addEventListener('click',()=>{const capacity=Number($<HTMLInputElement>('cell-budget').value);if(!Number.isInteger(capacity)){toast('Choose a whole-number ceiling.');return;}send({type:'cell-budget',capacity});});
for(const [id,assay] of [['shared-passive','passive'],['shared-scarcity','scarcity']] as const){$(id).addEventListener('click',()=>{
  const next=physicalSetup();if(!validSetup(next))return;next.physicalAssay=assay;if(assay==='scarcity')next.nutrients=.08;
  selectedId=null;autoselect=true;view.select(null);send({type:'physical-study',config:next});toast('Physical world held. The experiment uses the same cells, chemistry, joints and inheritance engine.');
});}
$('return-physical').addEventListener('click',()=>{selectedId=null;autoselect=true;view.select(null);send({type:'return-physical'});});
function updatePause(){ $('pause').innerHTML=icon(paused?'play':'pause');$('pause').setAttribute('aria-label',paused?'Resume evolution':'Pause evolution');$('run-state').textContent=paused?'PAUSED':'EVOLVING';document.querySelector('.stage-badge')!.classList.toggle('paused',paused);createIcons({icons}); }
function select(id:number|null,zoom=false){selectedId=id;send({type:'select',id});view.select(id,zoom);}
sliderDefs.forEach(([key])=>$(key).addEventListener('input',()=>{const value=Number($<HTMLInputElement>(key).value)/100;config={...config,[key]:value};$(key+'-value').textContent=`${Math.round(value*100)}%`;$(key).style.setProperty('--value',`${value*100}%`);send({type:'config',patch:{[key]:value}});}));
$('capacity').addEventListener('change',()=>{config.capacity=Number($<HTMLSelectElement>('capacity').value);send({type:'config',patch:{capacity:config.capacity}});});
document.querySelectorAll<HTMLElement>('[data-world]').forEach(el=>el.addEventListener('click',()=>{const kind=el.dataset.world as Exclude<WorldKind,'lab'>;config={...PRESETS[kind]};syncControls();createWorld();}));
$('new-world').addEventListener('click',createWorld);
$('random-seed').addEventListener('click',()=>{$<HTMLInputElement>('seed').value=String(crypto.getRandomValues(new Uint32Array(1))[0]);toast('New seed chosen. Create a world to use it.');});
$('pause').addEventListener('click',()=>{paused=!paused;send({type:'control',paused});updatePause();});
$('simulation-speed').addEventListener('change',()=>{speed=Number($<HTMLSelectElement>('simulation-speed').value);send({type:'control',speed});});
$('start-autonomous').addEventListener('click',createPhysical);
$('start-population').addEventListener('click',()=>{selectedId=null;autoselect=true;send({type:'competition',action:'start',count:Number($<HTMLSelectElement>('competing-founders').value)});view.select(null);toast('Your ecosystem is held. Supplied single cells compete, grow and reproduce together.');});
$('population-results').addEventListener('click',event=>{const target=event.target as HTMLElement;const sensing=target.closest<HTMLElement>('[data-sensing]');if(sensing){send({type:'competition',action:sensing.textContent?.startsWith('Disable')?'sense-off':'sense-on',count:0});return;}const family=target.closest<HTMLElement>('[data-family]');if(family&&snapshot){const row=snapshot.competition?.families.find(f=>f.id===Number(family.dataset.family));if(row?.representative)select(row.representative,true);}});
$('start-tissue').addEventListener('click',()=>{selectedId=1;autoselect=false;send({type:'lab',action:'study'});view.select(1);toast('Your ecosystem is held. The study starts with 80 single cells and one supplied precursor.');});
$('start-joints').addEventListener('click',()=>{selectedId=1;autoselect=false;send({type:'lab',action:'joints'});view.select(1);toast('Your ecosystem is held. One supplied cell develops jointed segments and inherited controllers.');});
$('toggle-motors').addEventListener('click',()=>send({type:'lab',action:snapshot?.lab?.motorsEnabled?'motors-off':'motors-on'}));
$('start-lab').addEventListener('click',()=>{selectedId=1;autoselect=false;send({type:'lab',action:'start'});view.select(1);toast('Your ecosystem is held. The lab starts with one supplied precursor cell.');});
$('replay-best').addEventListener('click',()=>send({type:'lab',action:'replay'}));
$('continue-selection').addEventListener('click',()=>send({type:'lab',action:'evolve'}));
$('leave-lab').addEventListener('click',()=>{selectedId=null;autoselect=true;send({type:'lab',action:'leave'});view.select(null);toast('Returning to the ecosystem.');});
$('pulse').addEventListener('click',()=>{send({type:'pulse'});toast('Nutrients added. Watch how the population responds.');});
$('seed-life').addEventListener('click',()=>{if(snapshot?.competition){send({type:'competition',action:'introduce',count:12});toast('12 supplied competing cells introduced where capacity allows.');}else{send({type:'seed',count:80});toast('80 founder cells introduced where capacity allows.');}autoselect=true;});
$('resource-layer').addEventListener('click',()=>{view.setOverlay(!view.overlay);$('resource-layer').classList.toggle('active',view.overlay);toast(view.overlay?(snapshot?.autonomous?.physics?'Nutrients and measured fluid currents.':'Nutrient field: green is abundant; amber is depleted.'):'Habitat layers hidden.');});
$('reset-camera').addEventListener('click',()=>view.resetCamera());
$('settings').addEventListener('click',()=>{view.setQuality(view.quality==='high'?'low':'high');toast(`Graphics: ${view.quality==='high'?'high · bloom and shadows':'performance · reduced GPU work'}`);});
$('focus').addEventListener('click',()=>{if(selectedId)select(selectedId,true);else toast('Select a lifeform first.');});
$('observe').addEventListener('click',()=>{
  if(!snapshot?.ids.length){toast('This world has no living organisms yet.');return;}
  let best=0;for(let i=1;i<snapshot.ids.length;i++){if(snapshot.data[i*STRIDE+3]>snapshot.data[best*STRIDE+3])best=i;}
  select(snapshot.ids[best],true);
});
document.querySelectorAll<HTMLElement>('[data-tab]').forEach(el=>el.addEventListener('click',()=>{tab=el.dataset.tab!;document.querySelectorAll('[data-tab]').forEach(e=>e.classList.toggle('active',e===el));$('life-content').hidden=tab!=='life';$('notes-content').hidden=tab!=='notes';}));
$('lineages').addEventListener('click',e=>{const button=(e.target as HTMLElement).closest<HTMLElement>('[data-lineage]');if(!button||!snapshot)return;const l=Number(button.dataset.lineage),line=snapshot.lineages.find(x=>x.id===l);if(!line)return;
  // The rendering buffer contains no lineage IDs; ask the core by the representative ID supplied in metadata.
  const representative=(line as typeof line & {representative?:number}).representative;if(representative)select(representative,true);
});
$('help').addEventListener('click',()=>$<HTMLDialogElement>('help-dialog').showModal());
for(const id of ['close-help','start-exploring'])$(id).addEventListener('click',()=>$<HTMLDialogElement>('help-dialog').close());
$('save').addEventListener('click',()=>send({type:'save'}));
$('restore').addEventListener('click',async()=>{try{const json=await readWorld();if(!json){toast('There is no local save yet.');return;}send({type:'load',data:JSON.parse(json),restore:true});selectedId=null;view.select(null);autoselect=true;}catch{toast('Could not read the local save. You can open an exported JSON file.');}});
readWorld().then(json=>{$<HTMLButtonElement>('restore').disabled=!json;}).catch(()=>{});
$('load').addEventListener('click',()=>$('file-input').click());
$('file-input').addEventListener('change',async()=>{
  const file=$<HTMLInputElement>('file-input').files?.[0];if(!file)return;
  try{if(file.size>256*1024*1024)throw new Error('Save file is too large (256 MB limit).');const data=JSON.parse(await file.text());send({type:'load',data});selectedId=null;view.select(null);autoselect=true;}catch(error){toast(String(error));}
  $<HTMLInputElement>('file-input').value='';
});
function drawChart(){
  if(!snapshot)return;const canvas=$<HTMLCanvasElement>('history-chart'),rect=canvas.getBoundingClientRect(),dpr=Math.min(devicePixelRatio,2);canvas.width=rect.width*dpr;canvas.height=rect.height*dpr;
  const ctx=canvas.getContext('2d')!;ctx.scale(dpr,dpr);const w=rect.width,h=rect.height,history=snapshot.history;
  if(snapshot.lab){
    const results=snapshot.lab.results,maximum=Math.max(.1,...results.map(r=>r.distance));
    document.querySelector('.chart-heading > span')!.textContent='MEASURED NET TRAVEL / COMPLETED CANDIDATES';
    document.querySelector('.chart-legend')!.innerHTML='<b></b> Distance';document.querySelector('.chart-legend.colony')!.innerHTML='<b></b> Farthest trial';
    ctx.strokeStyle='#ffffff0c';for(let y=0;y<3;y++){ctx.beginPath();ctx.moveTo(0,4+y*(h-10)/3);ctx.lineTo(w,4+y*(h-10)/3);ctx.stroke();}
    ctx.beginPath();results.forEach((r,i)=>{const x=i/Math.max(1,results.length-1)*w,y=h-4-r.distance/maximum*(h-10);i?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.strokeStyle='#b7df9c';ctx.lineWidth=1.7;ctx.stroke();
    let best=0;ctx.beginPath();results.forEach((r,i)=>{best=Math.max(best,r.distance);const x=i/Math.max(1,results.length-1)*w,y=h-4-best/maximum*(h-10);i?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.strokeStyle='#9ebdbc';ctx.lineWidth=1;ctx.stroke();
    $('chart-start').textContent='Equal-duration trials';$('chart-end').textContent=`${results.length} recent candidates`;return;
  }
  document.querySelector('.chart-heading > span')!.textContent='LIFE THROUGH TIME';document.querySelector('.chart-legend')!.innerHTML='<b></b> Organisms';document.querySelector('.chart-legend.colony')!.innerHTML='<b></b> Colonies';
  if(history.length<2){$('chart-start').textContent=`${fmt(history[0]?.time??0)} cycles`;$('chart-end').textContent='now';return;}
  const max=Math.max(200,...history.map(p=>p.population));
  ctx.strokeStyle='#ffffff0c';ctx.lineWidth=1;for(let y=0;y<3;y++){ctx.beginPath();ctx.moveTo(0,4+y*(h-10)/3);ctx.lineTo(w,4+y*(h-10)/3);ctx.stroke();}
  const t0=history[0].time,t1=Math.max(t0+1,history[history.length-1].time);
  const path=(key:'population'|'colonies')=>{ctx.beginPath();history.forEach((p,i)=>{const x=(p.time-t0)/(t1-t0)*w,y=h-4-p[key]/max*(h-10);i?ctx.lineTo(x,y):ctx.moveTo(x,y);});};
  path('population');ctx.lineTo(w,h);ctx.lineTo(0,h);ctx.closePath();const gradient=ctx.createLinearGradient(0,0,0,h);gradient.addColorStop(0,'#bdea9d35');gradient.addColorStop(1,'#bdea9d00');ctx.fillStyle=gradient;ctx.fill();
  path('population');ctx.strokeStyle='#b7df9c';ctx.lineWidth=1.7;ctx.stroke();path('colonies');ctx.strokeStyle='#9ebdbc';ctx.lineWidth=1.3;ctx.stroke();
  $('chart-start').textContent=`${fmt(t0)} cycles`;$('chart-end').textContent=`${fmt(t1)} cycles`;
}
function physicsPanel(p:import('./sim/types').PhysicsView){
  return `<span class="eyebrow">PHYSICAL HABITAT / CONSERVATION</span><p>Finite molecules diffuse and travel with a viscous fluid. Cells build proteins and bind minerals from actual stores. Sunlight supplies tracked energy; material recycles after death. Conditions stay fixed.</p><p>${p.assay==='passive'?'PASSIVE ASSAY · actuation disabled':'Shared engine · full material mechanics'} · ${fmt(p.joints)} material bending joints · ${p.angularWork.toFixed(4)} angular motor work</p><div class="cell-summary"><span>Organic matter<strong>${p.organic.toFixed(2)}</strong></span><span>Organic balance error<strong>${p.massError.toExponential(1)}</strong></span><span>Mineral balance error<strong>${p.mineralError.toExponential(1)}</strong></span><span>Chemical energy balance error<strong>${p.energyError.toExponential(1)}</strong></span><span>Cell protein mass<strong>${p.proteinMass.toFixed(3)}</strong></span><span>Cell-bound minerals<strong>${p.boundMineral.toFixed(3)}</strong></span><span>Contact bindings<strong>${p.boundEvents}</strong></span><span>Bond dissociations<strong>${p.dissociations}</strong></span><span>Maximum fluid speed<strong>${p.fluidSpeed.toFixed(4)}</strong></span><span>Energy supplied by light<strong>${p.photonEnergy.toFixed(2)}</strong></span></div><p>Use the layers button to show nutrient availability and fluid arrows. Arrows magnify measured flow 18×. This is a reduced chemistry model and a horizontal fluid grid, in model units. Organs and walking remain possible research outcomes, not promised milestones.</p>`;
}
function updateUI(s:Snapshot){
  const openPrograms=new Set(Array.from($('body-mechanics').querySelectorAll<HTMLDetailsElement>('details[open]')).map(el=>el.dataset.programCell));
  const st=s.stats;config=s.config;
  const auto=s.autonomous;$('mechanics-description').textContent=auto?'Cells copy and mutate variable genomes. Local signals regulate what they do; resources and actual cell survival select descendants. The older supplied-trait studies remain available for comparison.':'Attached cells can digest smaller lifeforms, grow support segments and paired muscles, and inherit movement controllers. Explore their development in the lab.';const lab=s.lab,study=s.study,competition=s.competition,sharedStudy=!!auto?.physics&&s.config.physicalAssay!=='full'&&s.config.physicalAssay!==undefined,inStudy=!!lab||study||!!competition||sharedStudy;$('lab-results').hidden=!lab;$('start-lab').hidden=inStudy;$('start-joints').hidden=inStudy;$('start-tissue').hidden=inStudy;$('competition-entry').hidden=inStudy;$('leave-lab').hidden=!inStudy;
  $('start-autonomous').hidden=inStudy;$('start-autonomous').textContent=auto?.physics?'Restart physical habitat':auto?'Restart autonomous world':'Open physical habitat';
  $<HTMLButtonElement>('undo-world').disabled=!s.canUndoWorld;
  $('run-setup').hidden=!!lab||study||!!competition;
  $('return-physical').hidden=!sharedStudy;
  for(const id of ['shared-passive','shared-scarcity','create-physical'])$<HTMLButtonElement>(id).disabled=sharedStudy;
  $('leave-lab').hidden=!(!!lab||study||!!competition);
  $('autonomous-results').hidden=!auto;
  if(auto){$('autonomous-results').innerHTML=`${auto.physics?physicsPanel(auto.physics):''}<span class="eyebrow">AUTONOMOUS CELLULAR EVOLUTION</span><p>Each cell owns a variable genome. Division copies it with occasional edits, duplication, deletion or new genes. Genes regulate catalyst and material production in physical habitats. Local resources and signals influence expression. Connected daughters form bodies; there are no tissue stages or target body sizes.</p><div class="cell-summary"><span>Cell divisions<strong>${fmt(auto.divisions)}</strong></span><span>Attached daughters<strong>${fmt(auto.attachedDivisions)}</strong></span><span>Living genome variants<strong>${fmt(auto.genomes)}</strong></span><span>Genes per cell<strong>${auto.genomeMin}–${auto.genomeMax}</strong></span><span>Point edits<strong>${fmt(auto.pointEdits)}</strong></span><span>Duplications / deletions<strong>${auto.duplications} / ${auto.deletions}</strong></span><span>New action genes<strong>${fmt(auto.innovations)}</strong></span><span>Contact gene transfers<strong>${fmt(auto.exchanges)}</strong></span><span>${auto.physics?'Energy released by metabolism':'Environmental food energy'}<strong>${auto.foodEnergy.toFixed(1)}</strong></span><span>Paid contraction work<strong>${auto.motorWork.toFixed(2)}</strong></span><span>Largest connected body<strong>${auto.peakBody} cells</strong></span><span>Maximum descent<strong>G${auto.peakGeneration}</strong></span></div><span class="eyebrow">EXPRESSED GENETIC PROGRAMS</span><div class="cell-table">${auto.actions.map((name,i)=>`<div><span>${esc(name)}</span><span>${auto.expressed[i]} cells</span></div>`).join('')}</div><p>The founder supplies four genes: light harvesting, nutrient transport, replication and weak adhesion. Other production pathways require gene changes. These are defined chemical building blocks, not evolved organs. There is no automatic rescue; extinction is possible. Conditions and mutation protocol are fixed for the run.</p><span class="eyebrow">RECENT CELL DIVISIONS</span><div class="ancestry-list">${auto.recent.map(r=>`<div>G${r.generation} · #${r.child} ← #${r.parent}<small>${r.attached?'Attached':'Separated'} · genome ${r.genome} · ${r.edits} edits · ${r.time.toFixed(1)} cycles</small></div>`).join('')||'No divisions yet.'}</div>`;}
  $('seed-life').innerHTML=icon('plus')+(competition?' Introduce 12 competing cells':' Introduce 80 protocells');
  $('population-results').hidden=!competition;
  if(competition){
    $('toggle-sensing').textContent=competition.directionalSensing?'Disable directional sensing':'Enable directional sensing';
    $('population-details').innerHTML=`<p>Supplied single-cell families share finite food. Survival and offspring determine which traits spread. Founder labels describe starting biases, not fixed species. Nutrients controls resource capacity and renewal; reduce it to increase scarcity.</p><p>Directional sensing: ${competition.directionalSensing?'on':'off'}. Disabling removes food, prey and mate direction from steering and muscle control. Contact, joint feedback, tissue regulation and sensing costs remain. Save first, then restore before changing the setting for an equal-start comparison. Food energy counts environmental uptake, excluding prey meals; older saves begin tracking on import.</p><div class="cell-summary"><span>Food energy acquired<strong>${competition.foodEnergy.toFixed(1)}</strong></span><span>Offspring born<strong>${fmt(st.births)}</strong></span><span>Gene exchanges<strong>${fmt(st.mating)}</strong></span><span>Cells eaten<strong>${fmt(st.feedings)}</strong></span><span>Food-poor locations<strong>${Math.round(competition.foodPoorFraction*100)}%</strong></span><span>Jointed bodies / peak<strong>${st.jointed} / ${competition.peakJointed}</strong></span><span>Largest grown body<strong>${competition.largestBody} cells</strong></span><span>Maximum descent<strong>G${competition.peakGeneration}</strong></span></div><div class="family-list">${competition.families.map(f=>`<button data-family="${f.id}" ${f.representative?'':'disabled'}><strong>${esc(f.name)}</strong><span>${f.living} living · ${f.births} offspring · ${f.meals} meals · G${f.generation}</span><small>${f.exchanges} exchanges · ${f.deaths} deaths · digestion ${Math.round(f.meanPredation*100)}% · controller change ${(f.controllerChange*100).toFixed(1)}%</small></button>`).join('')}</div><p>Family totals follow the reproducing parent; mating also contributes genes. Controller change is distance from that family's starting weights, not a fitness score. Food-poor locations have less than 0.5 model resource units. Extinction remains possible; introductions are manual.</p><span class="eyebrow">RECENT ANCESTRY</span><div class="ancestry-list">${competition.ancestry.length?competition.ancestry.slice(0,8).map(b=>`<div>G${b.generation} · #${b.child} ← #${b.parent}${b.mate?` + #${b.mate}`:''}<small>${b.time.toFixed(1)} cycles</small></div>`).join(''):'<p>Retained cell division grows bodies. Independent offspring will appear here.</p>'}</div>`;
  }
  document.querySelector<HTMLElement>('.conditions')!.hidden=!!lab||!!auto;
  document.querySelector<HTMLElement>('.origin-section')!.hidden=!!lab||!!auto;
  for(const id of ['pulse','seed-life','new-world','chemistry','random-seed','seed'])$<HTMLButtonElement>(id).disabled=!!lab||!!auto;
  $<HTMLButtonElement>('resource-layer').disabled=!!lab;
  for(const id of ['sunlight','nutrients','minerals','temperature','radiation','solvent','mutation','capacity'])$<HTMLInputElement>(id).disabled=!!lab||!!auto;
  document.querySelector('.world-coordinate > span')!.textContent=auto?.physics?'PHYSICAL PREVIEW / fluid, chemistry and cellular heredity':auto?'Cell-local genomes / division / ecological selection':lab?(lab.jointed?'Angular joints / paired motors / inherited control':'Physical cell bodies / contact assay'):competition?'Competing bodies / survival and reproduction':study?'Controlled tissue ecology / supplied precursor':'Cellular terrain ecosystem';
  if(lab){
    $('toggle-motors').hidden=!lab.jointed;$('toggle-motors').textContent=lab.motorsEnabled?'Disable joint motors':'Enable joint motors';
    $('replay-best').hidden=lab.replaying;$<HTMLButtonElement>('replay-best').disabled=!lab.champion;$('continue-selection').hidden=!lab.replaying;
    $('lab-phase').textContent=lab.replaying?`Best body / ${lab.phase.toLowerCase()}`:lab.phase;$('lab-trial').textContent=!lab.motorsEnabled?'Motors disabled · selection held':lab.replaying?'Champion replay · selection held':`Generation ${lab.generation+1} · Candidate ${lab.candidate+1} / ${lab.candidates}`;
    $<HTMLProgressElement>('lab-progress').value=lab.elapsed;
    $('lab-distance').textContent=lab.distance.toFixed(2);$('lab-work').textContent=lab.work.toFixed(2);
    $('lab-best').textContent=lab.champion?.distance.toFixed(2)??'—';$('lab-score').textContent=lab.champion?.score.toFixed(2)??'—';
    $('lab-improvement').textContent=lab.champion&&lab.baseline?`Baseline score ${lab.baseline.score.toFixed(2)} · improvement ${(lab.champion.score-lab.baseline.score).toFixed(2)}`:'First trial establishes the baseline. Try 10× or 50× to compare descendants.';
    $('lab-rounds').innerHTML=lab.progress.slice(-4).reverse().map(r=>`<div><span>Generation ${r.generation+1}</span><strong>${r.distance.toFixed(2)} travel · ${r.score.toFixed(2)} score</strong></div>`).join('');
  }

  $('population').textContent=fmt(st.population);$('cells').textContent=fmt(st.cells);$('diversity').textContent=fmt(st.diversity);$('generation').textContent=fmt(st.generation);
  $('budget-note').textContent=auto?`${fmt(st.cells)} / ${fmt(st.cellBudget)} cells${st.cellBudgetReached?' · ceiling reached':''}`:st.cellBudgetReached?'Cell physics budget reached':st.capped?'Population budget reached':`of ${fmt(config.capacity)} capacity`;
  $('colony-note').textContent=`${fmt(st.colonies)} multicellular colonies`;$('time').textContent=st.time.toFixed(1);
  $('achieved-speed').textContent=`${(st.actualSpeed??0).toFixed(1)}× actual${paused?' · paused':speed>0&&(st.actualSpeed??0)<speed*.8?' · CPU limited':''}`;
  $('perf').textContent=`${Math.round(view.fps)} FPS · ${st.computeMs.toFixed(1)} MS / TICK`;
  $('lineage-count').textContent=fmt(st.diversity);
  $('diversity-label').textContent=auto?'Genomes':'Lineages';$('diversity-note').textContent=auto?'living sequence variants':'distinct genetic branches';document.querySelector('.lineage-heading > span')!.textContent=auto?'BODY REPRESENTATIVES':'LIVING LINEAGES';
  $('lineages').innerHTML=s.lineages.slice(0,6).map(l=>`<button class="lineage-row" data-lineage="${l.id}" title="Inspect a member of this lineage"><span class="lineage-dot" style="--lineage-color:${/^#[0-9a-f]{6}$/i.test(l.color)?l.color:'#a9cca0'}"></span><span><strong>${esc(l.name.split(' · ')[0])}</strong><small>${fmt(l.count)} organisms · G${l.generation}</small></span><span class="lineage-percent">${Math.round(l.count/Math.max(1,st.population)*100)}%</span></button>`).join('')||(auto?'<p class="empty-note">No surviving bodies. This run remains extinct.</p>':'<p class="empty-note">No surviving lineages.<br>Life may begin again.</p>');
  $('events').innerHTML=s.events.slice().reverse().map(e=>`<div class="event"><span class="event-symbol">${e.type==='colony'?'✧':e.type==='extinction'?'○':e.type==='sex'?'◇':'•'}</span><div><span>${e.time.toFixed(1)} CYCLES</span><p>${esc(e.text)}</p></div></div>`).join('')||'<p class="empty-note">Chemistry is stirring.<br>The first chapter is still unwritten.</p>';
  $('journal-stats').innerHTML=`<div><span>Reproduction events</span><strong>${fmt(st.births)}</strong></div><div><span>${auto?'Cells lost':'Organisms lost'}</span><strong>${fmt(st.deaths)}</strong></div><div><span>Gene exchanges</span><strong>${fmt(st.mating)}</strong></div><div><span>${auto?'Contact digestion events':'Cells consumed'}</span><strong>${fmt(st.feedings)}</strong></div><div><span>Injured colonies</span><strong>${fmt(st.injured)}</strong></div><div><span>Living cell budget</span><strong>${fmt(st.cells)} / ${fmt(st.cellBudget)}</strong></div>`;
  const o=s.selected;
  specimen.update(o);
  if(o){
    $('specimen-type').textContent=o.cells===1?'UNICELLULAR':o.cells<4?'SIMPLE COLONY':'MULTICELLULAR';$('specimen-name').textContent=lab?(lab.replaying?'Champion body':`Candidate ${lab.candidate+1}`):o.name.split(' · ')[0];$('organism-id').textContent=`#${o.id}`;
    $('specimen-description').textContent=lab?'A supplied precursor. Its developmental and motor program varies through artificial selection.':o.meals>0?'A colony that supplements its metabolism by digesting smaller lifeforms. Food can fund growth and offspring.':study&&o.lineage===9?'A supplied precursor lineage. Cells grow and specialize under local food and stress; survival and reproduction select descendants.':o.genes[GI.photo]>o.genes[GI.chemo]?'A lineage that draws energy from light.':'A lineage that draws energy from minerals.';
    $('organism-cells').textContent=fmt(o.cells);$('organism-gen').textContent=fmt(o.generation);$('organism-energy').textContent=o.energy.toFixed(1);
    const traitDefs:[string,number][]=[['Light harvesting',GI.photo],['Chemosynthesis',GI.chemo],['Cell adhesion',GI.adhesion],['Contractile potential',GI.muscle],['Structural support',GI.skeleton],['Specialization',GI.specialization],['Cell digestion',GI.predation],['Resource sensing',GI.sensing],['Gene exchange',GI.sex]];
    $('traits').innerHTML=traitDefs.map(([label,index])=>`<div class="trait-row"><span>${label}</span><div><b style="width:${Math.round(o.genes[index]*100)}%"></b></div><small>${Math.round(o.genes[index]*100)}%</small></div>`).join('');
    const roles=o.body.nodes.some(n=>n.role===0)||o.cells===1?['Metabolic']:[];if(o.body.nodes.some(n=>n.role===1))roles.push('Support tissue');if(o.body.links.some(l=>l.kind===2)||o.body.joints.some(j=>j.enabled&&j.work>0))roles.push('Contractile tissue');if(o.body.nodes.some(n=>n.role===3))roles.push('Sensory');if(o.body.nodes.some(n=>n.role===4))roles.push('Digestive tissue');if(o.body.nodes.some(n=>n.role===5))roles.push('Reproductive tissue');if(o.genes[GI.sex]>.5)roles.push('Gene exchange');if(o.cells>1&&o.genes[GI.predation]>.02)roles.push('Cell ingestion');
    $('roles').innerHTML=roles.map(r=>`<span>${r}</span>`).join('');
    const support=o.body.links.filter(l=>l.kind===1).length,motors=o.body.links.filter(l=>l.kind===2).length;
    $('body-mechanics').innerHTML=o.body.nodes.length||competition?`<span class="eyebrow">LIVE BODY MECHANICS</span><div><span>${o.body.joints.length?'Support links / linear motors':'Support / motor links'}</span><strong>${support} / ${motors}</strong></div><div><span>Ground contacts</span><strong>${o.body.contacts} cells</strong></div><div><span>Foot contact transitions</span><strong>${o.body.steps}</strong></div><div><span>Contraction work</span><strong>${o.body.work.toFixed(2)}</strong></div>${competition?`<div><span>Independent offspring</span><strong>${fmt(o.offspring)}</strong></div><div><span>Parents</span><strong>${o.parent?`#${o.parent}`:'Supplied founder'}${o.mate?` + #${o.mate}`:''}</strong></div>`:''}${!lab?`<div><span>Cells consumed</span><strong>${fmt(o.meals)}</strong></div><div><span>Digestion</span><strong>${o.feedingCooldown>0?`${o.feedingCooldown.toFixed(1)} cycles`:'Ready'}</strong></div>`:''}<small>Ivory: support · coral: contractile · blue: sensory. Contacts and joint flexion come from the solver. Contact transitions alone do not establish walking.</small>`:'';
    $('joint-mechanics').hidden=!o.body.joints.length;
    if(o.body.joints.length){
      const active=o.body.joints.filter(j=>j.enabled),deg=(r:number)=>Math.round(r*180/Math.PI);
      $('joint-mechanics').innerHTML=`<span class="eyebrow">MINERALIZED SEGMENTS / ANGULAR JOINTS</span><p>${auto?'Mineral-bearing adhesive contacts resist bending. Contractile material responds to locally secreted signals A and B; changing the bend costs energy. No skeleton or gait is supplied.':'Paired flexion and extension motors respond to inherited recurrent control, contact, joint angle and sensed resource direction.'}</p><div class="cell-summary"><span>Expressed joints<strong>${active.length} / ${o.body.joints.length}</strong></span><span>Paired motors<strong>${active.filter(j=>j.work>0).length*2}</strong></span><span>Joint work<strong>${o.body.joints.reduce((sum,j)=>sum+j.work,0).toFixed(2)}</strong></span><span>Largest limit error<strong>${deg(Math.max(...active.map(j=>j.limit_error),0))}°</strong></span></div><div class="joint-table"><div><span>${auto?'Joint / angle':'Joint / angle / limits'}</span><span>Flex</span><span>Extend</span></div>${o.body.joints.slice(0,16).map((j,i)=>`<div><span>${i+1} · ${deg(j.angle)}°${auto?'':` / ${deg(j.min)}–${deg(j.max)}°`}${j.enabled?'':' · latent'}</span><span>${Math.round(j.flexion*100)}%</span><span>${Math.round(j.extension*100)}%</span></div>`).join('')}</div>${lab?.jointed?`<p>Alternating support: ${lab.alternations} changes this trial. Joint limits, contacts and travel are measurements; they do not certify a walking gait.</p>`:''}`;
    }
    $('cell-ecology').hidden=!o.regulation||o.cells<2;
    if(o.regulation&&o.cells>1){
      const health=o.body.nodes.reduce((total,n)=>total+n.health,0)/o.cells;
      const reserves=o.body.nodes.map(n=>n.reserve),broken=o.body.links.filter(l=>l.integrity<=.08).length;
      $('cell-ecology').innerHTML=`<span class="eyebrow">CELL ECOLOGY / LOCAL REGULATION</span><p>Food, stress and neighbor signals determine cell functions through inherited rules.</p><div class="tissue-counts">${TISSUES.map((name,role)=>`<span data-tissue="${role}"><b>${o.body.nodes.filter(n=>n.role===role).length}</b>${name}</span>`).join('')}</div><div class="cell-summary"><span>Tissue health<strong>${Math.round(health*100)}%</strong></span><span>Broken links<strong>${broken}</strong></span><span>Repair energy<strong>${o.body.repair_work.toFixed(2)}</strong></span><span>Fate changes<strong>${fmt(o.body.differentiations)}</strong></span></div><p>Stored energy: ${Math.min(...reserves).toFixed(2)}–${Math.max(...reserves).toFixed(2)} per cell. Intact connections transport food. Repair and contraction spend it.</p><div class="cell-table"><div class="cell-table-head"><span>Cell / function</span><span>Health</span><span>Energy</span></div>${o.body.nodes.map((n,i)=>`<div><span>${i+1} · ${TISSUES[n.role]}</span><span>${Math.round(n.health*100)}%</span><span>${n.reserve.toFixed(2)}</span></div>`).join('')}</div>`;
    }

    if(auto&&o.programCells){
      $('specimen-description').textContent='A connected component of living cells. Cells share signals and physical bonds; each has its own inherited genome.';
      $('traits').innerHTML='';$('roles').innerHTML='';$('joint-mechanics').hidden=!o.body.joints.length;$('cell-ecology').hidden=true;
      $('body-mechanics').innerHTML=`<span class="eyebrow">CELL GENOMES / LOCAL EXPRESSION</span><p>These are actual cells and genes, not fixed organism trait sliders. Gene promoters respond to local inputs; cells with copied genomes can express different actions. The gene inspector lists up to 64 cells and the specimen view shows up to 256. The world simulates every cell.</p><div class="ancestry-list">${o.programCells.slice(0,64).map(c=>`<details data-program-cell="${c.id}" ${openPrograms.has(String(c.id))?'open':''}><summary>Cell #${c.id} · genome ${c.genome} · G${c.generation}</summary><small>${c.genes.length} genes · ${c.energy.toFixed(2)} energy · parent ${c.parent||'founder'}</small>${c.genes.map(g=>`<small>${esc(auto.actions[g.action])} · strength ${g.strength.toFixed(2)} · promoter [${g.promoter.map(v=>v.toFixed(1)).join(', ')}]</small>`).join('')}${c.material?`<small>Biomass ${c.material.biomass.toFixed(3)} · fuel ${c.material.fuel.toFixed(3)} · bound mineral ${c.material.mineral.toFixed(4)}</small><small>Material stores: ${c.material.proteins.map((v,i)=>v>0.0001?`${esc(auto.actions[i].replace(' synthesis',''))} ${v.toFixed(4)}`:null).filter(Boolean).join(' · ')}</small>`:''}<small>Expressed: ${c.expression.map((v,i)=>v>0.1?auto.actions[i]:null).filter(Boolean).join(', ')||'none'}</small></details>`).join('')}</div>`;
    }
  }else{
    $('joint-mechanics').hidden=true;
    $('cell-ecology').hidden=true;
    const pending=selectedId!==null&&s.ids.includes(selectedId);
    $('specimen-type').textContent=pending?'INSPECTING':st.population?'NO SELECTION':'AWAITING LIFE';$('specimen-name').textContent=pending?'Reading a lifeform':selectedId?'Lifeform no longer alive':st.population?'Choose a lifeform':auto?'Extinct':'Chemistry alone';$('specimen-description').textContent=pending?'Tracing its inherited traits.':st.population?'Click a cell or explore a living lineage.':auto?'No cells survive. Restart or restore a saved world.':'Try seeding life, or wait for a possible origin.';
    for(const id of ['organism-cells','organism-gen','organism-energy'])$(id).textContent='—';$('traits').innerHTML='';$('roles').innerHTML='';$('body-mechanics').innerHTML='';$('organism-id').textContent='';
  }
  drawChart();
}
worker.onmessage=async(event:MessageEvent<Snapshot|{type:'error';message:string}|{type:'save';json:string}|{type:'loaded'}>)=>{
  const data=event.data;
  if(data.type==='error'){pendingLoad=false;toast(data.message);$('loading').hidden=true;return;}
  if(data.type==='loaded'){pendingLoad=true;return;}
  if(data.type==='save'){
    let local=false;try{await storeWorld(data.json);local=true;$<HTMLButtonElement>('restore').disabled=false;}catch{}
    const blob=new Blob([data.json],{type:'application/json'}),url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download=`primordia-${config.kind}-${config.seed}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
    toast(local?'World saved locally and exported. Restore returns to this moment.':'World exported. Open the JSON file to restore it.');return;
  }
  const changed=!snapshot||snapshot.config.seed!==data.config.seed||snapshot.config.kind!==data.config.kind||snapshot.config.name!==data.config.name;
  snapshot=data;view.update(data);$('loading').hidden=true;
  if(changed){config=data.config;syncControls();}
  if(pendingLoad){pendingLoad=false;config=data.config;syncControls();toast('Saved world restored. Evolution continues from its saved state.');}
  if(data.lab&&selectedId!==1){select(1,true);}
  if(autoselect&&data.ids.length){autoselect=false;select(data.ids[0]);}
  const now=performance.now();if(now-lastUI>250||changed){lastUI=now;updateUI(data);}
  if(now-lastSpecimen>250){lastSpecimen=now;specimen.update(data.selected);}
};
worker.onerror=e=>{toast(`Simulation stopped: ${e.message}`);$('loading').hidden=true;};
document.addEventListener('keydown',e=>{if(e.code==='Space'&&!(e.target instanceof HTMLInputElement)&&!(e.target instanceof HTMLSelectElement)&&!$<HTMLDialogElement>('help-dialog').open){e.preventDefault();$('pause').click();}});
window.addEventListener('resize',drawChart);
send({type:'init',config});
