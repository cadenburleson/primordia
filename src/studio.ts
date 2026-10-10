import type {Snapshot,PhysicalBody} from './sim/types';
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
  host.innerHTML=`<span class="eyebrow">HOW IT FEEDS</span><p>${total>0?`It lives mostly on <strong>${ranked[0][0].toLowerCase()}</strong>.`:'These cells have not fed yet.'}</p>${total>0?`<div class="diet-bar" aria-label="Measured fuel sources">${ranked.map(([label,v])=>`<span style="flex:${v/total}" title="${label}: ${(v/total*100).toFixed(1)}%" data-diet="${label}"></span>`).join('')}</div><div class="diet-key">${ranked.map(([label,v])=>`<span>${label} <b>${Math.round(v/total*100)}%</b></span>`).join('')}</div>`:''}${mouth.cells?`<p class="body-function-note"><strong>${mouth.cells} mouth cell${mouth.cells===1?'':'s'}</strong>${mouth.largest>1?`, the largest mouth ${mouth.largest} cells across`:''}. The whole body’s bulk decides what it can swallow.</p>`:''}${gut>0&&total>0?`<p class="body-function-note"><strong>It has a gut.</strong> Its mouth cells have passed ${Math.round(Math.min(1,gut/total)*100)}% of everything this body took in to cells that have no mouth.</p>`:''}<p class="body-function-note">${functions} kind${functions===1?'':'s'} of working material · ${work.toFixed(3)} spent on movement.</p><small>Totals since each living cell was born, so they shift as cells die and divide.</small>`;
}
