import { createIcons, Orbit, Dna, Sun, Droplets, Mountain, Thermometer, Zap, Play, Pause, RotateCcw, Plus, Focus, Layers, ArrowUpRight, Download, Upload, X, FlaskConical, Leaf, Settings2, CircleHelp, Volume2, VolumeX, ChevronRight, Activity, Bone, Microscope, Maximize2, Sprout, Globe } from 'lucide';
import { WorldView } from './render/world';
import { SpecimenView } from './render/specimen';
import { PRESETS, GI, GENES, TISSUES, type WorldConfig, type WorldKind, STRIDE, type Snapshot, type Command } from './sim/types';
import './style.css';
import './studio.css';
import './planet-ui.css';
import './brush-ui.css';
import {EvolutionGuide,guideHTML,type GuideAction} from './evolution-guide';
import {initStudio,updateStudio,describeBody,setStudioPanel} from './studio';
import {storeWorld,readWorld} from './storage';
import {Inspection,ecologyHTML,evidenceHTML} from './inspection';
import {resourceLabels,resourceColors,resourceUnit} from './sim/resource-view';
import {Collection,organismBrowserHTML,saveOrganismHTML,collectionHTML} from './collection';
import type {SavedSpecimen,SpecimenMessage} from './sim/types';
import {WorldTools,worldToolsHTML,geneEditorHTML,brushToolbarHTML,type EditorMessage} from './world-tools';
import {progressHTML,updateProgress} from './progress';
import {Ladder,ladderHTML,ladderHUD,milestoneHTML,type LadderAction} from './ladder';
import {Powers,powersHTML} from './powers';
import {CELL_TYPES,CHOSEN_COLOR,cellType,isChosen,isComplex} from './sim/roles';
import {CELL_STRIDE} from './sim/types';

const icons={Orbit,Dna,Sun,Droplets,Mountain,Thermometer,Zap,Play,Pause,RotateCcw,Plus,Focus,Layers,ArrowUpRight,Download,Upload,X,FlaskConical,Leaf,Settings2,CircleHelp,Volume2,VolumeX,ChevronRight,Activity,Bone,Microscope,Maximize2,Sprout,Globe};
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const icon=(name:string)=>`<i data-lucide="${name}"></i>`;
const esc=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
const fmt=(n:number)=>Math.floor(n).toLocaleString();
let config:WorldConfig={...PRESETS.tidal};
let setupConfig:WorldConfig={...config};
let snapshot:Snapshot|null=null,paused=true,speed=1,selectedId:number|null=null,tab='progress',autoselect=true;
let view:WorldView, specimen:SpecimenView,inspection:Inspection;
let collection:Collection,placementSpecimen:SavedSpecimen|null=null,placementPending=false;
let pendingFocusId:number|null=null;
let worldTools:WorldTools,editorSelectionPending=false,creatingWorld=false;
let ladder:Ladder,powers:Powers;
let guide:EvolutionGuide,nurseryBusy=false,pendingPlanetSpecimen:SavedSpecimen|null=null;
let lastUI=0,lastSpecimen=0,pendingLoad=false,hasLocalSave=false,quietSteer:string|null=null;
const worker=new Worker(new URL('./sim/worker.ts',import.meta.url),{type:'module'});
const send=(command:Command)=>worker.postMessage(command);
const syncVisibility=()=>send({type:'visibility',visible:!document.hidden});
document.addEventListener('visibilitychange',syncVisibility);
syncVisibility();

document.querySelector('#app')!.innerHTML=`
  <header class="topbar">
    <span class="brand" aria-label="Primordia"><span class="brand-mark">${icon('orbit')}</span>primordia<span class="brand-beta">ENGINE / 12</span></span>

    <div class="header-actions"><button id="save" class="save-button" aria-label="Save world">${icon('download')}<span>Save world</span></button><details id="studio-menu" class="studio-menu"><summary aria-label="World files and tools">${icon('settings-2')}<span>More</span></summary><div><button id="new-world" class="text-button">${icon('plus')} Create a new planet</button><button id="recover-nursery" class="text-button" disabled>${icon('rotate-ccw')} Recover last held planet</button><button id="load" class="text-button">${icon('upload')} Open world file</button><button id="restore" class="text-button" disabled>${icon('rotate-ccw')} Restore browser save</button><button id="open-labs" class="text-button">${icon('flask-conical')} Comparison labs</button><button id="help" class="text-button">${icon('circle-help')} How life works</button><button id="settings" class="text-button">${icon('settings-2')} Toggle view resolution</button></div></details></div>
  </header>
  <div class="workspace awaiting-world hide-shape hide-observe">
    <nav class="planet-dock" aria-label="Planet tools"><button id="toggle-guide" aria-expanded="false">${icon('sprout')}<span>Life</span></button><button id="toggle-powers" aria-expanded="false">${icon('zap')}<span>Powers</span></button><button id="toggle-shape" aria-expanded="false">${icon('globe')}<span>World</span></button><button id="toggle-observe" aria-expanded="false">${icon('microscope')}<span>Inspect</span></button><button id="toggle-collection" aria-expanded="false">${icon('dna')}<span>Collection</span></button></nav>
    <button id="guide-hint" class="guide-hint" aria-label="Open life's ascent">${ladderHUD}</button>
    ${milestoneHTML}
    ${guideHTML}
    ${powersHTML}
    <div id="nursery-status" class="nursery-status" hidden><span><strong>Evolution nursery</strong><small id="nursery-parent-note">Your planet is held.</small></span><button id="return-nursery">Return to planet ↗</button></div>
    <aside class="world-panel" aria-label="Shape the planet" hidden><div class="drawer-heading"><span class="eyebrow">PLANET TOOLS</span><button data-close-panel class="icon-button" aria-label="Close planet tools">×</button></div>
      ${worldToolsHTML}
      <div id="creation-panel">
      <div class="panel-intro"><span class="eyebrow">CREATE A WORLD</span><h1>Let there be life.</h1><p>Set the conditions. Plant the first cells.<br>Discover what survives.</p></div>
<button id="fast-setup" class="observe-button">Quick start · a small, fast planet</button><p class="setup-note">A compact world where generations pass quickly. The best place to begin.</p>
      <section class="panel-section" id="habitat-setup"><div class="section-label"><span>01</span> HABITAT</div>
        <div class="world-options">
          <button class="world-option active" data-world="tidal" aria-pressed="true"><span class="planet tidal"></span><span><strong>Tidal garden</strong><small>Water-rich · abundant light</small><small class="habitat-effect">A gentler start for replication</small></span>${icon('chevron-right')}</button>
          <button class="world-option" data-world="volcanic" aria-pressed="false"><span class="planet volcanic"></span><span><strong>Ember sea</strong><small>Mineral-rich · dim light · active vents</small><small class="habitat-effect">Less sunlight; life can learn to feed on vent chemistry</small></span>${icon('chevron-right')}</button>
          <button class="world-option" data-world="alien" aria-pressed="false"><span class="planet alien"></span><span><strong>Violet frontier</strong><small>Less solvent · moderate light</small><small class="habitat-effect">Cooler, with less water available</small></span>${icon('chevron-right')}</button>
        </div>
      </section>
      <section class="panel-section run-setup" id="run-setup"><div class="section-label"><span>02</span> SCALE &amp; STARTING LIFE</div>
        <div class="capacity-row"><label for="world-size">World size <small>Planet circumference / habitat width</small></label><select id="world-size"><option value="64">64 units</option><option value="128">128 units</option><option value="256">256 units</option><option value="1024" selected>1,024 · regional planet</option><option value="8192">8,192 · large planet</option><option value="64000">64,000 · vast planet</option></select></div>
        <div class="capacity-row"><label for="initial-cells">Starting cells <small>1–8,192 single-cell founders</small></label><input id="initial-cells" type="number" min="1" max="8192" step="1" value="256"></div>
        <div class="capacity-row"><label for="cell-budget">Living-cell limit <small>100–64,000 cells · must fit your starting population</small></label><input id="cell-budget" type="number" min="100" max="64000" step="1" value="8192"></div>
        <p id="setup-impact" class="setup-impact" aria-live="polite"></p><details class="setup-details"><summary>Seed, placement &amp; resource pattern</summary>
          <div class="seed-row"><label for="setup-seed">World seed</label><input id="setup-seed" type="number" min="0" max="4294967295" value="8042"><button id="random-seed" class="icon-button" aria-label="Randomize world seed" title="Randomize world seed">${icon('rotate-ccw')}</button></div>
          <div class="capacity-row"><label for="world-shape">Geometry</label><select id="world-shape"><option value="sphere">Spherical planet</option><option value="planar">Planar habitat</option></select></div>
          <div class="capacity-row"><label for="resource-layout">Ecology</label><select id="resource-layout"><option value="patchy">Patchy food &amp; shade</option><option value="diffuse">Diffuse food &amp; light</option></select></div><div class="capacity-row"><label for="founder-spread">Placement</label><select id="founder-spread"><option value="local" selected>One local origin</option><option value="global">Across the world</option></select></div>
          <p id="world-scale" class="setup-note"></p>
        </details>
        <div class="capacity-row"><label for="setup-star">Star <small>Light is the only income a world has</small></label><select id="setup-star"><option value="1">Dim · life struggles to pay its way</option><option value="3">Warm · a small, steady population</option><option value="6" selected>Bright · life can flourish</option><option value="8">Blazing · crowded, and slower to run</option></select></div>
        <div class="capacity-row"><label for="setup-pace">Evolution pace</label><select id="setup-pace"><option value="1">Real time · watch closely</option><option value="0" selected>Fast-forward · generations first</option></select></div>
        <details class="setup-details"><summary>Starting environment</summary><div class="capacity-row"><label for="setup-sunlight">Sunlight · %</label><input id="setup-sunlight" type="number" min="0" max="100" value="75"></div><div class="capacity-row"><label for="setup-temperature">Baseline temperature · 0–100 model scale</label><input id="setup-temperature" type="number" min="0" max="100" value="48"></div><div class="capacity-row"><label for="setup-nutrients">Starting fuel · %</label><input id="setup-nutrients" type="number" min="0" max="100" value="70"></div><div class="capacity-row"><label for="setup-minerals">Starting minerals · %</label><input id="setup-minerals" type="number" min="0" max="100" value="45"></div><p class="setup-note">A richer world feeds more cells. What those cells become is up to evolution, and to you.</p></details>
        <div class="creation-launch"><p id="setup-summary"></p><p id="creation-status" class="creation-status" role="status" aria-live="polite" hidden></p><button id="create-physical" class="primary-button">${icon('plus')} Create planet &amp; begin</button></div>
        <button id="open-saved-from-setup" class="observe-button">Open a saved world instead</button>
        <button id="undo-world" class="observe-button" disabled>Restore previous world</button>
        <p class="setup-note">Changes apply when you create a world. Every founder begins as one cell.</p>
      </section>
      </div>
      <details class="panel-section engine-card"><summary>What can evolve · population limit</summary>
        <strong id="engine-name">Autonomous evolution</strong><p id="engine-description">Cell heredity, physical bodies and ecology run together in this world.</p>
        <details class="setup-details" id="engine-capabilities"><summary>What can evolve here?</summary><ul class="engine-capabilities"><li>Division, mutation and attached cell colonies</li><li>Specialized material and tissue functions</li><li>Mineral support, contraction and joints</li><li>Swimming, crawling and steering by sense</li><li>Engulfing prey, venom, spines and armour</li><li>Feeding on vent chemistry without light</li><li>Sensing and adaptive cell signals</li><li>Digestion, fluid transport and gas exchange</li><li>Gametes, chromosome recombination and fusion</li></ul><p class="setup-note">None of it is handed out. Each ability starts as a mutation and is built from real material and energy. Use Powers to decide which inventions get a future.</p></details>
        <details class="setup-details" id="run-options"><summary>View &amp; run settings</summary>
          <div class="capacity-row"><label for="run-cell-budget">Current cell limit</label><input id="run-cell-budget" type="number" min="100" max="64000" step="1" value="8192"></div>
          <button id="apply-budget" class="observe-button">Update current limit</button><p class="setup-note">More room means more cells and more chances to change, at a cost in speed.</p>
        </details>
      </details>
      <div class="panel-foot">Shape conditions. Let life find its way.</div>
    </aside>
    <main class="experiment">
      <div class="experiment-heading"><div><span class="eyebrow" id="experiment-label">YOUR OBSERVATORY</span><h2>A world of possibility<span>.</span></h2></div><span class="experiment-caption">Cell by cell. Generation by generation.</span></div>
      <section id="study-status" class="study-status" hidden><div><span class="eyebrow">EXPERIMENT ACTIVE · MAIN WORLD HELD</span><strong id="study-name"></strong><p id="study-description"></p></div><button id="return-physical" class="observe-button" hidden>Return to main world</button><button id="leave-lab" class="observe-button" hidden>Return to main world</button><button id="return-assessment" class="observe-button" hidden>Return to checkpoint</button></section>
      <div class="stats-bar"><div><span>Organisms</span><strong id="population">—</strong><small id="budget-note">of 4,000 capacity</small></div><div><span>Living cells</span><strong id="cells">—</strong><small id="colony-note">waiting for life</small></div><div><span id="diversity-label">Lineages</span><strong id="diversity">—</strong><small id="diversity-note">distinct genetic branches</small></div><div><span>Generation</span><strong id="generation">—</strong><small>maximum descent</small></div></div>
      <div class="world-stage"><div id="world-canvas" aria-label="Interactive 3D ecosystem. Drag to orbit; scroll to zoom; click a cell to inspect."></div>
        <div class="stage-top"><div><span class="stage-badge"><span class="live-dot"></span><span id="run-state">EVOLVING</span></span><span class="world-name" id="world-name">TIDAL GARDEN</span></div><div class="view-tools"><button id="resource-layer" class="stage-button" title="Toggle resource map" aria-label="Toggle resource map">${icon('layers')}</button><button id="focus-life" class="stage-button" title="View living cells" aria-label="View living cells">${icon('microscope')}</button><button id="reset-camera" class="stage-button" title="World overview" aria-label="World overview">${icon('maximize-2')}</button></div></div>
        <details class="world-layers"><summary>View layers</summary><div class="layer-controls"><label for="field-type">View</label><select id="field-type" aria-label="World display layer"><option value="surface">Surface &amp; water</option>${Object.entries(resourceLabels).map(([key,label])=>`<option value="${key}">${label}</option>`).join('')}</select><div id="world-layer-legend" aria-live="polite"></div></div></details>
        <div id="cell-legend" class="cell-legend" aria-label="Cell types alive now" hidden></div>
        ${brushToolbarHTML}
        <div id="placement-status" class="placement-status" hidden><span id="placement-note"></span><button id="cancel-placement">Cancel</button></div>
        <div id="loading" class="loading setup-empty"><strong>A world is waiting</strong><span>Choose a planet and plant the first cells.</span><button id="begin-setup" class="primary-button">Create a planet</button></div>
        <div class="stage-bottom"><span class="orbit-hint">${icon('orbit')} Drag to orbit <b>·</b> Scroll to explore <b>·</b> Click to inspect</span><span id="perf">RUST + WASM</span></div>
        <div class="world-coordinate"><span id="camera-mode">LOCAL BIOSPHERE</span><br><span id="camera-note">Every cell at its true size</span></div>
      </div>
      <section class="timeline"><div class="timeline-top"><div class="playback"><button id="pause" class="play-button" aria-label="Pause evolution">${icon('pause')}<span>Pause</span></button><div class="speed-controls"><label for="simulation-speed">Speed</label><select id="simulation-speed">${[1,10,50,100,500,1000,10000,0].map(s=>`<option value="${s}">${s===0?'Max':`${s.toLocaleString()}×`}</option>`).join('')}</select><span id="achieved-speed">Waiting for a planet</span><details class="time-settings"><summary>Display priority</summary><select id="run-priority" aria-label="Simulation display priority"><option value="live">Watch closely</option><option value="evolution">Generations first</option></select><p>Generations first redraws the planet less often so evolution runs faster. The world itself is identical.</p></details></div></div><span class="cycle"><strong id="time">0.0</strong> cycles elapsed</span><button id="pulse" class="text-button" title="Manual intervention: add nutrients across the world">${icon('droplets')} Feed world</button></div>
        <details id="population-history" class="setup-details"><summary>Population history</summary><div class="chart-heading"><span>LIFE THROUGH TIME</span><div><span class="chart-legend"><b></b> Organisms</span><span class="chart-legend colony"><b></b> Colonies</span></div></div>
        <canvas id="history-chart" aria-label="Population and colony counts through time"></canvas>
        <div class="chart-axis"><span id="chart-start">0 cycles</span><span>Natural selection has no finish line.</span><span id="chart-end">now</span></div></details>
      </section>
    </main>
    <aside class="life-panel" aria-label="Inspect life and resources" hidden><div class="drawer-heading"><span class="eyebrow">UNDER THE LENS</span><button data-close-panel class="icon-button" aria-label="Close inspector">×</button></div><div class="observer-heading"><span class="eyebrow">OBSERVE</span><span id="observer-context">The story so far</span></div><nav class="panel-tabs" aria-label="Observations"><button data-tab="progress" class="active">Overview</button><button data-tab="life">Inspect life</button><button data-tab="ecology">Resources</button></nav><label class="observer-more" for="observer-view"><span>Go deeper</span><select id="observer-view"><option value="">Choose a view…</option><option value="evidence">Test what a tissue does</option><option value="collection">Organism collection</option><option value="notes">Evolution journal</option></select></label>
      <div id="progress-content">${progressHTML}</div>
      <div id="life-content" hidden>${organismBrowserHTML}<div class="specimen-heading"><span class="eyebrow">UNDER THE LENS</span><button id="focus" class="icon-button" title="Follow this lifeform" aria-label="Follow this lifeform">${icon('focus')}</button></div>
        <div id="specimen-canvas"></div><div class="specimen-info"><span class="specimen-type" id="specimen-type">THE FIRST CELL</span><h3 id="specimen-name">Waiting for life</h3><p id="specimen-description">A small beginning. An open future.</p></div>
        <div id="body-story" class="body-story" hidden></div><div class="specimen-stats"><div><span>Cells</span><strong id="organism-cells">—</strong></div><div><span>Generation</span><strong id="organism-gen">—</strong></div><div><span>Energy</span><strong id="organism-energy">—</strong></div></div>
        ${saveOrganismHTML}
        <details id="technical-details" class="setup-details"><summary>Genes &amp; inner workings</summary><div class="traits-heading">INHERITED FUNCTIONS <span id="organism-id"></span></div><div id="traits"></div>
        <div class="roles" id="roles"></div><div id="body-mechanics" class="body-mechanics"></div><section id="joint-mechanics" class="cell-ecology" hidden></section><section id="cell-ecology" class="cell-ecology" hidden></section>
        <section id="autonomous-results" class="cell-ecology" hidden></section><section id="population-results" class="cell-ecology" hidden><span class="eyebrow">COMPETING ECOSYSTEM / ECOLOGICAL SELECTION</span><button id="toggle-sensing" class="observe-button" data-sensing>Disable directional sensing</button><div id="population-details"></div></section><section id="lab-results" class="lab-results" hidden><span class="eyebrow">CONTROLLED LOCOMOTION ASSAY</span><h3 id="lab-phase">Developing</h3><p id="lab-trial"></p><progress id="lab-progress" max="20" value="0"></progress><div class="lab-metrics"><div><span>Net travel</span><strong id="lab-distance">0.00</strong></div><div><span>Motor work</span><strong id="lab-work">0.00</strong></div><div><span>Champion travel</span><strong id="lab-best">—</strong></div><div><span>Best score</span><strong id="lab-score">—</strong></div></div><p id="lab-improvement"></p><small>Each candidate grows from one cell. Travel is measured after development and settling. Selection rewards net travel and penalizes work and collapse. Units are model units.</small><div id="lab-rounds"></div><button id="replay-best" class="observe-button">Watch best body</button><button id="toggle-motors" class="observe-button" hidden>Disable joint motors</button><button id="continue-selection" class="observe-button" hidden>Continue selection</button></section></details><button id="observe" class="observe-button">${icon('microscope')} Find the largest body ${icon('arrow-up-right')}</button>
        <div class="lineage-heading"><span>LIVING LINEAGES</span><span id="lineage-count">0</span></div><div id="lineages"></div>
        <button id="seed-life" class="seed-button">${icon('plus')} Introduce 80 protocells</button>
      </div>
      <div id="ecology-content" hidden>${ecologyHTML}</div><div id="evidence-content" hidden>${evidenceHTML}</div>
      <div id="collection-content" hidden>${collectionHTML}</div>
      <div id="notes-content" hidden><div class="notes-intro"><span class="eyebrow">AN EVOLUTIONARY JOURNAL</span><h3>Small changes.<br>New possibilities.</h3><p>Milestones record what actually happened in this world.</p></div><div id="events"></div><div class="journal-stats" id="journal-stats"></div></div>
      <div class="science-note">${icon('flask-conical')} A model of possibility.<br><span>Not a prediction of extraterrestrial life.</span></div>
    </aside>
  </div>
  <div id="toast" role="status"></div><input id="file-input" type="file" accept="application/json,.json" hidden>
  <dialog id="creation-dialog" aria-label="Create a new world"><button id="close-creation" class="icon-button" aria-label="Close world creation">${icon('x')}</button><div id="creation-dialog-content"></div></dialog>
  ${geneEditorHTML}
  <dialog id="labs-dialog" aria-labelledby="labs-title"><button id="close-labs" class="icon-button" aria-label="Close labs">${icon('x')}</button><span class="eyebrow">OPTIONAL EXPERIMENTS</span><h2 id="labs-title">Labs &amp; comparisons</h2><p>The main world already includes the current cellular and physical mechanisms. Experiments explore specific conditions; they do not unlock features.</p><p id="labs-active-note" class="setup-note" hidden>Return to the main world before starting another experiment.</p>
    <section class="labs-section"><h3>Current engine experiments</h3><p>A fresh population uses your current world's settings. The main run is held while the experiment runs.</p><div class="labs-actions"><button id="shared-passive" class="observe-button">${icon('activity')} Passive mechanics</button><small>Contraction is disabled to observe physical structure and contact.</small><button id="shared-scarcity" class="observe-button">${icon('leaf')} Scarce resources</button><small>Starts with less fuel. Evolution and all physical mechanisms remain available.</small></div></section>
    <details class="setup-details legacy-labs"><summary>Legacy comparisons</summary><p>Older supplied traits and precursor bodies use earlier models. The locomotion and joint comparisons select movement scores. Their results do not demonstrate autonomous evolution in the main world.</p><div class="labs-actions"><button id="start-lab" class="observe-button">${icon('activity')} Locomotion comparison</button><button id="start-joints" class="observe-button">${icon('bone')} Joint comparison</button><button id="start-tissue" class="observe-button">${icon('microscope')} Tissue comparison</button><div id="competition-entry"><label for="competing-founders">Starting competitors</label><select id="competing-founders"><option value="24">24 cells</option><option value="48" selected>48 cells</option><option value="96">96 cells</option></select><button id="start-population" class="observe-button">${icon('dna')} Competing ecosystem comparison</button></div></div></details>
  </dialog>
  <dialog id="help-dialog"><button id="close-help" class="icon-button" aria-label="Close help">${icon('x')}</button><span class="eyebrow">HOW LIFE WORKS HERE</span><h2>Nothing is handed out.</h2><p>You plant single cells with four genes: feed on light, take up fuel, stick together, divide. Everything after that has to be invented by mutation and built from real material.</p><div class="help-grid"><div><h3>Genes build materials</h3><p>A gene makes one material: muscle fibre, mineral binder, a light receptor. A cell is whatever it has managed to build, and every bit of it costs fuel and energy.</p></div><div><h3>Bodies grow cell by cell</h3><p>Daughter cells that stay attached become a body. Bodies stand, bend and move through real physics: gravity, water, terrain and their own contractions.</p></div><div><h3>You steer, evolution invents</h3><p>Powers never write a gene. They change how often copies vary, how bright the star is, and which bodies survive. Breed for a purpose and the gene pool follows.</p></div><div><h3>Life tells you what it reached</h3><p>Life tracks each step, from colonies to organs, by measuring living cells. A step counts only when cells have actually built the tissue.</p></div></div><p class="help-small">Honest limits: this is a reduced model of life, in its own units, with a designed chemistry. Flagella, horns and image-forming eyes are not modelled. Muscle, skeleton, nerves and organs are, but coordinated walking and intelligence have not yet been seen to evolve here. Extinction is always possible.</p><button id="start-exploring" class="primary-button">Back to your world ${icon('arrow-up-right')}</button></dialog>
`;

$('creation-dialog-content').appendChild($('creation-panel'));
$('progress-content').appendChild($('population-history'));
const statistics=document.createElement('details');statistics.className='setup-details';statistics.innerHTML='<summary>World population</summary>';statistics.appendChild(document.querySelector('.stats-bar')!);$('progress-content').appendChild(statistics);
createIcons({icons});
function toast(message:string){$('toast').textContent=message;$('toast').classList.add('show');setTimeout(()=>$('toast').classList.remove('show'),3500);}
try { view=new WorldView($('world-canvas'),id=>select(id)); specimen=new SpecimenView($('specimen-canvas')); }
catch(error){$('loading').innerHTML=`<strong>3D graphics unavailable</strong><span>This world needs a browser with WebGL 2 enabled.</span>`;throw error;}
inspection=new Inspection(send,(x,z)=>{view.focusLocation(x,z);syncViewNote();},kind=>{
  view.resourceField=kind as typeof view.resourceField;$<HTMLSelectElement>('field-type').value=kind;view.setOverlay(true);syncLayerLegend();
});
collection=new Collection(send,id=>select(id,true),startPlacement,toast,openTab,placeOnPlanet,count=>guide?.collectionChanged(count));
worldTools=new WorldTools(send,startSurfaceTool,cancelPlacement,startPlacement,toast,openTab,layer=>{
  if(layer==='surface')view.setOverlay(false);else{view.resourceField=layer as typeof view.resourceField;$<HTMLSelectElement>('resource-kind').value=layer;view.setOverlay(true);}
  syncLayerLegend();inspection.render();
},brush=>{
  if(brush){placementSpecimen=null;placementPending=false;view.setPlacement(null);$('placement-status').hidden=true;setStudioPanel(null);if(!paused){paused=true;send({type:'control',paused:true});updatePause();}}
  view.setBrush(brush);
});
guide=new EvolutionGuide(handleGuideAction);
document.querySelector('#evolution-guide .guide-intro')!.insertAdjacentHTML('afterend',ladderHTML);
ladder=new Ladder(handleLadderAction);
powers=new Powers(send,()=>selectedId,aimStrike,toast);
function openCreation(){cancelPlacement();if(snapshot)setupConfig={...config};syncSetupControls();if(!creatingWorld)clearCreationFeedback();$<HTMLDialogElement>('creation-dialog').showModal();}
$('new-world').addEventListener('click',openCreation);$('begin-setup').addEventListener('click',openCreation);
$('close-creation').addEventListener('click',()=>$<HTMLDialogElement>('creation-dialog').close());
$('open-saved-from-setup').addEventListener('click',()=>{$<HTMLDialogElement>('creation-dialog').close();$('load').click();});
function startSurfaceTool(note:string,action:(x:number,z:number)=>void){
  if(!worldTools.available()){toast('Return to the main world before editing.');return;}
  cancelPlacement();setStudioPanel(null);send({type:'control',paused:true});$('placement-note').textContent=note;$('placement-status').hidden=false;
  view.setPlacement(action);
}
function openTab(name:string){worldTools?.stopBrush();updateStudio(name);tab=name;document.querySelectorAll<HTMLElement>('[data-tab]').forEach(el=>el.classList.toggle('active',el.dataset.tab===name));for(const key of ['progress','life','ecology','evidence','collection','notes'])$(key+'-content').hidden=tab!==key;inspection.render();collection?.refresh();if(snapshot){updateUI(snapshot);if(tab==='life')specimen.update(snapshot.selected);}}
function cancelPlacement(){worldTools?.stopBrush();placementSpecimen=null;placementPending=false;view.setPlacement(null);$('placement-status').hidden=true;}
function switchNursery(action:'enter'|'leave'|'recover'){
  if(nurseryBusy)return;
  cancelPlacement();nurseryBusy=true;selectedId=null;guide.feedback(action==='enter'?'Holding your planet and opening the compact nursery…':'Restoring your held planet…');
  send({type:'nursery',action});
}
function placeOnPlanet(saved:SavedSpecimen){
  if(nurseryBusy)return;
  if(snapshot?.nursery){pendingPlanetSpecimen=saved;switchNursery('leave');}else startPlacement(saved);
}
function handleLadderAction(action:LadderAction){
  if(action.kind==='powers'){setStudioPanel('powers');return;}
  quietSteer=action.note;send({type:'steer',action:action.action});
}
/** Aim a one-off act at the surface without stopping time. */
function aimStrike(note:string,strike:(x:number,z:number)=>void){
  cancelPlacement();setStudioPanel(null);$('placement-note').textContent=note;$('placement-status').hidden=false;
  view.setPlacement((x,z)=>{cancelPlacement();strike(x,z);});
}
function handleGuideAction(action:GuideAction){
  if(action==='create'){openCreation();return;}
  if(action==='powers'){setStudioPanel('powers');return;}
  if(action==='nursery'){if(!snapshot){openCreation();return;}if(snapshot.nursery){toast('You are already in the evolution nursery.');return;}switchNursery('enter');return;}
  if(action==='return'){switchNursery('leave');return;}
  if(action==='resume'){$('fast-evolution').click();setStudioPanel(null);return;}
  if(action==='inspect'){$('observe').click();return;}
  if(action==='resources'){openTab('ecology');return;}
  if(action==='collection'){openTab('collection');return;}
  if(action==='function'){openTab('evidence');return;}
  if(action==='save'){$('save').click();return;}
  if(action==='founder'){$('introduce-founder').click();return;}
  setStudioPanel('shape');
  if(action==='seasons'){document.querySelector<HTMLButtonElement>('[data-shape="climate"]')!.click();const strength=$<HTMLInputElement>('season-strength');strength.value='0.5';$<HTMLSelectElement>('season-period').value='960';strength.dispatchEvent(new Event('input'));document.querySelector<HTMLDetailsElement>('.season-controls')!.open=true;$('apply-seasons').scrollIntoView({block:'nearest'});return;}
  if(action==='capacity'){$<HTMLDetailsElement>('run-options').open=true;document.querySelector<HTMLDetailsElement>('.engine-card')!.open=true;$('run-options').scrollIntoView({block:'nearest'});return;}
  document.querySelector<HTMLButtonElement>(`[data-shape="${action==='climate'?'climate':'surface'}"]`)!.click();
  if(action==='food')document.querySelector<HTMLButtonElement>('[data-surface="fuel"]')!.click();
}
function startPlacement(s:SavedSpecimen){
  if(!collection.canPlace()){toast('Place copies in the main cellular world, outside an active comparison.');return;}
  cancelPlacement();setStudioPanel(null);placementSpecimen=s;placementPending=false;send({type:'control',paused:true});
  $('placement-note').textContent=`Place ${s.summary.name} · click a clear spot on the surface · drag and zoom normally`;$('placement-status').hidden=false;
  view.setPlacement((x,z)=>{if(!placementSpecimen||placementPending)return;placementPending=true;send({type:'specimen',action:'place',json:placementSpecimen.json,x,z});});
}
$('cancel-placement').addEventListener('click',cancelPlacement);
document.addEventListener('keydown',e=>{if(e.key==='Escape')cancelPlacement();});
for(const id of ['create-physical','undo-world','restore','load','return-physical','leave-lab','start-lab','start-joints','start-tissue','start-population','shared-passive','shared-scarcity'])$(id).addEventListener('click',()=>{cancelPlacement();pendingFocusId=null;});
$('return-assessment').addEventListener('click',()=>send({type:'assessment',action:'restore'}));
let experiment=1;
function syncSetupControls(){
  $<HTMLSelectElement>('world-size').value=String(setupConfig.worldSize??64);$<HTMLSelectElement>('world-shape').value=setupConfig.worldShape??'sphere';$<HTMLSelectElement>('founder-spread').value=setupConfig.founderSpread??'local';$<HTMLSelectElement>('resource-layout').value=setupConfig.resourceLayout??'diffuse';
  for(const [id,value] of [['setup-sunlight',setupConfig.sunlight],['setup-temperature',setupConfig.temperature],['setup-nutrients',setupConfig.nutrients],['setup-minerals',setupConfig.minerals]] as const)$<HTMLInputElement>(id).value=String(Math.round(value*100));
  $<HTMLInputElement>('initial-cells').value=String(setupConfig.initialCells??256);$<HTMLInputElement>('cell-budget').value=String(setupConfig.capacity);$<HTMLInputElement>('setup-seed').value=String(setupConfig.seed);
  document.querySelectorAll<HTMLElement>('[data-world]').forEach(el=>{const active=el.dataset.world===setupConfig.kind;el.classList.toggle('active',active);el.setAttribute('aria-pressed',String(active));});
  scaleDescription();
}
function syncControls(){
  if(config.kind!=='lab'&&!snapshot?.lab&&!snapshot?.study&&!snapshot?.competition&&(config.physicalAssay===undefined||config.physicalAssay==='full')){setupConfig={...config};syncSetupControls();}
  document.querySelector('.experiment-heading h2')!.textContent=config.name.replace(/^The /,'');$('world-name').textContent=planetName(config).toUpperCase();$('experiment-label').textContent=`EXPERIMENT ${String(experiment).padStart(3,'0')} / SEED ${config.seed}`;
}
/** The planet the player chose; the engine relabels running worlds generically. */
function planetName(c:WorldConfig){return c.kind==='lab'?c.name.replace(/^The /,''):PRESETS[c.kind].name.replace(/^The /,'');}
function clearCreationFeedback(){
  $('creation-status').hidden=true;$('creation-status').textContent='';
  $('creation-panel').querySelectorAll('[aria-invalid="true"]').forEach(el=>{el.removeAttribute('aria-invalid');el.removeAttribute('aria-describedby');});
}
function creationFeedback(message:string,error=false,field?:string){
  const status=$('creation-status');status.classList.toggle('error',error);status.setAttribute('role',error?'alert':'status');status.setAttribute('aria-live',error?'assertive':'polite');status.textContent=message;status.hidden=false;
  if(field){const input=$(field);input.setAttribute('aria-invalid','true');input.setAttribute('aria-describedby','creation-status');const details=input.closest('details');if(details)details.open=true;input.focus({preventScroll:true});}
  if($<HTMLDialogElement>('creation-dialog').open)status.scrollIntoView({block:'nearest'});
}
function setCreatingWorld(value:boolean){
  creatingWorld=value;const button=$<HTMLButtonElement>('create-physical');button.disabled=value;button.textContent=value?'Creating planet…':'Create planet & begin';$('creation-panel').setAttribute('aria-busy',String(value));
}
function validSetup(c:WorldConfig){
  const reject=(message:string,field:string)=>{if($<HTMLDialogElement>('creation-dialog').open){clearCreationFeedback();creationFeedback(message,true,field);}else toast(message);return false;};
  if(![64,128,256,1024,8192,64000].includes(c.worldSize??64))return reject('Choose a supported world size.','world-size');
  if((c.worldSize??64)>256&&c.worldShape!=='sphere')return reject('Worlds larger than 256 units need spherical geometry.','world-shape');
  if(!Number.isInteger(c.initialCells)||c.initialCells!<1||c.initialCells!>8192)return reject('Starting cells must be a whole number from 1 to 8,192.','initial-cells');
  if(!Number.isInteger(c.capacity)||c.capacity<100||c.capacity>64000)return reject('Living-cell limit must be a whole number from 100 to 64,000. Update this value, then create your planet.','cell-budget');
  if(c.capacity<c.initialCells!)return reject('Living-cell limit must be at least the number of starting cells.','cell-budget');
  for(const [value,field,label] of [[c.sunlight,'setup-sunlight','Sunlight'],[c.temperature,'setup-temperature','Temperature'],[c.nutrients,'setup-nutrients','Starting fuel'],[c.minerals,'setup-minerals','Starting minerals']] as const){if(!Number.isFinite(value)||value<0||value>1)return reject(`${label} must be between 0 and 100.`,field);}
  if(!Number.isInteger(c.seed)||c.seed<0||c.seed>4294967295)return reject('World seed must be a whole number from 0 to 4,294,967,295.','setup-seed');
  return true;
}
function physicalSetup():WorldConfig {
  return {...setupConfig,sunlight:$<HTMLInputElement>('setup-sunlight').valueAsNumber/100,temperature:$<HTMLInputElement>('setup-temperature').valueAsNumber/100,nutrients:$<HTMLInputElement>('setup-nutrients').valueAsNumber/100,minerals:$<HTMLInputElement>('setup-minerals').valueAsNumber/100,resourceLayout:$<HTMLSelectElement>('resource-layout').value as 'diffuse'|'patchy',worldSize:Number($<HTMLSelectElement>('world-size').value),worldShape:$<HTMLSelectElement>('world-shape').value as 'planar'|'sphere',founderSpread:$<HTMLSelectElement>('founder-spread').value as 'local'|'global', initialCells:Number($<HTMLInputElement>('initial-cells').value),capacity:Number($<HTMLInputElement>('cell-budget').value),seed:Number($<HTMLInputElement>('setup-seed').value),origin:'seeded',physicalAssay:'full'};
}
function updateSetupSummary(){
  const size=Number($<HTMLSelectElement>('world-size').value),cells=Number($<HTMLInputElement>('initial-cells').value),limit=Number($<HTMLInputElement>('cell-budget').value),local=$<HTMLSelectElement>('founder-spread').value==='local';
  $('setup-impact').textContent=size>256?'A bigger planet gives life room to spread, not a head start. Keep the founders together so they can meet.':'A compact planet runs fastest. Every living cell costs time, so start modest and raise the limit later.';
  const budgetValid=Number.isInteger(limit)&&limit>=100&&limit<=64000&&limit>=cells;
  $('setup-summary').textContent=`${Number.isFinite(cells)?fmt(cells):'—'} single-cell founders · ${fmt(size)}-unit world · ${local?'one local origin':'spread across the world'}. ${budgetValid?`Population can grow to ${fmt(limit)} cells.`:'Choose a living-cell limit of 100–64,000 that fits the starting cells.'}`;
}
function scaleDescription(){updateSetupSummary();const size=Number($<HTMLSelectElement>('world-size').value),sphere=$<HTMLSelectElement>('world-shape').value==='sphere',grid=Math.min(160,Math.round(size/1.6)),spacing=size/grid;$('world-scale').textContent=sphere?`Circumference ${fmt(size)}; radius ${(size/(2*Math.PI)).toFixed(1)} model units. Cells begin about 0.44 units across. ${size>256?`Coarse planetary fields at ${spacing.toFixed(1)} nominal spacing; individual 3D cells and contacts retain their scale. Local chemical gradients inside a coarse site are unresolved. Use a local origin to keep founders close.`:'Radial gravity and connected surface flow, with 1.6-unit equatorial field spacing.'}`:`${size} × ${size} model units; ${grid} × ${grid} fluid sites. Large spans require spherical geometry.`;}

for(const id of ['world-size','world-shape','initial-cells','cell-budget','founder-spread','resource-layout','setup-pace'])$(id).addEventListener('change',scaleDescription);
for(const id of ['initial-cells','cell-budget'])$(id).addEventListener('input',updateSetupSummary);
function createPhysical(){if(creatingWorld)return;const next=physicalSetup();clearCreationFeedback();if(!validSetup(next))return;setCreatingWorld(true);creationFeedback(`Creating your planet with ${fmt(next.initialCells!)} starting cells. Loading the engine and preparing the world…`);setupConfig={...next};speed=Number($<HTMLSelectElement>('setup-pace').value);$<HTMLSelectElement>('simulation-speed').value=String(speed);$<HTMLSelectElement>('run-priority').value=speed===0?'evolution':'live';send({type:'display-priority',mode:speed===0?'evolution':'live'});selectedId=null;autoselect=true;paused=false;view.select(null);specimen.update(null);$('loading').hidden=false;$('loading').classList.add('preparing-world');const star=Number($<HTMLSelectElement>('setup-star').value);send({type:'init',config:next,steering:[...(star===1?[]:[{kind:'light' as const,scale:star}]),...(next.kind==='volcanic'?[{kind:'vents' as const,scale:4}]:[])]});send({type:'control',speed});updatePause();}
$('create-physical').addEventListener('click',createPhysical);
$('creation-panel').addEventListener('input',()=>{if(!creatingWorld)clearCreationFeedback();});
$('fast-setup').addEventListener('click',()=>{setupConfig={...PRESETS.tidal,worldSize:128,initialCells:64,capacity:4096};syncSetupControls();$<HTMLSelectElement>('setup-pace').value='0';clearCreationFeedback();creationFeedback('Quick start ready: a small planet, 64 starting cells, room for 4,096. Create your planet to begin.');});
$('fast-evolution').addEventListener('click',()=>{if(!snapshot)return;if(worldTools.painting){toast('Finish the brush stroke first.');return;}worldTools.stopBrush();speed=0;paused=false;$<HTMLSelectElement>('simulation-speed').value='0';$<HTMLSelectElement>('run-priority').value='evolution';send({type:'display-priority',mode:'evolution'});send({type:'control',speed,paused:false});updatePause();openTab('progress');toast('Fast-forwarding. The planet redraws less often so generations pass sooner.');});
$('run-priority').addEventListener('change',()=>send({type:'display-priority',mode:$<HTMLSelectElement>('run-priority').value as 'live'|'evolution'}));
$('population-history').addEventListener('toggle',drawChart);
$('technical-details').addEventListener('toggle',()=>{if(snapshot&&tab==='life')updateUI(snapshot);});
$('undo-world').addEventListener('click',()=>{paused=true;selectedId=null;autoselect=true;view.select(null);send({type:'undo-world'});updatePause();});
$('apply-budget').addEventListener('click',()=>{const capacity=Number($<HTMLInputElement>('run-cell-budget').value);if(!Number.isInteger(capacity)||capacity<100||capacity>64000){toast('Choose a whole-number limit from 100 to 64,000.');return;}send({type:'cell-budget',capacity});});
$('field-type').addEventListener('change',()=>{const kind=$<HTMLSelectElement>('field-type').value;if(kind==='surface')view.setOverlay(false);else{view.resourceField=kind as typeof view.resourceField;$<HTMLSelectElement>('resource-kind').value=kind;inspection.render();view.setOverlay(true);}syncLayerLegend();});
for(const [id,assay] of [['shared-passive','passive'],['shared-scarcity','scarcity']] as const){$(id).addEventListener('click',()=>{
  const next={...config,origin:'seeded' as const,physicalAssay:assay};if(!validSetup(next))return;if(assay==='scarcity')next.nutrients=.08;
  selectedId=null;autoselect=true;view.select(null);send({type:'physical-study',config:next});toast('Physical world held. The experiment uses the same cells, chemistry, joints and inheritance engine.');
});}
$('return-physical').addEventListener('click',()=>{selectedId=null;autoselect=true;view.select(null);send({type:'return-physical'});});
function updatePause(){ $('pause').innerHTML=icon(paused?'play':'pause')+`<span>${paused?'Resume':'Pause'}</span>`;$('pause').setAttribute('aria-label',paused?'Resume evolution':'Pause evolution');$('run-state').textContent=paused?'PAUSED':'EVOLVING';document.querySelector('.stage-badge')!.classList.toggle('paused',paused);createIcons({icons}); }
function syncViewNote(){
  const overview=view.cameraMode==='overview';
  $('camera-mode').textContent=overview?'WORLD OVERVIEW':'LOCAL BIOSPHERE';
  $('camera-note').textContent=overview&&view.config?.worldShape==='sphere'&&(view.config.worldSize??64)>256?'Dots mark where life is · not its size':'Every cell at its true size';
  $('focus-life').classList.toggle('active',!overview);
  $('reset-camera').classList.toggle('active',overview);
  syncLayerLegend();
}
function syncLayerLegend(){
  $('resource-layer').classList.toggle('active',view.overlay);
  const lab=!!snapshot?.lab;$<HTMLSelectElement>('field-type').disabled=lab;
  $<HTMLSelectElement>('field-type').value=view.overlay?view.resourceField:'surface';
  if(lab){$('world-layer-legend').textContent='Laboratory floor';return;}
  if(!view.overlay){$('world-layer-legend').textContent='Blue = surface water · dissolved resources: choose a map';return;}
  const kind=view.resourceField,r=view.resourceRange,high=resourceColors[kind].join(','),num=(v:number)=>v>0&&v<.001?v.toExponential(2):v.toFixed(3);
  $('world-layer-legend').innerHTML=r.available?`<span class="layer-scale" style="background:linear-gradient(90deg,rgb(24,31,36),rgb(${high}))"></span><span>${resourceLabels[kind]} · ${num(r.min)}–${num(r.max)} ${resourceUnit(kind)}<small>Brighter = more · logarithmic scale · surface colors replaced</small></span>`:'This field is unavailable in the current model.';
}
function select(id:number|null,zoom=false){if(snapshot?.assessment?.running&&id!==selectedId){toast('This trial tracks its captured body. Return to the checkpoint before selecting another.');return;}selectedId=id;send({type:'select',id});view.select(id,zoom);syncViewNote();if(id!==null)openTab('life');}
document.querySelectorAll<HTMLElement>('[data-world]').forEach(el=>el.addEventListener('click',()=>{
  const current=physicalSetup(),kind=el.dataset.world as Exclude<WorldKind,'lab'>;
  setupConfig={...PRESETS[kind],worldSize:current.worldSize,worldShape:current.worldShape,founderSpread:current.founderSpread,initialCells:current.initialCells,capacity:current.capacity,seed:current.seed,resourceLayout:current.resourceLayout};
  syncSetupControls();
}));
$('random-seed').addEventListener('click',()=>{$<HTMLInputElement>('setup-seed').value=String(crypto.getRandomValues(new Uint32Array(1))[0]);});
$('pause').addEventListener('click',()=>{if(worldTools.painting){toast('Finish the brush stroke first.');return;}worldTools.stopBrush();paused=!paused;send({type:'control',paused});updatePause();});
$('simulation-speed').addEventListener('change',()=>{speed=Number($<HTMLSelectElement>('simulation-speed').value);send({type:'control',speed});});
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
$('resource-layer').addEventListener('click',()=>{view.setOverlay(!view.overlay);document.querySelector<HTMLDetailsElement>('.world-layers')!.open=view.overlay;$<HTMLSelectElement>('resource-kind').value=view.resourceField;inspection.render();syncLayerLegend();});
$('reset-camera').addEventListener('click',()=>{view.resetCamera();syncViewNote();});
$('focus-life').addEventListener('click',()=>{if(!view.focusLife(selectedId,true))toast('No living cells to focus on.');syncViewNote();});
$('settings').addEventListener('click',()=>{view.setQuality(view.quality==='high'?'low':'high');toast(`Render resolution: ${view.quality==='high'?'high':'1× pixels · default'}. Effects remain off.`);});
$('focus').addEventListener('click',()=>{if(selectedId)select(selectedId,true);else toast('Select a lifeform first.');});
$('observe').addEventListener('click',()=>{
  if(!snapshot?.ids.length){toast('This world has no living organisms yet.');return;}
  let best=0;for(let i=1;i<snapshot.ids.length;i++){if(snapshot.data[i*STRIDE+3]>snapshot.data[best*STRIDE+3])best=i;}
  select(snapshot.ids[best],true);
});
document.querySelectorAll<HTMLElement>('[data-tab]').forEach(el=>el.addEventListener('click',()=>openTab(el.dataset.tab!)));
$('lineages').addEventListener('click',e=>{const button=(e.target as HTMLElement).closest<HTMLElement>('[data-lineage]');if(!button||!snapshot)return;const l=Number(button.dataset.lineage),line=snapshot.lineages.find(x=>x.id===l);if(!line)return;
  // The rendering buffer contains no lineage IDs; ask the core by the representative ID supplied in metadata.
  const representative=(line as typeof line & {representative?:number}).representative;if(representative)select(representative,true);
});
initStudio(openTab);
for(const id of ['toggle-guide','guide-hint','toggle-powers','toggle-shape','toggle-observe','toggle-collection'])$(id).addEventListener('click',()=>worldTools.stopBrush());
$('return-nursery').onclick=()=>switchNursery('leave');
$('recover-nursery').onclick=()=>switchNursery('recover');
readWorld('nursery-parent').then(json=>$<HTMLButtonElement>('recover-nursery').disabled=!json).catch(()=>{});
$('open-labs').addEventListener('click',()=>$<HTMLDialogElement>('labs-dialog').showModal());
$('close-labs').addEventListener('click',()=>$<HTMLDialogElement>('labs-dialog').close());
for(const id of ['shared-passive','shared-scarcity','start-lab','start-joints','start-tissue','start-population'])$(id).addEventListener('click',()=>$<HTMLDialogElement>('labs-dialog').close());
$('help').addEventListener('click',()=>$<HTMLDialogElement>('help-dialog').showModal());
for(const id of ['close-help','start-exploring'])$(id).addEventListener('click',()=>$<HTMLDialogElement>('help-dialog').close());
$('save').addEventListener('click',()=>send({type:'save'}));
$('restore').addEventListener('click',async()=>{try{const json=await readWorld();if(!json){toast('There is no local save yet.');return;}send({type:'load',data:JSON.parse(json),restore:true});selectedId=null;view.select(null);autoselect=true;}catch{toast('Could not read the local save. You can open an exported JSON file.');}});
readWorld().then(json=>{hasLocalSave=!!json;$<HTMLButtonElement>('restore').disabled=!json||!!snapshot?.nursery;}).catch(()=>{});
$('load').addEventListener('click',()=>$('file-input').click());
$('file-input').addEventListener('change',async()=>{
  const file=$<HTMLInputElement>('file-input').files?.[0];if(!file)return;
  try{if(file.size>256*1024*1024)throw new Error('Save file is too large (256 MB limit).');const data=JSON.parse(await file.text());send({type:'load',data});selectedId=null;view.select(null);autoselect=true;}catch(error){toast(String(error));}
  $<HTMLInputElement>('file-input').value='';
});
function drawChart(){
  if(!snapshot||!$<HTMLDetailsElement>('population-history').open||document.querySelector<HTMLElement>('.life-panel')!.hidden)return;const canvas=$<HTMLCanvasElement>('history-chart'),rect=canvas.getBoundingClientRect(),dpr=Math.min(devicePixelRatio,2);canvas.width=rect.width*dpr;canvas.height=rect.height*dpr;
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
function systemsPanel(a:import('./sim/types').AutonomousView){
  const s=a.systems;if(!s)return '';
  return `<span class="eyebrow">DEVELOPING BODY SYSTEMS</span><p>Cells can form sealed fluid compartments, pump and filter material, exchange gases, deposit matrix and fibers, remodel support, and maintain reproductive cells. Functions depend on produced materials, local conditions and energy.</p><div class="cell-summary"><span>Fluid compartments / connections<strong>${fmt(s.fluidCells)} / ${fmt(s.fluidConnections)}</strong></span><span>Internal water / dissolved material<strong>${s.internalWater.toFixed(4)} / ${s.internalSolutes.toFixed(4)}</strong></span><span>Water / solute transported<strong>${s.ledger.waterFlow.toFixed(4)} / ${s.ledger.soluteFlow.toFixed(4)}</strong></span><span>Food absorbed / waste filtered<strong>${s.ledger.absorbed.toFixed(4)} / ${s.ledger.filtered.toFixed(4)}</strong></span><span>Gas exchange / respiration<strong>${s.ledger.gasExchange.toFixed(4)} / ${s.ledger.respired.toFixed(4)}</strong></span><span>Matrix / fiber material<strong>${s.matrix.toFixed(4)} / ${s.fibers.toFixed(4)}</strong></span><span>Support remodeled<strong>${s.ledger.mineralRemodeled.toFixed(4)}</strong></span><span>Material particles<strong>${fmt(s.particles.length)}</strong></span><span>Sequence variants / material<strong>${fmt(s.polymerSpecies)} / ${s.polymerMass.toFixed(4)}</strong></span><span>Replication-repair cells<strong>${fmt(s.germlineCells)}</strong></span><span>Pumping / replication-repair energy<strong>${s.ledger.pumpWork.toFixed(4)} / ${s.ledger.fidelityWork.toFixed(4)}</strong></span><span>Matrix consumed by other cells<strong>${s.ledger.matrixMeals.toFixed(4)}</strong></span><span>Gas balance error<strong>${s.gasError.toExponential(1)}</strong></span></div>`;
}
function developmentPanel(a:import('./sim/types').AutonomousView){
  const d=a.development;if(!d)return '';
  return `<span class="eyebrow">INHERITED DEVELOPMENT / CONTACT NETWORKS</span><p>Cells can synthesize light receptors, adaptive signaling material, polar scaffolds, gametic catalysts and membrane fusogens. Each pathway has to be invented by mutation and paid for in material and energy.</p><div class="cell-summary"><span>Adaptive signal contacts<strong>${fmt(d.synapses)}</strong></span><span>Signal transmissions<strong>${fmt(d.transmissions)}</strong></span><span>Local weight changes<strong>${fmt(d.adaptations)}</strong></span><span>Sensing / network energy<strong>${d.sensoryWork.toFixed(4)} / ${d.neuralWork.toFixed(4)}</strong></span><span>Gametes produced / living<strong>${fmt(d.gametesProduced)} / ${fmt(d.livingGametes)}</strong></span><span>Fertilized zygotes<strong>${fmt(d.zygotes)}</strong></span><span>Chromosome reductions<strong>${fmt(d.meioses)}</strong></span><span>Living diploid cells<strong>${fmt(d.diploidCells)}</strong></span></div><p>Contact signals can excite or inhibit other cells and change through paid local activity. Compatible gametes from different reproductive origins may fuse on contact. A zygote inherits two chromosomes, then grows through material-splitting division.</p><span class="eyebrow">TWO-PARENT ANCESTRY</span><div class="ancestry-list">${d.recent.map(b=>`<div>G${b.generation} · zygote #${b.child} ← #${b.parent} + #${b.mate}<small>${b.time.toFixed(1)} cycles</small></div>`).join('')||'No fertilization observed in this run.'}</div>`;
}
function physiologyPanel(a:import('./sim/types').AutonomousView){
  const b=a.physiology,p=a.planet;
  return `${p?`<span class="eyebrow">A CHANGING PLANET</span><p>${esc(p.weather)} · day ${p.day+1} · seasonal phase ${Math.round(p.season*100)}%. Light, moisture and temperature affect reactions. Volcanism changes terrain and releases tracked minerals.</p><div class="cell-summary"><span>Rain / snow fallen<strong>${p.precipitation.toFixed(2)}</strong></span><span>Snowpack / melted<strong>${p.snow.toFixed(2)} / ${p.snowmelt.toFixed(2)}</strong></span><span>Water balance error<strong>${p.waterError.toExponential(1)}</strong></span><span>Eruptions<strong>${p.eruptions}</strong></span><span>Organic detritus<strong>${(a.detritus??0).toFixed(3)}</strong></span><span>Oxidized waste<strong>${(a.waste??0).toFixed(3)}</strong></span></div>`:''}${b?`<span class="eyebrow">CONNECTED FUNCTIONS</span><p>Cells can make support, contractile material, reserves, receptors, excitable channels, transporters and protection. Bodies combine these however their genes direct.</p><div class="cell-summary"><span>Stored lipid-like material<strong>${b.lipid.toFixed(4)}</strong></span><span>Fuel / ATP transported<strong>${b.ledger.fuel_transported.toFixed(3)} / ${b.ledger.energy_transported.toFixed(3)}</strong></span><span>Excitable pulses<strong>${b.ledger.spikes}</strong></span><span>Injured / repaired<strong>${b.injured} / ${b.ledger.repairs.toFixed(4)}</strong></span><span>Excreted / decomposed<strong>${b.ledger.excreted.toFixed(3)} / ${b.ledger.decomposed.toFixed(3)}</strong></span><span>Bodies with ≥3 functions<strong>${b.mixedBodies}</strong></span></div><div class="cell-table"><div><span>Function</span><span>Cells / regions</span></div>${b.regions.map(r=>`<div><span>${esc(r.function)}</span><span>${r.cells} / ${r.regions}</span></div>`).join('')}</div><p>A region is three or more connected cells of the same type. Colours match the cell types shown on the planet.</p>`:''}`;
}
function physicsPanel(p:import('./sim/types').PhysicsView){
  const d=p.domain;const scale=d?`<span class="eyebrow">PHYSICAL SCALE</span><p>${d.shape==='sphere'?`Spherical planet · circumference ${d.width} · radius ${d.radius.toFixed(1)}`:`Planar habitat · ${d.width} × ${d.width}`} model units. ${fmt(d.sites)} surface sites · ${d.spacing.toFixed(1)} nominal spacing. ${d.shape==='sphere'?'Radial gravity; viscous surface flow carries actual water and materials across longitude and polar caps.':'Closed walls; horizontal fluid and material transport.'} Cell dimensions follow material stores.</p>`:'';
  return `${scale}<span class="eyebrow">PHYSICAL HABITAT / CONSERVATION</span><p>Finite molecules diffuse and travel with a viscous fluid. Cells build proteins and bind minerals from actual stores. Sunlight supplies tracked energy; material recycles after death. Environmental cycles evolve autonomously.</p><p>${p.assay==='passive'?'PASSIVE ASSAY · actuation disabled':'Shared engine · full material mechanics'} · ${fmt(p.joints)} material bending joints · ${p.angularWork.toFixed(4)} angular motor work</p><div class="cell-summary"><span>Organic matter<strong>${p.organic.toFixed(2)}</strong></span><span>Organic balance error<strong>${p.massError.toExponential(1)}</strong></span><span>Mineral balance error<strong>${p.mineralError.toExponential(1)}</strong></span><span>Chemical energy balance error<strong>${p.energyError.toExponential(1)}</strong></span><span>Cell protein mass<strong>${p.proteinMass.toFixed(3)}</strong></span><span>Cell-bound minerals<strong>${p.boundMineral.toFixed(3)}</strong></span><span>Contact bindings<strong>${p.boundEvents}</strong></span><span>Bond dissociations<strong>${p.dissociations}</strong></span><span>Maximum fluid speed<strong>${p.fluidSpeed.toFixed(4)}</strong></span><span>Energy supplied by light<strong>${p.photonEnergy.toFixed(2)}</strong></span></div><p>Use the layers button to show nutrient availability and fluid arrows. Arrows magnify measured flow 18×. This uses reduced chemistry and ${d?.shape==='sphere'?'a viscous shallow surface layer on curved terrain':'a horizontal fluid grid'}, in model units.</p>`;
}
/** Which cell types are alive now, in the colours they have on the planet. */
function updateLegend(s:Snapshot){
  const host=$('cell-legend'),cells=s.cellData;
  if(!s.autonomous?.physics||!cells.length){host.hidden=true;return;}
  const counts=new Array<number>(CELL_TYPES.length).fill(0);let chosen=0,complex=0;
  for(let p=5;p<cells.length;p+=CELL_STRIDE){const raw=cells[p],type=cellType(raw);if(type<counts.length)counts[type]++;if(isChosen(raw))chosen++;if(isComplex(raw))complex++;}
  const row=(color:string,name:string,count:number)=>`<span><i style="background:${color}"></i>${name}<b>${fmt(count)}</b></span>`;
  host.hidden=false;
  host.innerHTML=`<span class="eyebrow">CELL TYPES ALIVE</span>${CELL_TYPES.map((t,i)=>counts[i]?row(i?t.color:'linear-gradient(135deg,#7fd6a0,#8f9be0)',t.name,counts[i]):'').join('')}${complex?row('radial-gradient(circle,#ffffff 35%,#9fb7a7 40%)','Complex cells',complex):''}${chosen?row(CHOSEN_COLOR,'Chosen lineage',chosen):''}`;
}
function updateUI(s:Snapshot){
  const technicalOpen=$<HTMLDetailsElement>('technical-details').open;
  const openPrograms=new Set(Array.from($('body-mechanics').querySelectorAll<HTMLDetailsElement>('details[open]')).map(el=>el.dataset.programCell));
  const st=s.stats;config=s.config;
  $('experiment-label').textContent=`EXPERIMENT ${String(experiment).padStart(3,'0')} / SEED ${config.seed}`;
  const auto=s.autonomous,lab=s.lab,study=s.study,competition=s.competition;
  const comparison=!!s.assessment?.checkpoint&&!!s.assessment.control;
  const sharedStudy=!comparison&&!!auto?.physics&&s.config.physicalAssay!==undefined&&s.config.physicalAssay!=='full',inStudy=!!lab||study||!!competition||sharedStudy||comparison;
  $('return-assessment').hidden=!comparison;
  $('lab-results').hidden=!lab;$('study-status').hidden=!inStudy;
  $<HTMLButtonElement>('new-world').disabled=inStudy||!!s.nursery;
  $<HTMLButtonElement>('load').disabled=!!s.nursery;$<HTMLButtonElement>('restore').disabled=!!s.nursery||!hasLocalSave;
  $<HTMLButtonElement>('open-labs').disabled=inStudy||!!s.nursery;
  $<HTMLButtonElement>('apply-budget').disabled=inStudy;
  $('return-physical').hidden=!sharedStudy;$('leave-lab').hidden=!(!!lab||study||!!competition);
  $('labs-active-note').hidden=!inStudy;
  $('engine-capabilities').hidden=!auto?.physics||sharedStudy;
  for(const id of ['shared-passive','shared-scarcity','start-lab','start-joints','start-tissue','start-population'])$<HTMLButtonElement>(id).disabled=inStudy||!!s.nursery;
  $('engine-name').textContent=comparison?'Controlled comparison':sharedStudy?'Current engine experiment':inStudy?'Legacy comparison':auto?.physics?'Autonomous evolution':'Earlier saved model';
  $('engine-description').textContent=comparison?'A matched-checkpoint trial with a selected mechanism disabled, or its baseline.':sharedStudy?'The same cellular and physical engine, under experimental conditions.':inStudy?'Supplied traits or precursors from earlier models. Return to your main world for autonomous evolution.':auto?.physics?'Cell heredity, physical bodies and ecology run together in this world.':'This save uses an earlier model. Create a world to use the current physical engine.';
  if(inStudy){$('study-name').textContent=comparison?`Comparison: ${s.assessment!.control}`:sharedStudy?(s.config.physicalAssay==='passive'?'Passive mechanics':'Scarce resources'):competition?'Legacy competing ecosystem':study?'Legacy tissue comparison':lab?.jointed?'Legacy joint comparison':'Legacy locomotion comparison';$('study-description').textContent=comparison?`${s.assessment!.elapsed.toFixed(1)} / ${s.assessment!.duration} cycles. Return restores the original checkpoint.`:sharedStudy?'A fresh seeded population under controlled conditions. Return resumes your held world.':'Earlier supplied bodies or traits. Return resumes your held world.';}
  $<HTMLButtonElement>('undo-world').disabled=!s.canUndoWorld||!!s.nursery;
  const budget=$<HTMLInputElement>('run-cell-budget'),currentLimit=String(st.cellBudget??s.config.capacity);
  if(budget.dataset.limit!==currentLimit){budget.value=currentLimit;budget.dataset.limit=currentLimit;}
  $<HTMLInputElement>('run-cell-budget').disabled=inStudy;$<HTMLButtonElement>('apply-budget').disabled=inStudy;
  $('autonomous-results').hidden=!auto;
  if(auto&&tab==='life'&&technicalOpen){$('autonomous-results').innerHTML=`${systemsPanel(auto)}${developmentPanel(auto)}${physiologyPanel(auto)}${auto.physics?physicsPanel(auto.physics):''}<span class="eyebrow">AUTONOMOUS CELLULAR EVOLUTION</span><p>Each cell owns one or two variable chromosomes. Division copies them with occasional edits, duplication, deletion or new genes. Genes regulate catalyst and material production in physical habitats. Local resources and signals influence expression. Daughters that stay attached form bodies.</p><div class="cell-summary"><span>Cell divisions<strong>${fmt(auto.divisions)}</strong></span><span>Attached daughters<strong>${fmt(auto.attachedDivisions)}</strong></span><span>Living genome variants<strong>${fmt(auto.genomes)}</strong></span><span>Genes per cell<strong>${auto.genomeMin}–${auto.genomeMax}</strong></span><span>Point edits<strong>${fmt(auto.pointEdits)}</strong></span><span>Duplications / deletions<strong>${auto.duplications} / ${auto.deletions}</strong></span><span>New action genes<strong>${fmt(auto.innovations)}</strong></span><span>Contact gene transfers<strong>${fmt(auto.exchanges)}</strong></span><span>${auto.physics?'Energy released by metabolism':'Environmental food energy'}<strong>${auto.foodEnergy.toFixed(1)}</strong></span><span>Paid contraction work<strong>${auto.motorWork.toFixed(2)}</strong></span><span>Largest connected body<strong>${auto.peakBody} cells</strong></span><span>Maximum descent<strong>G${auto.peakGeneration}</strong></span></div><span class="eyebrow">EXPRESSED GENETIC PROGRAMS</span><div class="cell-table">${auto.actions.map((name,i)=>`<div><span>${esc(name)}</span><span>${auto.expressed[i]} cells</span></div>`).join('')}</div><p>The founder supplies four genes: light harvesting, nutrient transport, replication and weak adhesion. Other production pathways require gene changes. Extinction is always possible. Mutation is random; produced replication-repair material can reduce errors at an energy cost. Planetary cycles alter the local environment.</p><span class="eyebrow">RECENT CELL DIVISIONS</span><div class="ancestry-list">${auto.recent.map(r=>`<div>G${r.generation} · #${r.child} ← #${r.parent}<small>${r.attached?'Attached':'Separated'} · genome ${r.genome} · ${r.edits} edits · ${r.time.toFixed(1)} cycles</small></div>`).join('')||'No divisions yet.'}</div>`;}
  $('seed-life').innerHTML=icon('plus')+(competition?' Introduce 12 competing cells':' Introduce 80 protocells');
  $('population-results').hidden=!competition;
  if(competition){
    $('toggle-sensing').textContent=competition.directionalSensing?'Disable directional sensing':'Enable directional sensing';
    $('population-details').innerHTML=`<p>Supplied single-cell families share finite food. Survival and offspring determine which traits spread. Founder labels describe starting biases, not fixed species. Nutrients controls resource capacity and renewal; reduce it to increase scarcity.</p><p>Directional sensing: ${competition.directionalSensing?'on':'off'}. Disabling removes food, prey and mate direction from steering and muscle control. Contact, joint feedback, tissue regulation and sensing costs remain. Save first, then restore before changing the setting for an equal-start comparison. Food energy counts environmental uptake, excluding prey meals; older saves begin tracking on import.</p><div class="cell-summary"><span>Food energy acquired<strong>${competition.foodEnergy.toFixed(1)}</strong></span><span>Offspring born<strong>${fmt(st.births)}</strong></span><span>Gene exchanges<strong>${fmt(st.mating)}</strong></span><span>Cells eaten<strong>${fmt(st.feedings)}</strong></span><span>Food-poor locations<strong>${Math.round(competition.foodPoorFraction*100)}%</strong></span><span>Jointed bodies / peak<strong>${st.jointed} / ${competition.peakJointed}</strong></span><span>Largest grown body<strong>${competition.largestBody} cells</strong></span><span>Maximum descent<strong>G${competition.peakGeneration}</strong></span></div><div class="family-list">${competition.families.map(f=>`<button data-family="${f.id}" ${f.representative?'':'disabled'}><strong>${esc(f.name)}</strong><span>${f.living} living · ${f.births} offspring · ${f.meals} meals · G${f.generation}</span><small>${f.exchanges} exchanges · ${f.deaths} deaths · digestion ${Math.round(f.meanPredation*100)}% · controller change ${(f.controllerChange*100).toFixed(1)}%</small></button>`).join('')}</div><p>Family totals follow the reproducing parent; mating also contributes genes. Controller change is distance from that family's starting weights, not a fitness score. Food-poor locations have less than 0.5 model resource units. Extinction remains possible; introductions are manual.</p><span class="eyebrow">RECENT ANCESTRY</span><div class="ancestry-list">${competition.ancestry.length?competition.ancestry.slice(0,8).map(b=>`<div>G${b.generation} · #${b.child} ← #${b.parent}${b.mate?` + #${b.mate}`:''}<small>${b.time.toFixed(1)} cycles</small></div>`).join(''):'<p>Retained cell division grows bodies. Independent offspring will appear here.</p>'}</div>`;
  }
  for(const id of ['pulse','seed-life']){$<HTMLButtonElement>(id).disabled=!!lab||!!auto;$(id).hidden=!!auto;}
  $<HTMLButtonElement>('resource-layer').disabled=!!lab;
  syncViewNote();
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
  $('achieved-speed').textContent=paused?'Paused':`${(st.actualSpeed??0).toFixed(1)} cycles / second`;
  updateProgress(s);updateLegend(s);
  $('perf').textContent=`View on demand · ${st.computeMs.toFixed(1)} ms / step`;
  $('perf').title=`Draws are skipped while the view is unchanged. Simulation ${st.computeMs.toFixed(1)} ms/step; snapshot preparation ${(st.snapshotMs??0).toFixed(1)} ms; world display preparation ${view.updateMs.toFixed(1)} ms.`;
  if(tab==='notes'){
  $('events').innerHTML=s.events.slice().reverse().map(e=>`<div class="event"><span class="event-symbol">${e.type==='milestone'?'✦':e.type==='colony'?'✧':e.type==='extinction'?'○':e.type==='sex'?'◇':'•'}</span><div><span>${e.time.toFixed(1)} CYCLES</span><p>${esc(e.text)}</p></div></div>`).join('')||'<p class="empty-note">Chemistry is stirring.<br>The first chapter is still unwritten.</p>';
  $('journal-stats').innerHTML=`<div><span>Reproduction events</span><strong>${fmt(st.births)}</strong></div><div><span>${auto?'Cells lost':'Organisms lost'}</span><strong>${fmt(st.deaths)}</strong></div><div><span>${auto?.development?'Fertilizations':'Gene exchanges'}</span><strong>${fmt(st.mating)}</strong></div><div><span>${auto?'Contact digestion events':'Cells consumed'}</span><strong>${fmt(st.feedings)}</strong></div><div><span>${auto?'Injured cells':'Injured colonies'}</span><strong>${fmt(st.injured)}</strong></div><div><span>Living cell budget</span><strong>${fmt(st.cells)} / ${fmt(st.cellBudget)}</strong></div>`;
  }
  $('diversity-label').textContent=auto?'Genomes':'Lineages';$('diversity-note').textContent=auto?'living sequence variants':'distinct genetic branches';
  if(tab!=='life'||document.querySelector<HTMLElement>('.life-panel')!.hidden){drawChart();return;}
  $('lineage-count').textContent=fmt(st.diversity);
document.querySelector('.lineage-heading > span')!.textContent=auto?'BODIES ALIVE NOW':'LIVING LINEAGES';
  $('lineages').innerHTML=s.lineages.slice(0,6).map(l=>`<button class="lineage-row" data-lineage="${l.id}" title="Inspect a member of this lineage"><span class="lineage-dot" style="--lineage-color:${/^#[0-9a-f]{6}$/i.test(l.color)?l.color:'#a9cca0'}"></span><span><strong>${esc(l.name.split(' · ')[0])}</strong><small>${fmt(l.count)} organisms · G${l.generation}</small></span><span class="lineage-percent">${Math.round(l.count/Math.max(1,st.population)*100)}%</span></button>`).join('')||(auto?'<p class="empty-note">No surviving bodies. This run remains extinct.</p>':'<p class="empty-note">No surviving lineages.<br>Life may begin again.</p>');

  const o=s.selected;describeBody(s);

  if(o){
    $('specimen-type').textContent=o.cells===1?'UNICELLULAR':o.cells<4?'SIMPLE COLONY':'MULTICELLULAR';$('specimen-name').textContent=lab?(lab.replaying?'Champion body':`Candidate ${lab.candidate+1}`):o.name.split(' · ')[0];$('organism-id').textContent=`#${o.id}`;
    $('specimen-description').textContent=lab?'A supplied precursor. Its developmental and motor program varies through artificial selection.':o.meals>0?'A colony that supplements its metabolism by digesting smaller lifeforms. Food can fund growth and offspring.':study&&o.lineage===9?'A supplied precursor lineage. Cells grow and specialize under local food and stress; survival and reproduction select descendants.':o.genes[GI.photo]>o.genes[GI.chemo]?'A lineage that draws energy from light.':'A lineage that draws energy from minerals.';
    $('organism-cells').textContent=fmt(o.cells);$('organism-gen').textContent=fmt(o.generation);$('organism-energy').textContent=o.energy.toFixed(1);
    const traitDefs:[string,number][]=[['Light harvesting',GI.photo],['Chemosynthesis',GI.chemo],['Cell adhesion',GI.adhesion],['Contractile potential',GI.muscle],['Structural support',GI.skeleton],['Specialization',GI.specialization],['Cell digestion',GI.predation],['Resource sensing',GI.sensing],['Gene exchange',GI.sex]];
    if(technicalOpen&&!auto)$('traits').innerHTML=traitDefs.map(([label,index])=>`<div class="trait-row"><span>${label}</span><div><b style="width:${Math.round(o.genes[index]*100)}%"></b></div><small>${Math.round(o.genes[index]*100)}%</small></div>`).join('');
    const roles=o.body.nodes.some(n=>n.role===0)||o.cells===1?['Metabolic']:[];if(o.body.nodes.some(n=>n.role===1))roles.push('Support tissue');if(o.body.links.some(l=>l.kind===2)||o.body.joints.some(j=>j.enabled&&j.work>0))roles.push('Contractile tissue');if(o.body.nodes.some(n=>n.role===3))roles.push('Sensory');if(o.body.nodes.some(n=>n.role===4))roles.push('Digestive tissue');if(o.body.nodes.some(n=>n.role===5))roles.push('Reproductive tissue');if(o.genes[GI.sex]>.5)roles.push('Gene exchange');if(o.cells>1&&o.genes[GI.predation]>.02)roles.push('Cell ingestion');
    if(auto?.physics){roles.length=0;for(const role of new Set(o.body.nodes.map(n=>cellType(n.role))))roles.push(CELL_TYPES[role]?.name??CELL_TYPES[0].name);if(o.body.nodes.some(n=>isComplex(n.role)))roles.push('Complex cells');if(o.body.nodes.some(n=>isChosen(n.role)))roles.push('Chosen lineage');}
    $('roles').innerHTML=roles.map(r=>`<span>${r}</span>`).join('');
    const support=o.body.links.filter(l=>l.kind===1).length,motors=o.body.links.filter(l=>l.kind===2).length;
    if(technicalOpen)$('body-mechanics').innerHTML=o.body.nodes.length||competition?`<span class="eyebrow">LIVE BODY MECHANICS</span><div><span>${o.body.joints.length?'Support links / linear motors':'Support / motor links'}</span><strong>${support} / ${motors}</strong></div><div><span>Ground contacts</span><strong>${o.body.contacts} cells</strong></div><div><span>Foot contact transitions</span><strong>${o.body.steps}</strong></div><div><span>Contraction work</span><strong>${o.body.work.toFixed(2)}</strong></div>${competition?`<div><span>Independent offspring</span><strong>${fmt(o.offspring)}</strong></div><div><span>Parents</span><strong>${o.parent?`#${o.parent}`:'Supplied founder'}${o.mate?` + #${o.mate}`:''}</strong></div>`:''}${!lab?`<div><span>Cells consumed</span><strong>${fmt(o.meals)}</strong></div><div><span>Digestion</span><strong>${o.feedingCooldown>0?`${o.feedingCooldown.toFixed(1)} cycles`:'Ready'}</strong></div>`:''}<small>Contacts, strain and joint flexion are measured from the body as it moves.</small>`:'';
    $('joint-mechanics').hidden=!o.body.joints.length;
    if(technicalOpen&&o.body.joints.length){
      const active=o.body.joints.filter(j=>j.enabled),deg=(r:number)=>Math.round(r*180/Math.PI);
      $('joint-mechanics').innerHTML=`<span class="eyebrow">MINERALIZED SEGMENTS / ANGULAR JOINTS</span><p>${auto?'Mineral-bearing adhesive contacts resist bending. Contractile material responds to locally secreted signals and paid excitable pulses; changing the bend costs energy. No skeleton or gait is supplied.':'Paired flexion and extension motors respond to inherited recurrent control, contact, joint angle and sensed resource direction.'}</p><div class="cell-summary"><span>Expressed joints<strong>${active.length} / ${o.body.joints.length}</strong></span><span>Paired motors<strong>${active.filter(j=>j.work>0).length*2}</strong></span><span>Joint work<strong>${o.body.joints.reduce((sum,j)=>sum+j.work,0).toFixed(2)}</strong></span><span>Largest limit error<strong>${deg(Math.max(...active.map(j=>j.limit_error),0))}°</strong></span></div><div class="joint-table"><div><span>${auto?'Joint / angle':'Joint / angle / limits'}</span><span>Flex</span><span>Extend</span></div>${o.body.joints.slice(0,16).map((j,i)=>`<div><span>${i+1} · ${deg(j.angle)}°${auto?'':` / ${deg(j.min)}–${deg(j.max)}°`}${j.enabled?'':' · latent'}</span><span>${Math.round(j.flexion*100)}%</span><span>${Math.round(j.extension*100)}%</span></div>`).join('')}</div>${lab?.jointed?`<p>Alternating support: ${lab.alternations} changes this trial. Joint limits, contacts and travel are measurements; they do not certify a walking gait.</p>`:''}`;
    }
    $('cell-ecology').hidden=!o.regulation||o.cells<2;
    if(technicalOpen&&o.regulation&&o.cells>1){
      const health=o.body.nodes.reduce((total,n)=>total+n.health,0)/o.cells;
      const reserves=o.body.nodes.map(n=>n.reserve),broken=o.body.links.filter(l=>l.integrity<=.08).length;
      $('cell-ecology').innerHTML=`<span class="eyebrow">CELL ECOLOGY / LOCAL REGULATION</span><p>Food, stress and neighbor signals determine cell functions through inherited rules.</p><div class="tissue-counts">${TISSUES.map((name,role)=>`<span data-tissue="${role}"><b>${o.body.nodes.filter(n=>n.role===role).length}</b>${name}</span>`).join('')}</div><div class="cell-summary"><span>Tissue health<strong>${Math.round(health*100)}%</strong></span><span>Broken links<strong>${broken}</strong></span><span>Repair energy<strong>${o.body.repair_work.toFixed(2)}</strong></span><span>Fate changes<strong>${fmt(o.body.differentiations)}</strong></span></div><p>Stored energy: ${Math.min(...reserves).toFixed(2)}–${Math.max(...reserves).toFixed(2)} per cell. Intact connections transport food. Repair and contraction spend it.</p><div class="cell-table"><div class="cell-table-head"><span>Cell / function</span><span>Health</span><span>Energy</span></div>${o.body.nodes.map((n,i)=>`<div><span>${i+1} · ${TISSUES[n.role]}</span><span>${Math.round(n.health*100)}%</span><span>${n.reserve.toFixed(2)}</span></div>`).join('')}</div>`;
    }

    if(auto?.physics){$('specimen-description').textContent='One living body. Its colours show what its cells have become, and its genes had to build every one of those tissues.';$('traits').innerHTML='';}
    if(auto&&o.programCells&&technicalOpen){
      $('specimen-description').textContent='Every cell here carries its own genome. Open a cell below to read the genes it inherited and the materials it has built.';
      $('traits').innerHTML='';$('joint-mechanics').hidden=!o.body.joints.length;$('cell-ecology').hidden=true;
      $('body-mechanics').innerHTML=`<span class="eyebrow">CELL GENOMES / LOCAL EXPRESSION</span><p>These are actual cells and genes, not fixed organism trait sliders. Gene promoters respond to local inputs; cells with copied genomes can express different actions. The gene inspector lists up to 64 cells and the specimen view shows up to 256. The world simulates every cell.</p><div class="ancestry-list">${o.programCells.slice(0,64).map(c=>`<details data-program-cell="${c.id}" ${openPrograms.has(String(c.id))?'open':''}><summary>Cell #${c.id} · genome ${c.genome} · G${c.generation}</summary><small>${c.genes.length} genes · ${c.energy.toFixed(2)} energy · parent ${c.parent||'founder'}</small>${c.lifecycle?`<small>${esc(c.lifecycle.phase)} · ${c.lifecycle.homolog?'diploid':'haploid'} · reproductive origin #${c.lifecycle.root||c.id} · developmental depth ${c.lifecycle.depth}${c.lifecycle.mate?` · other parent #${c.lifecycle.mate}`:''}</small>`:''}${c.genes.map(g=>`<small>${esc(auto.actions[g.action])} · strength ${g.strength.toFixed(2)} · promoter [${g.promoter.map(v=>v.toFixed(1)).join(', ')}]</small>`).join('')}${c.lifecycle?.homolog?`<small>Second chromosome ${c.lifecycle.homolog.id}: ${c.lifecycle.homolog.genes.map(g=>`${esc(auto.actions[g.action])} · strength ${g.strength.toFixed(2)} · promoter [${g.promoter.map(v=>v.toFixed(1)).join(', ')}]`).join('; ')}</small>`:''}${c.material?`<small>Biomass ${c.material.biomass.toFixed(3)} · fuel ${c.material.fuel.toFixed(3)} · bound mineral ${c.material.mineral.toFixed(4)} · lipid ${c.material.lipid.toFixed(4)}</small>${c.material.body?`<small>Internal water ${c.material.body.water.toFixed(4)} · fluid fuel ${c.material.body.solutes[0].toFixed(4)} · oxidized waste ${c.material.body.solutes[1].toFixed(4)} · available gas ${c.material.body.gas[0].toFixed(5)} · matrix ${c.material.body.matrix.toFixed(4)} · fibers ${c.material.body.fibers.toFixed(4)}</small><small>Assembled sequences: ${c.material.body.polymers.map(p=>`${p.sequence.map(x=>'ABCD'[x]).join('')} ${p.amount.toFixed(5)}`).join(' · ')||'none'}</small>`:''}${c.state?`<small>Excitation ${c.state.excitation.toFixed(2)} · recovery ${c.state.recovery.toFixed(2)} · injury ${(c.state.injury*100).toFixed(1)}% · strain ${c.state.strain.toFixed(2)} · light ${(c.state.light??0).toFixed(2)} · neural drive ${(c.state.neural_input??0).toFixed(2)} · pressure ${(c.state.pressure??0).toFixed(3)} · replication repair ${(c.state.germline??0).toFixed(2)}</small>`:''}<small>Material stores: ${c.material.proteins.map((v,i)=>v>0.0001?`${esc(auto.actions[i].replace(' synthesis',''))} ${v.toFixed(4)}`:null).filter(Boolean).join(' · ')}</small>`:''}<small>Expressed: ${c.expression.map((v,i)=>v>0.1?auto.actions[i]:null).filter(Boolean).join(', ')||'none'}</small></details>`).join('')}</div>`;
    }
  }else{
    $('joint-mechanics').hidden=true;
    $('cell-ecology').hidden=true;
    const pending=selectedId!==null&&s.ids.includes(selectedId);
    $('specimen-type').textContent=pending?'INSPECTING':st.population?'NO SELECTION':'AWAITING LIFE';$('specimen-name').textContent=pending?'Reading a lifeform':selectedId?'This one has died':st.population?'Choose a lifeform':auto?'Extinct':'Chemistry alone';$('specimen-description').textContent=pending?'Tracing what it inherited.':st.population?'Click any body on the planet to look inside it.':auto?'No cells survive. Restore a save, or plant new life from World.':'Try seeding life, or wait for a possible origin.';
    for(const id of ['organism-cells','organism-gen','organism-energy'])$(id).textContent='—';$('traits').innerHTML='';$('roles').innerHTML='';$('body-mechanics').innerHTML='';$('organism-id').textContent='';
  }
  drawChart();
}
worker.onmessage=async(event:MessageEvent<Snapshot|SpecimenMessage|EditorMessage|{type:'steered';kind:string;message:string}|{type:'specimen-placed';id:number}|{type:'error';message:string}|{type:'save';json:string}|{type:'loaded'}|{type:'world-created'}|{type:'nursery-changed';active:boolean}>)=>{
  const data=event.data;
  if(data.type==='nursery-changed'){nurseryBusy=false;guide.feedback('');selectedId=null;autoselect=true;view.select(null);if(data.active){speed=0;$<HTMLSelectElement>('simulation-speed').value='0';$<HTMLSelectElement>('run-priority').value='evolution';setStudioPanel('guide');$<HTMLButtonElement>('recover-nursery').disabled=false;}else if(!pendingPlanetSpecimen){openTab('collection');toast('Your planet is restored and paused. Place saved organisms, then resume.');}return;}
  if(data.type==='error'){quietSteer=null;if(nurseryBusy){nurseryBusy=false;pendingPlanetSpecimen=null;guide.feedback(data.message);}if(!snapshot){$('loading').hidden=false;document.querySelector('.workspace')!.classList.add('awaiting-world');}pendingLoad=false;placementPending=false;setCreatingWorld(false);worldTools.onError();collection.onError();if($<HTMLDialogElement>('creation-dialog').open)creationFeedback(data.message,true);else toast(data.message);$('loading').classList.remove('preparing-world');$('loading').hidden=!!snapshot;return;}
  if(data.type==='editor-state'||data.type==='editor-specimen'||data.type==='edited'||data.type==='brush-ack'){if(data.type==='edited'||data.type==='brush-ack'){editorSelectionPending=true;view.invalidateTerrain();}worldTools.accept(data);return;}
  if(data.type==='steered'){if(data.kind==='meteor')view.invalidateTerrain();toast(quietSteer??data.message);quietSteer=null;return;}
  if(data.type==='specimen'){await collection.accept(data);return;}
  if(data.type==='specimen-placed'){const name=placementSpecimen?.summary.name??'Organism';cancelPlacement();autoselect=false;pendingFocusId=data.id;select(data.id,false);toast(`${name} placed as body #${data.id}. World paused; place another or resume.`);return;}
  if(data.type==='world-created'){openTab('progress');if(creatingWorld)experiment++;setCreatingWorld(false);clearCreationFeedback();$<HTMLDialogElement>('creation-dialog').close();setStudioPanel(null);autoselect=true;return;}
  if(data.type==='loaded'){pendingLoad=true;$<HTMLDialogElement>('creation-dialog').close();return;}
  if(data.type==='save'){
    let local=false;try{await storeWorld(data.json);local=true;hasLocalSave=true;$<HTMLButtonElement>('restore').disabled=!!snapshot?.nursery;}catch{}
    const blob=new Blob([data.json],{type:'application/json'}),url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download=`primordia-${config.kind}-${config.seed}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
    toast(local?'World saved locally and exported. Restore returns to this moment.':'World exported. Open the JSON file to restore it.');return;
  }
  const changed=!snapshot||snapshot.config.seed!==data.config.seed||snapshot.config.kind!==data.config.kind||snapshot.config.name!==data.config.name||snapshot.config.worldSize!==data.config.worldSize||snapshot.config.worldShape!==data.config.worldShape;
  snapshot=data;view.update(data);$('loading').hidden=true;$('loading').classList.remove('preparing-world');document.querySelector('.workspace')!.classList.remove('awaiting-world');for(const id of ['pause','save','open-labs','simulation-speed','run-priority','focus-life','reset-camera','resource-layer','settings'])($(id) as HTMLButtonElement).disabled=false;
  if(editorSelectionPending){editorSelectionPending=false;selectedId=data.selection||null;view.select(selectedId);}
  worldTools.update(data);
  if(changed){cancelPlacement();config=data.config;syncControls();}
  if(pendingLoad){pendingLoad=false;config=data.config;syncControls();toast('Saved world restored. Evolution continues from its saved state.');}
  if(data.lab&&selectedId!==1){select(1,true);}
  if(autoselect&&data.ids.length){autoselect=false;view.resetCamera();syncViewNote();}
  if(data.paused!==undefined&&paused!==data.paused){paused=data.paused;updatePause();}
  if(data.assessment?.checkpoint&&data.selection!==undefined&&data.selection!==(selectedId??0)){selectedId=data.selection||null;view.select(selectedId);}
  if(pendingFocusId!==null&&data.ids.includes(pendingFocusId)){view.focusLife(pendingFocusId,false);pendingFocusId=null;syncViewNote();}
  inspection.update(data);ladder.update(data);powers.update(data);
  collection.update(data);guide.update(data);$('nursery-status').hidden=!data.nursery;$('nursery-parent-note').textContent=data.nursery?`${data.nursery.parentName} is held · your collection travels with you.`:'';
  if(pendingPlanetSpecimen&&!data.nursery&&!nurseryBusy){const saved=pendingPlanetSpecimen;pendingPlanetSpecimen=null;startPlacement(saved);}
  const now=performance.now();if(now-lastUI>250||changed){lastUI=now;updateUI(data);}
  if(tab==='life'&&!document.querySelector<HTMLElement>('.life-panel')!.hidden&&now-lastSpecimen>500){lastSpecimen=now;specimen.physics=!!data.autonomous?.physics;specimen.update(data.selected);}
};
worker.onerror=e=>{nurseryBusy=false;pendingPlanetSpecimen=null;guide.feedback(`The engine stopped: ${e.message}. Save any available world and reload.`);setCreatingWorld(false);const message=`The simulation engine stopped: ${e.message}. Reload this page to restart the engine.`;if($<HTMLDialogElement>('creation-dialog').open)creationFeedback(message,true);else toast(message);$('loading').classList.remove('preparing-world');$('loading').hidden=!!snapshot;};
document.addEventListener('keydown',e=>{if(e.code==='Space'&&!(e.target instanceof HTMLInputElement)&&!(e.target instanceof HTMLSelectElement)&&!$<HTMLDialogElement>('help-dialog').open&&!$<HTMLDialogElement>('labs-dialog').open&&!$<HTMLDialogElement>('creation-dialog').open&&!$<HTMLDialogElement>('gene-editor').open){e.preventDefault();$('pause').click();}});
window.addEventListener('resize',drawChart);
syncSetupControls();
for(const id of ['pause','save','open-labs','simulation-speed','run-priority','focus-life','reset-camera','resource-layer','settings'])($(id) as HTMLButtonElement).disabled=true;
updatePause();
$<HTMLDialogElement>('creation-dialog').showModal();
