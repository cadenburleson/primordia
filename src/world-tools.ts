import {BrushStroke,type BrushTool,type BrushSettings,type BrushAck} from './brush';
import {planetPowersHTML,eventsHTML} from './powers';
import type {LifeSource} from './creatures';
import {foodWebHTML,planetStatsHTML} from './planet-stats';
import type {Command, Snapshot, ProgramCell, WorldEdit, SavedSpecimen, SpecimenSummary} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const escape=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
export const worldToolsHTML=`
<div id="active-world-tools">
  <div class="panel-intro"><span class="eyebrow">PLANET</span><h1 id="active-world-title">Your planet</h1></div>
  <section id="planet-stats" class="planet-stats" aria-label="The planet now"></section>
  <details class="planet-web"><summary>Food web and the planet’s books</summary><section id="planet-web" aria-label="Who eats whom"></section></details>
  <p id="world-tools-unavailable" class="setup-note" hidden>Return to the main cellular world to use these tools.</p>
  <fieldset id="world-tool-controls" class="world-tool-controls" disabled>
    <nav class="shape-tabs" aria-label="Planet tools"><button data-shape="climate" class="active" aria-pressed="true">Sky &amp; sea</button><button data-shape="surface" aria-pressed="false">Terrain</button><button data-shape="events" aria-pressed="false">Events</button></nav>
    <section class="panel-section world-tool-section" data-shape-panel="climate">
      ${planetPowersHTML}
      <details class="season-controls"><summary>Seasons <span id="season-summary">Natural cycle</span></summary><div class="season-presets"><button data-season="0">Steady</button><button data-season="0.5">Gentle</button><button data-season="1">Natural</button></div><label for="season-strength">Variation <output id="season-strength-value">100%</output></label><input id="season-strength" type="range" min="0" max="2" step="0.1" value="1"><label for="season-period">Year length <small>cycles</small></label><select id="season-period"><option value="120">Short · 120 cycles</option><option value="240" selected>Standard · 240 cycles</option><option value="960">Long · 960 cycles</option></select><p id="season-phase" class="season-phase"></p><p id="season-impact"></p><button id="apply-seasons" class="observe-button">Apply seasons</button></details>
      <details class="setup-details weather-controls"><summary>Weather by hand</summary>
      <div class="capacity-row"><label for="live-sunlight">Sunlight <small>%</small></label><input id="live-sunlight" type="range" min="0" max="100" step="1"><output id="sunlight-value" for="live-sunlight">—</output></div>
      <div class="capacity-row"><label for="live-temperature">Temperature <small>0 cold · 100 hot</small></label><input id="live-temperature" type="range" min="0" max="100" step="1"><output id="temperature-value" for="live-temperature">—</output></div>
      <div class="capacity-row"><label for="live-clouds">Cloud cover <small>%</small></label><input id="live-clouds" type="range" min="0" max="100" step="1"><output id="clouds-value" for="live-clouds">—</output></div>
      <p id="climate-impact" class="tool-impact" aria-live="polite"></p><button id="apply-climate" class="observe-button">Apply weather</button><button id="place-rain" class="observe-button">Place a rain storm</button><p id="live-weather" class="setup-note"></p><p class="setup-note">Applying weather pauses time.</p></details>
    </section>
    <section class="panel-section world-tool-section" data-shape-panel="surface" hidden><div class="tool-heading"><h3>Shape the land and sea.</h3><p>Pick a brush, then drag across the planet. Painting pauses time and resumes when you are done.</p></div><div class="surface-palette"><span>Land</span><div><button data-surface="raise" class="active">Raise</button><button data-surface="lower">Lower</button><button data-surface="flatten">Flatten</button></div><span>Add</span><div><button data-surface="water">Water</button><button data-surface="fuel">Food</button><button data-surface="mineral">Minerals</button><button data-surface="detritus">Remains</button></div></div>
      <div class="capacity-row surface-select" hidden><label for="surface-tool">Tool</label><select id="surface-tool"><option value="raise">Raise land</option><option value="lower">Lower land</option><option value="flatten">Flatten locally</option><option value="water">Add water</option><option value="fuel">Add dissolved fuel</option><option value="mineral">Add minerals</option><option value="detritus">Add detritus</option></select></div>
      <p id="surface-impact" class="tool-impact" aria-live="polite"></p><button id="paint-surface" class="observe-button">Open brush</button>
    </section>
    <section class="panel-section world-tool-section" data-shape-panel="events" hidden>
      ${eventsHTML}
      <button id="undo-edit" class="observe-button" disabled>Undo the last change</button><p class="setup-note">Undo restores the whole world to just before your last brush stroke, release or disaster.</p>
    </section>
  </fieldset>
</div>`;
const brushIcons:Record<string,string>={raise:'M3 18 12 4l9 14H3m9-9v6m-3-3 3-3 3 3',lower:'M3 6 12 20 21 6H3m9 3v6m-3-3 3 3 3-3',flatten:'M3 15h18M6 8h12m-6-4v8m-3-3 3 3 3-3',water:'M12 3S5 11 5 15a7 7 0 0 0 14 0c0-4-7-12-7-12Z',fuel:'m13 2-8 12h6l-1 8 9-13h-7l1-7Z',mineral:'m5 5 7-3 7 3 3 9-10 8L2 14 5 5Zm0 0 7 17 7-17M2 14h20',detritus:'M20 4C6 2 2 10 6 18c8 4 16-2 14-14ZM5 20l10-10'};
const brushNames:Record<string,string>={raise:'Raise',lower:'Lower',flatten:'Flatten',water:'Water',fuel:'Fuel',mineral:'Mineral',detritus:'Detritus'};
export const brushToolbarHTML=`<section id="brush-toolbar" class="brush-toolbar" aria-label="Brush" hidden>
  <div class="brush-topline"><span class="eyebrow" id="brush-title">PAINT THE PLANET</span><span id="brush-status" role="status">Time paused while painting</span><button id="brush-undo" title="Restore the world to before your last stroke">↶ Undo</button><button id="brush-done">Done <kbd>Esc</kbd></button></div>
  <fieldset id="brush-controls"><nav class="brush-swatches" id="brush-swatches" aria-label="Brush type">${Object.entries(brushNames).map(([key,name])=>`<button data-brush-tool="${key}" aria-pressed="false" title="${name}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${brushIcons[key]}"/></svg><span>${name}</span></button>`).join('')}</nav>
  <div class="brush-sliders" id="brush-terrain"><label for="brush-radius">Size <output id="brush-size-value"></output><input id="brush-radius" type="range" min="1.2" max="32" step="any" value="4.8"></label><label for="brush-amount">Strength <output id="brush-amount-value"></output><input id="brush-amount" type="range" min="0.05" max="2" step="0.05" value="0.35"></label><details class="brush-transform"><summary>Shape</summary><div><label for="brush-aspect">Round → narrow <output id="brush-aspect-value">Round</output></label><input id="brush-aspect" type="range" min="0.25" max="1" step="0.05" value="1"><label for="brush-angle">Rotation <output id="brush-angle-value">0°</output></label><input id="brush-angle" type="range" min="0" max="180" step="5" value="0"><button id="brush-reset-shape">Reset shape</button><small id="brush-resolution"></small><small>Strength is added per spaced dab. Overlapping strokes build up. Flatten uses the height where your drag begins.</small></div></details></div>
  <div class="brush-sliders" id="brush-life" hidden><label for="brush-life-size">Brush size <output id="brush-life-size-value"></output><input id="brush-life-size" type="range" min="0.5" max="24" step="0.5" value="3"></label><label for="brush-life-count">How many <output id="brush-life-count-value"></output><input id="brush-life-count" type="range" min="1" max="8" step="1" value="2"></label></div></fieldset>
  <p id="brush-effect" class="brush-effect"></p><div class="brush-hints"><span><i></i> Drag to paint</span><span>Right / Alt-drag to orbit</span><span><kbd>[</kbd> <kbd>]</kbd> resize · wheel zoom</span></div>
</section>`;
export const geneEditorHTML=`<dialog id="gene-editor" aria-labelledby="gene-editor-title"><button id="close-gene-editor" class="icon-button" aria-label="Close gene editor">×</button><span class="eyebrow">MANUAL CHROMOSOME EDIT</span><h2 id="gene-editor-title">Edit a cell's genes</h2><p>This changes the primary chromosome of one cell. Its materials, age, other chromosome and neighboring cells stay intact. Future division can pass the edited genes on; this does not instantly create an organ or movement.</p><label for="edit-cell">Cell to edit</label><select id="edit-cell"></select><p id="gene-cell-note" class="setup-note"></p><div id="gene-rows"></div><div class="gene-add"><select id="new-gene-action" aria-label="New gene function"></select><button id="add-gene" class="observe-button">Add gene</button></div><button id="apply-genes" class="primary-button">Apply chromosome edit</button></dialog>`;
export type EditorMessage =
 | {type:'editor-state';canUndo:boolean}
 | {type:'editor-specimen';json:string;summary:SpecimenSummary}
 | {type:'edited';message:string}
 | BrushAck;
export class WorldTools {
  private snapshot:Snapshot|null=null;
  private climateKey='';private geometryKey='';private body=0;private editCells:ProgramCell[]=[];private genes:ProgramCell['genes']=[];
  private busy=false;private undo=false;private armed=false;private stroke:BrushStroke;private seasonKey='';
  /** The creature the brush is loaded with, when it paints life instead of terrain. */
  private life:LifeSource|null=null;private undoing=false;private stats='';private web='';private statsAt=0;private statsTimer=0;
  constructor(private send:(c:Command)=>void,private tool:(note:string,action:(x:number,z:number)=>void)=>void,private cancel:()=>void,private place:(s:SavedSpecimen)=>void,private toast:(s:string)=>void,private openTab:(s:string)=>void,private showLayer:(s:string)=>void,private brushView:(b:BrushTool|null)=>void){
    this.stroke=new BrushStroke(this.send,()=>this.render(),message=>{$('brush-status').textContent=message;});
    $('apply-climate').onclick=()=>{const values=['live-sunlight','live-temperature','live-clouds'].map(id=>$<HTMLInputElement>(id).valueAsNumber/100);if(values.some(v=>!Number.isFinite(v)||v<0||v>1)){this.toast('Environment values must be between 0 and 100.');return;}this.apply({kind:'climate',sunlight:values[0],temperature:values[1],clouds:values[2]});};
    $('place-rain').onclick=()=>this.tool('Place rain · click the surface · uses existing atmospheric water',(x,z)=>this.apply({kind:'rain',x,z}));
    $('paint-surface').onclick=()=>this.armBrush();
    $('surface-tool').onchange=()=>{this.describeSurface();this.armBrush();};
    document.querySelectorAll<HTMLButtonElement>('[data-surface],[data-brush-tool]').forEach(b=>b.onclick=()=>{if(!this.available())return;$<HTMLSelectElement>('surface-tool').value=b.dataset.surface??b.dataset.brushTool!;this.describeSurface();this.armBrush();});
    for(const id of ['brush-radius','brush-amount','brush-aspect','brush-angle','brush-life-size','brush-life-count'])$(id).addEventListener('input',()=>{this.describeBrush();if(this.armed)this.installBrush();});
    $('brush-done').onclick=()=>this.stopBrush();
    $('brush-reset-shape').onclick=()=>{$<HTMLInputElement>('brush-aspect').value='1';$<HTMLInputElement>('brush-angle').value='0';this.describeBrush();if(this.armed)this.installBrush();};
    document.addEventListener('keydown',e=>{if(!this.armed||this.stroke.busy||e.altKey||e.ctrlKey||e.metaKey||e.target instanceof HTMLInputElement||e.target instanceof HTMLSelectElement||e.target instanceof HTMLTextAreaElement||!['[',']'].includes(e.key))return;e.preventDefault();const input=$<HTMLInputElement>(this.life?'brush-life-size':'brush-radius');input.value=String(Math.max(Number(input.min),Math.min(Number(input.max),input.valueAsNumber*(e.key==='['?.85:1.18))));this.describeBrush();this.installBrush();});
    for(const id of ['season-strength','season-period'])$(id).addEventListener('input',()=>this.describeSeasons());
    document.querySelectorAll<HTMLButtonElement>('[data-season]').forEach(b=>b.onclick=()=>{$<HTMLInputElement>('season-strength').value=b.dataset.season!;this.describeSeasons();});
    $('apply-seasons').onclick=()=>this.apply({kind:'seasons',strength:$<HTMLInputElement>('season-strength').valueAsNumber,period:Number($<HTMLSelectElement>('season-period').value)});
    document.querySelectorAll<HTMLButtonElement>('[data-shape]').forEach(b=>b.onclick=()=>{this.cancel();document.querySelectorAll<HTMLElement>('[data-shape]').forEach(el=>{const active=el===b;el.classList.toggle('active',active);el.setAttribute('aria-pressed',String(active));});document.querySelectorAll<HTMLElement>('[data-shape-panel]').forEach(el=>el.hidden=el.dataset.shapePanel!==b.dataset.shape);});
    for(const id of ['live-sunlight','live-temperature','live-clouds'])$(id).addEventListener('input',()=>this.describeClimate());
    this.describeSurface();this.describeClimate();this.describeBrush();this.describeSeasons();
    $('copy-organism').onclick=()=>{const s=this.snapshot?.selected;if(s&&this.available()){this.busy=true;this.send({type:'editor',action:'copy',id:s.id,name:`Copy of ${s.name.split(' · ')[0]}`.slice(0,70)});this.render();}};
    // Undo from the brush keeps the brush in hand, so a stroke can be tried again.
    $('brush-undo').onclick=$('undo-edit').onclick=()=>{if(!this.available()||!this.undo)return;this.undoing=this.armed;if(!this.undoing)this.cancel();this.busy=true;this.send({type:'editor',action:'undo'});this.render();};
    $('edit-organism').onclick=()=>this.openEditor();
    $('close-gene-editor').onclick=()=>$<HTMLDialogElement>('gene-editor').close();
    $('edit-cell').onchange=()=>this.chooseCell();
    $('add-gene').onclick=()=>{if(!this.readGenes())return;if(this.genes.length>=48){this.toast('A chromosome supports up to 48 genes.');return;}this.genes.push({action:Number($<HTMLSelectElement>('new-gene-action').value),bias:0,strength:.1,promoter:Array(32).fill(0)});this.renderGenes();};
    $('gene-rows').onclick=e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-remove-gene]');if(b&&this.readGenes()){this.genes.splice(Number(b.dataset.removeGene),1);this.renderGenes();}};
    $('apply-genes').onclick=()=>{if(!this.readGenes())return;if(!this.genes.length){this.toast('Keep at least one gene in the chromosome.');return;}this.apply({kind:'genes',body:this.body,cell:Number($<HTMLSelectElement>('edit-cell').value),genes:this.genes});};
  }
  available(){const s=this.snapshot;return !this.busy&&!this.stroke.busy&&!!s?.autonomous?.physics&&(s.config.physicalAssay??'full')==='full'&&!s.assessment?.running&&!s.assessment?.control&&!s.autonomous.observationControl;}
  update(s:Snapshot){
    this.snapshot=s;
    // The planet at a glance, refreshed about once a second while its panel is open.
    // The planet at a glance, refreshed about once a second. A paused world
    // sends one last picture, so a skipped refresh is caught up shortly after.
    clearTimeout(this.statsTimer);
    if(performance.now()-this.statsAt>900)this.showStats();else this.statsTimer=window.setTimeout(()=>this.showStats(),950);
    const clouds=s.autonomous?.planet?.cloudCover??0,key=JSON.stringify([s.config.seed,s.config.sunlight,s.config.temperature,clouds]);
    if(key!==this.climateKey){this.climateKey=key;for(const [id,value] of [['live-sunlight',s.config.sunlight],['live-temperature',s.config.temperature],['live-clouds',clouds]] as const)$<HTMLInputElement>(id).value=String(Math.round(value*100));this.describeClimate();}
    const planet=s.autonomous?.planet,seasonKey=JSON.stringify([s.config.seed,planet?.seasonStrength,planet?.seasonPeriod]);
    if(this.seasonKey!==seasonKey){this.seasonKey=seasonKey;$<HTMLInputElement>('season-strength').value=String(planet?.seasonStrength??1);const period=planet?.seasonPeriod??240,select=$<HTMLSelectElement>('season-period');if(!Array.from(select.options).some(o=>o.value===String(period)))select.add(new Option(`Custom · ${period} cycles`,String(period)));select.value=String(period);this.describeSeasons();}
    const d=s.autonomous?.physics?.domain,g=JSON.stringify([d?.width,d?.shape]);
    if(d&&g!==this.geometryKey){this.geometryKey=g;const input=$<HTMLInputElement>('brush-radius');input.min=String(d.spacing*.75);input.max=String(d.width*.2);input.value=String(d.spacing*3);this.describeBrush();$('brush-resolution').textContent=`Field spacing ${d.spacing.toLocaleString()} units. Brushes edit this grid; minimum narrow radius ${(d.spacing*.75).toLocaleString()}. Zoom locally to inspect height changes.`;}
    const p=s.autonomous?.planet;$('season-phase').textContent=p?`${(p.seasonStrength??1)===0?'Seasonal variation off':`Year ${Math.floor(s.stats.time/(p.seasonPeriod??240))+1} · ${Math.floor(p.season*100)}% through cycle`} · time ${s.paused?'paused':'running'}`:'';$('live-weather').textContent=p?`${p.weather} · current model temperature ${(p.temperature*100).toFixed(0)} · cloud cover ${(clouds*100).toFixed(0)}%`:'Climate unavailable in this model.';
    const selected=s.selected;$('world-tool-selection').textContent=selected?`Selected: ${selected.name.split(' · ')[0]} · ${selected.cells.toLocaleString()} cell${selected.cells===1?'':'s'}`:'Click a body on the planet to copy it.';
    this.render();
  }
  private showStats(){
    if(!this.snapshot)return;this.statsAt=performance.now();
    const html=planetStatsHTML(this.snapshot);if(html!==this.stats){this.stats=html;$('planet-stats').innerHTML=html;}
    const web=foodWebHTML(this.snapshot);if(web!==this.web){this.web=web;$('planet-web').innerHTML=web;}
  }
  private describeClimate(){
    const light=$<HTMLInputElement>('live-sunlight').valueAsNumber,temperature=$<HTMLInputElement>('live-temperature').valueAsNumber,clouds=$<HTMLInputElement>('live-clouds').valueAsNumber;
    $('sunlight-value').textContent=`${light}%`;$('temperature-value').textContent=`${temperature} / 100`;$('clouds-value').textContent=`${clouds}%`;
    $('climate-impact').textContent=`Clouds leave about ${Math.round(light*(1-.8*clouds/100))}% nominal light before local shade. Light can fuel cells with photosynthetic machinery. Temperature influences chemical activity; clouds also cool the model slightly. Apply to change the world.`;
  }
  private describeSurface(){
    const tool=$<HTMLSelectElement>('surface-tool').value;
    const descriptions:Record<string,string>={raise:'Lift real terrain. New slopes alter gravity and surface water flow; bodies on raised land move up with it.',lower:'Lower terrain. Water can flow into the depression; cells still obey contact and gravity.',flatten:'Blend local elevations toward the clicked height. An amount of 1 fully blends the brush center.',water:'Add surface water. It flows over terrain and carries dissolved material. This is new water added by you.',fuel:'Add dissolved fuel. Cells with uptake machinery can use it to sustain chemistry and reproduction.',mineral:'Add dissolved minerals. Genes and energy are still needed to bind minerals into structural material.',detritus:'Add organic remains. Cells with suitable digestion can turn them into usable material.'};
    $('surface-impact').textContent=descriptions[tool];
    const effects:Record<string,string>={raise:'Lift terrain. Raised ground changes slopes and water flow.',lower:'Carve depressions. Water can collect and flow into them.',flatten:'Smooth toward the height where your drag begins.',water:'Add water that flows and carries dissolved resources.',fuel:'Add food for cells with the machinery to absorb it.',mineral:'Supply raw material for cells that can build mineral support.',detritus:'Add organic remains for cells with suitable digestion.'};
    $('brush-effect').textContent=effects[tool];
    document.querySelectorAll<HTMLElement>('[data-surface],[data-brush-tool]').forEach(b=>{const active=(b.dataset.surface??b.dataset.brushTool)===tool;b.classList.toggle('active',active);b.setAttribute('aria-pressed',String(active));});
  }
  private readBrush():BrushSettings{
    if(this.life)return {tool:'life',radius:$<HTMLInputElement>('brush-life-size').valueAsNumber,amount:$<HTMLInputElement>('brush-life-count').valueAsNumber,aspect:1,angle:0,life:this.life.token};
    return {tool:$<HTMLSelectElement>('surface-tool').value as BrushSettings['tool'],radius:$<HTMLInputElement>('brush-radius').valueAsNumber,amount:$<HTMLInputElement>('brush-amount').valueAsNumber,aspect:$<HTMLInputElement>('brush-aspect').valueAsNumber,angle:$<HTMLInputElement>('brush-angle').valueAsNumber};}
  private describeBrush(){
    const aspect=$<HTMLInputElement>('brush-aspect').valueAsNumber,r=$<HTMLInputElement>('brush-radius'),min=(this.snapshot?.autonomous?.physics?.domain.spacing??1.6)*.75/aspect;
    r.min=String(min);if(r.valueAsNumber<min)r.value=String(min);
    $('brush-size-value').textContent=`${(r.valueAsNumber*2).toFixed(1)} units`;
    $('brush-amount-value').textContent=$<HTMLInputElement>('brush-amount').valueAsNumber.toFixed(2);
    $('brush-aspect-value').textContent=aspect===1?'Round':`${Math.round(aspect*100)}% width`;
    $('brush-angle-value').textContent=`${$<HTMLInputElement>('brush-angle').value}°`;
    const size=$<HTMLInputElement>('brush-life-size').valueAsNumber,count=$<HTMLInputElement>('brush-life-count').valueAsNumber;
    $('brush-life-size-value').textContent=`${(size*2).toFixed(0)} units`;$('brush-life-count-value').textContent=count===1?'One at a time':`${count} a dab`;
    if(this.life)$('brush-effect').textContent=`Drag across the map to release ${this.life.name}s. Sea life belongs in water; anything set down on land has to cope with it.`;
  }
  /** Load the brush with a creature: every drag scatters copies of it. */
  armLife(source:LifeSource,sustain:boolean){
    if(!this.available()){this.toast('Return to the main world before releasing creatures.');return false;}
    this.life=source;this.send({type:'life-source',token:source.token,species:source.species,json:source.json,sustain});
    this.armed=true;this.describeBrush();this.showLayer('surface');$('brush-status').textContent='Time paused · drag to paint';this.installBrush();this.render();return true;
  }
  get arming(){return this.armed;}
  private describeSeasons(){const strength=$<HTMLInputElement>('season-strength').valueAsNumber,period=$<HTMLSelectElement>('season-period').value;
    $('season-strength-value').textContent=`${Math.round(strength*100)}%`;$('season-summary').textContent=strength===0?'Steady climate':`${Math.round(strength*100)}% · ${period} cycles`;
    $('season-impact').textContent=`Seasonal temperature swings by ±${(strength*18).toFixed(0)} model points; seasonal light varies ±${(strength*15).toFixed(0)}%. A shorter year changes conditions more often, not the mutation rate.`;
    document.querySelectorAll<HTMLElement>('[data-season]').forEach(b=>{const active=Number(b.dataset.season)===strength;b.classList.toggle('active',active);b.setAttribute('aria-pressed',String(active));});
  }
  private armBrush(){if(!this.available())return;this.life=null;this.armed=true;this.describeBrush();this.describeSurface();const tool=this.readBrush().tool;this.showLayer(['raise','lower','flatten'].includes(tool)?'surface':tool);$('brush-status').textContent='Time paused · drag to paint';this.installBrush();this.render();}
  private installBrush(){if(!this.snapshot||this.stroke.busy)return;const settings=this.readBrush();this.brushView({settings,begin:p=>{if(!this.available()||!this.snapshot)return false;document.querySelector<HTMLDetailsElement>('.brush-transform')!.open=false;return this.stroke.begin(p,settings,this.snapshot.config);},move:p=>this.stroke.move(p),end:()=>this.stroke.end()});}
  stopBrush(){if(!this.armed&&!this.stroke.busy)return;this.armed=false;this.brushView(null);this.stroke.stop();this.render();}
  get painting(){return this.stroke.busy;}
  private render(){const enabled=this.available();
    $('brush-toolbar').hidden=!this.armed&&!this.stroke.busy;
    const life=!!this.life&&(this.armed||this.stroke.busy);
    $('brush-swatches').hidden=life;$('brush-terrain').hidden=life;$('brush-life').hidden=!life;
    $('brush-title').textContent=life?`PAINTING · ${this.life!.name.toUpperCase()}`:'PAINT THE PLANET';
    document.querySelector('.workspace')?.classList.toggle('brush-mode',this.armed||this.stroke.busy);
    $<HTMLFieldSetElement>('brush-controls').disabled=!enabled;
    $<HTMLButtonElement>('brush-undo').disabled=!this.undo||this.stroke.busy;
    if(this.stroke.busy)$('brush-status').textContent=this.stroke.finishing?'Finishing stroke…':'Painting · time paused';
$<HTMLFieldSetElement>('world-tool-controls').disabled=!enabled;$('world-tools-unavailable').hidden=enabled||this.busy||this.stroke.busy;
    $<HTMLButtonElement>('copy-organism').disabled=!enabled||!this.snapshot?.selected?.observation;
    $<HTMLButtonElement>('edit-organism').disabled=!enabled||!this.snapshot?.selected?.programCells?.length;
    $<HTMLButtonElement>('undo-edit').disabled=!this.undo;
    $<HTMLButtonElement>('apply-genes').disabled=this.busy;
  }
  accept(m:EditorMessage){
    if(m.type==='brush-ack'){this.stroke.accept(m);return;}
    if(m.type==='editor-state'){this.undo=m.canUndo;this.render();return;}
    this.busy=false;
    if(m.type==='editor-specimen'){this.render();this.place({id:crypto.randomUUID(),savedAt:new Date().toISOString(),json:m.json,summary:m.summary});return;}
    if(this.undoing){this.undoing=false;$('brush-status').textContent='Stroke undone · drag to paint';this.render();return;}
    this.cancel();$<HTMLDialogElement>('gene-editor').close();this.render();this.toast(m.message);
  }
  onError(){this.busy=false;this.undoing=false;this.render();}
  private apply(edit:WorldEdit){if(!this.available())return;this.busy=true;this.send({type:'editor',action:'apply',edit});this.render();}
  private openEditor(){
    const s=this.snapshot?.selected;if(!s?.programCells?.length||!this.available())return;
    this.cancel();this.send({type:'control',paused:true});this.body=s.id;this.editCells=structuredClone(s.programCells);
    $<HTMLSelectElement>('edit-cell').innerHTML=this.editCells.map(c=>`<option value="${c.id}">Cell #${c.id} · genome #${c.genome}</option>`).join('');
    $('gene-cell-note').textContent=`Body #${s.id}. ${this.editCells.length} inspectable cells${s.cells>this.editCells.length?` of ${s.cells} (first 64)` : ''}. Only the chosen cell's primary chromosome is edited.`;
    $<HTMLSelectElement>('new-gene-action').innerHTML=(this.snapshot?.autonomous?.actions??[]).map((a,i)=>`<option value="${i}">${escape(a)}</option>`).join('');
    this.chooseCell();$<HTMLDialogElement>('gene-editor').showModal();
  }
  private chooseCell(){const c=this.editCells.find(c=>c.id===Number($<HTMLSelectElement>('edit-cell').value));if(c){this.genes=structuredClone(c.genes);this.renderGenes();}}
  private renderGenes(){const actions=this.snapshot?.autonomous?.actions??[];$('gene-rows').innerHTML=this.genes.map((g,i)=>`<div class="gene-edit-row" data-gene-row="${i}"><strong>${i+1}. ${escape(actions[g.action]??String(g.action))}</strong><label>Bias <input data-bias type="number" min="-2" max="2" step="any" value="${g.bias}"></label><label>Strength <input data-strength type="number" min="0" max="2" step="any" value="${g.strength}"></label><button data-remove-gene="${i}" class="text-button">Remove</button><span class="gene-fates">Read by ${['Stem cells','Fate A','Fate B','Fate C'].map((name,k)=>`<label><input type="checkbox" data-fate="${k}" ${(g.fates??15)>>k&1?'checked':''}>${name}</label>`).join('')}</span><details><summary>Regulatory inputs (${g.promoter.length} weights)</summary><label>Comma-separated promoter weights, each −2 to 2<input data-promoter type="text" value="${g.promoter.join(', ')}"></label></details></div>`).join('');}
  private readGenes(){
    const next=structuredClone(this.genes);
    for(const row of Array.from($('gene-rows').querySelectorAll<HTMLElement>('[data-gene-row]'))){
      const g=next[Number(row.dataset.geneRow)];g.bias=row.querySelector<HTMLInputElement>('[data-bias]')!.valueAsNumber;g.strength=row.querySelector<HTMLInputElement>('[data-strength]')!.valueAsNumber;
      const raw=row.querySelector<HTMLInputElement>('[data-promoter]')!.value.split(',');g.promoter=raw.map(v=>v.trim()===''?NaN:Number(v));
      g.fates=Array.from(row.querySelectorAll<HTMLInputElement>('[data-fate]')).reduce((mask,box)=>mask|(box.checked?1<<Number(box.dataset.fate):0),0);
      if(!g.fates){this.toast('Each gene must be read by at least one kind of cell.');return false;}
      if(!Number.isFinite(g.bias)||Math.abs(g.bias)>2||!Number.isFinite(g.strength)||g.strength<0||g.strength>2||g.promoter.length!==32||g.promoter.some(v=>!Number.isFinite(v)||Math.abs(v)>2)){this.toast('Each gene needs bias −2…2, strength 0…2 and exactly 32 promoter weights −2…2.');return false;}
    }
    this.genes=next;return true;
  }
}
