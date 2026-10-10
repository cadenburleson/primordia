import type {Snapshot} from './sim/types';
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
  const sources=[['Light',o.flux.photosynthesis],['Dissolved fuel',o.flux.fuel+o.flux.absorbed],['Organic remains',o.flux.detritus],['Living prey / matrix',o.flux.prey]] as const;
  const total=sources.reduce((sum,[,v])=>sum+v,0),ranked=sources.filter(([,v])=>v>0).sort((a,b)=>b[1]-a[1]);
  const work=s.selected!.body.work,functions=o.functions.reduce((n,v)=>n+Number(v>0),0);
  host.innerHTML=`<span class="eyebrow">OBSERVED STRATEGY</span><p>${total>0?`Most measured fuel comes from <strong>${ranked[0][0].toLowerCase()}</strong>.`:'No fuel acquisition recorded for these living cells yet.'}</p>${total>0?`<div class="diet-bar" aria-label="Measured fuel sources">${ranked.map(([label,v])=>`<span style="flex:${v/total}" title="${label}: ${(v/total*100).toFixed(1)}%" data-diet="${label}"></span>`).join('')}</div><div class="diet-key">${ranked.map(([label,v])=>`<span>${label} <b>${Math.round(v/total*100)}%</b></span>`).join('')}</div>`:''}<p class="body-function-note">${functions} material function${functions===1?'':'s'} above the display threshold · ${work.toFixed(3)} contraction work.</p><small>Current cells’ accumulated activity since birth. Composition can change as cells die or divide; this is not an instantaneous rate.</small>`;
}
