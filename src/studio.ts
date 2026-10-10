import type {Snapshot,PhysicalBody,ProgramCell} from './sim/types';
import {cellType} from './sim/roles';

/** Size of the largest patch of joined mouth cells in a body: its biggest mouth. */
function largestMouth(body:PhysicalBody){
  const mouth=body.nodes.map(n=>cellType(n.role)===15),seen=new Array<boolean>(mouth.length).fill(false);
  const next:number[][]=mouth.map(()=>[]);
  for(const l of body.links)if(mouth[l.a]&&mouth[l.b]){next[l.a].push(l.b);next[l.b].push(l.a);}
  let best=0;
  for(let i=0;i<mouth.length;i++){
    if(!mouth[i]||seen[i])continue;
    let size=0;const stack=[i];seen[i]=true;
    while(stack.length){const k=stack.pop()!;size++;for(const j of next[k])if(!seen[j]){seen[j]=true;stack.push(j);}}
    best=Math.max(best,size);
  }
  return {largest:best,cells:mouth.filter(Boolean).length};
}
/** How much of a working material a cell has built, as the engine measures it. */
const built=(c:ProgramCell,action:number)=>{const m=c.material,p=m?.proteins[action]??0;return m&&p>0?p/(.015*m.biomass+p):0;};
const has=(c:ProgramCell,action:number)=>built(c,action)>.25;
/** The working parts a single cell can carry: its organs. */
const PARTS:[string,(c:ProgramCell)=>boolean][]=[
  ['Light harvester',c=>has(c,0)],['Captured light-feeder',c=>(c.state?.symbiont??0)>0],['Vent catalyst',c=>has(c,37)],['Mouth',c=>has(c,34)],['Digestive enzymes',c=>has(c,9)||has(c,15)],
  ['Wall',c=>has(c,41)],['Skin',c=>has(c,14)],['Spines',c=>has(c,36)],['Venom',c=>has(c,35)],['Motor',c=>has(c,32)],['Rudder',c=>has(c,33)],['Muscle fibre',c=>has(c,4)],['Fin',c=>has(c,42)],
  ['Sensors',c=>has(c,11)||has(c,16)||has(c,24)],['Signal channels',c=>has(c,12)],['Glue',c=>has(c,2)],['Growth compass',c=>has(c,20)],['Copying machinery',c=>has(c,7)],['Fat store',c=>!!c.material&&c.material.lipid/Math.max(c.material.biomass,1e-9)>.08]];
const FATE_NAMES=['stem','fate A','fate B','fate C'];
/** Which broad kind of life a body is, by the engine's own rule. */
function kindOfLife(cells:ProgramCell[],own:number,intake:number){
  if(cells.filter(c=>has(c,41)).length*2>=cells.length&&cells.length)return intake<=0?null:own*2>=intake?['Plant-like','walled, and it makes its own food']:['Fungus-like','walled, and it absorbs or digests its food'];
  if(cells.some(c=>has(c,34)||has(c,32)||has(c,42)))return ['Animal-like','bare-skinned, with a mouth, a motor or fins'];
  return cells.length>1?['A simple colony','alike cells with no wall, mouth, motor or fin']:['A free microbe','no wall, mouth, motor or fin'];
}
/** A body's build: bone, joints, muscle, feet and stores, counted from its cells. */
function bodyPlan(cells:ProgramCell[],joints:number){
  if(cells.length<3)return '';
  const bone=cells.filter(c=>!!c.material&&c.material.mineral/Math.max(c.material.biomass+c.material.fuel+c.material.mineral+c.material.lipid,1e-9)>.025).length;
  const count=(test:(c:ProgramCell)=>boolean)=>cells.filter(test).length,muscle=count(c=>has(c,4)),fins=count(c=>has(c,42)),beat=count(c=>has(c,12)),fat=count(c=>!!c.material&&c.material.lipid/Math.max(c.material.biomass,1e-9)>.08),aimed=count(c=>has(c,20));
  const parts=[bone?`${bone} bone cell${bone===1?'':'s'}${joints?` meeting at ${joints} joint${joints===1?'':'s'}`:''}`:'',muscle?`${muscle} muscle`:'',beat?`${beat} that can set a beat`:'',fins?`${fins} fin${fins===1?'':'s'} or feet`:'',fat?`${fat} storing fat`:''].filter(Boolean);
  const moves=muscle&&fins&&beat?'It has everything it needs to row or crawl.':muscle&&!fins?'Its muscle has nothing to push with: no fins or feet.':muscle&&!beat?'Its muscle has nothing to set a beat.':'';
  return `<span class="eyebrow">BODY PLAN</span><p>${parts.length?`${parts.join(', ')}.`:'No bone, muscle, fins or fat yet: a soft body.'} ${aimed*2>=cells.length?'Its cells aim their divisions, so it grows to a shape.':'It grows as a lump: its cells divide whichever way they happen to face.'} ${moves}</p>`;
}
function insides(cells:ProgramCell[],total:number){
  if(!cells.length)return '';
  const parts=PARTS.map(([name,test])=>[name,cells.filter(test).length] as const).filter(([,n])=>n>0);
  const fates=FATE_NAMES.map((name,f)=>[name,cells.filter(c=>(c.state?.fate??0)===f)] as const).filter(([,list])=>list.length);
  const workers=cells.filter(c=>(c.state?.fate??0)>0&&!has(c,7)).length,shown=cells.length<total?` (first ${cells.length} cells)`:'';
  return `<span class="eyebrow">INSIDE ITS CELLS</span><p>A cell is a body in miniature. These are its organs${shown}:</p><div class="cell-parts">${parts.length?parts.map(([name,n])=>`<span>${name}${cells.length>1?` <b>${n}</b>`:''}</span>`).join(''):'<span>Nothing built yet</span>'}</div>${fates.length>1||fates[0]?.[0]!=='stem'?`<p class="body-function-note"><strong>${fates.length} kind${fates.length===1?'':'s'} of cell by fate:</strong> ${fates.map(([name,list])=>`${list.length} ${name}`).join(', ')}. Each fate reads a different part of the same genome.${workers?` ${workers} committed cell${workers===1?' has':'s have'} given up dividing: workers.`:''}</p>`:''}`;
}
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const descriptions:Record<string,string>={progress:'The story so far',life:'Select a body. Discover its strategy.',ecology:'What is available, eaten, and produced?',evidence:'Does this mechanism actually help?',collection:'Preserve a lineage. Place it elsewhere.',notes:'A record of this world’s changes'};
type StudioPanel='guide'|'powers'|'shape'|'observe'|null;
let panel:StudioPanel=null,currentTab='progress';
export function setStudioPanel(next:StudioPanel){
  panel=next;const workspace=document.querySelector<HTMLElement>('.workspace')!;
  workspace.classList.toggle('hide-shape',next!=='shape');workspace.classList.toggle('hide-observe',next!=='observe');workspace.classList.toggle('panel-open',next!==null);
  document.querySelector<HTMLElement>('.world-panel')!.hidden=next!=='shape';document.querySelector<HTMLElement>('.life-panel')!.hidden=next!=='observe';$('evolution-guide').hidden=next!=='guide';$('powers-panel').hidden=next!=='powers';
  for(const [id,active] of [['toggle-guide',next==='guide'],['toggle-powers',next==='powers'],['toggle-shape',next==='shape'],['toggle-observe',next==='observe'&&currentTab!=='collection'],['toggle-collection',next==='observe'&&currentTab==='collection']] as const){$(id).classList.toggle('active',active);$(id).setAttribute('aria-expanded',String(active));}
}
export function updateStudio(tab:string){
  currentTab=tab;$('observer-context').textContent=descriptions[tab]??'';
  $<HTMLSelectElement>('observer-view').value=['evidence','collection','notes'].includes(tab)?tab:'';
  setStudioPanel('observe');
}
export function initStudio(openTab:(tab:string)=>void){
  $('toggle-guide').onclick=()=>setStudioPanel(panel==='guide'?null:'guide');
  $('guide-hint').onclick=()=>setStudioPanel('guide');
  $('toggle-powers').onclick=()=>setStudioPanel(panel==='powers'?null:'powers');
  $('toggle-shape').onclick=()=>setStudioPanel(panel==='shape'?null:'shape');
  $('toggle-observe').onclick=()=>{if(panel==='observe'&&currentTab!=='collection')setStudioPanel(null);else openTab(currentTab==='collection'?'progress':currentTab);};
  $('toggle-collection').onclick=()=>{if(panel==='observe'&&currentTab==='collection')setStudioPanel(null);else openTab('collection');};
  document.querySelectorAll<HTMLButtonElement>('[data-close-panel]').forEach(b=>b.onclick=()=>setStudioPanel(null));
  $('observer-view').onchange=()=>{const value=$<HTMLSelectElement>('observer-view').value;if(value)openTab(value);};
  $('progress-inspect').onclick=()=>$('observe').click();$('progress-resources').onclick=()=>openTab('ecology');
  const menu=$<HTMLDetailsElement>('studio-menu');
  menu.addEventListener('click',e=>{if((e.target as HTMLElement).closest('button'))menu.open=false;});
  document.addEventListener('click',e=>{if(!menu.contains(e.target as Node))menu.open=false;});
  document.addEventListener('keydown',e=>{if(e.key==='Escape'){menu.open=false;if(!document.querySelector('dialog[open]'))setStudioPanel(null);}});
  setStudioPanel(null);
}

export function describeBody(s:Snapshot){
  const host=$('body-story'),o=s.selected?.observation;host.hidden=!o;if(!o)return;
  const sources=[['Light',o.flux.photosynthesis],['Dissolved fuel',o.flux.fuel+o.flux.absorbed],['Organic remains',o.flux.detritus],['Living prey',o.flux.prey],['Vents',o.flux.chemosynthesis??0]] as const;
  const total=sources.reduce((sum,[,v])=>sum+v,0),ranked=sources.filter(([,v])=>v>0).sort((a,b)=>b[1]-a[1]);
  const work=s.selected!.body.work,functions=o.functions.reduce((n,v)=>n+Number(v>0),0),gut=o.flux.shared??0,mouth=largestMouth(s.selected!.body);
  const cells=s.selected!.programCells??[],kind=cells.length?kindOfLife(cells,o.flux.photosynthesis+(o.flux.chemosynthesis??0),total):null;
  host.innerHTML=`${kind?`<span class="eyebrow">WHAT IT IS</span><p><strong>${kind[0]}</strong>: ${kind[1]}.</p>`:''}<span class="eyebrow">HOW IT FEEDS</span><p>${total>0?`It lives mostly on <strong>${ranked[0][0].toLowerCase()}</strong>.`:'These cells have not fed yet.'}</p>${total>0?`<div class="diet-bar" aria-label="Measured fuel sources">${ranked.map(([label,v])=>`<span style="flex:${v/total}" title="${label}: ${(v/total*100).toFixed(1)}%" data-diet="${label}"></span>`).join('')}</div><div class="diet-key">${ranked.map(([label,v])=>`<span>${label} <b>${Math.round(v/total*100)}%</b></span>`).join('')}</div>`:''}${mouth.cells?`<p class="body-function-note"><strong>${mouth.cells} mouth cell${mouth.cells===1?'':'s'}</strong>${mouth.largest>1?`, the largest mouth ${mouth.largest} cells across`:''}. The whole body’s bulk decides what it can swallow.</p>`:''}${gut>0&&total>0?`<p class="body-function-note"><strong>It has a gut.</strong> Its mouth cells have passed ${Math.round(Math.min(1,gut/total)*100)}% of everything this body took in to cells that have no mouth.</p>`:''}<p class="body-function-note">${functions} kind${functions===1?'':'s'} of working material · ${work.toFixed(3)} spent on movement.</p><small>Totals since each living cell was born, so they shift as cells die and divide.</small>${bodyPlan(cells,s.selected!.body.joints.length)}${insides(cells,s.selected!.cells)}`;
}
