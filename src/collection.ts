import type {Command,Snapshot,SavedSpecimen,SpecimenMessage,SpecimenSummary,ChampionMessage} from './sim/types';
import {STRIDE} from './sim/types';
import {listSpecimens,storeSpecimen,deleteSpecimen} from './storage';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const esc=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
export const organismBrowserHTML=`<details class="organism-browser"><summary>Find an organism <span id="browser-count"></span></summary><div class="browser-filters"><input id="organism-search" type="search" placeholder="Search body ID" aria-label="Search living body ID"><select id="organism-sort" aria-label="Order living organisms"><option value="large">Largest first</option><option value="small">Smallest first</option><option value="id">Body ID</option></select></div><div id="organism-list"></div><div class="browser-pages"><button id="organism-prev">Previous</button><span id="organism-page"></span><button id="organism-next">Next</button></div></details>`;
export const saveOrganismHTML=`<section class="capture-organism"><label for="specimen-label">Keep this creature</label><input id="specimen-label" maxlength="80" placeholder="Give it a name"><button id="capture-organism" class="observe-button" disabled>Save to Creatures</button><small>Keeps every cell, gene and store, so you can paint copies anywhere.</small><div id="selected-exchanges"></div><div class="selected-inspect"><button id="inspect-diet">Diet &amp; exchanges</button><button id="inspect-function">Evidence of function</button></div></section>`;
/** How many automatically kept record-holders are held at once; the oldest make way. */
const AUTO_LIMIT=12;
/** A body's outline from above, drawn from its saved cell positions. */
function silhouette(o:SpecimenSummary){
  const cells=o.shape?.length?o.shape:[[0,0,0,.22]];
  let minX=Infinity,maxX=-Infinity,minZ=Infinity,maxZ=-Infinity;
  for(const [x,,z,r] of cells){minX=Math.min(minX,x-r);maxX=Math.max(maxX,x+r);minZ=Math.min(minZ,z-r);maxZ=Math.max(maxZ,z+r);}
  const span=Math.max(maxX-minX,maxZ-minZ,.9),cx=(minX+maxX)/2,cz=(minZ+maxZ)/2,k=36/span;
  return `<svg viewBox="0 0 40 40" aria-hidden="true">${cells.map(([x,,z,r])=>`<circle cx="${(20+(x-cx)*k).toFixed(1)}" cy="${(20+(z-cz)*k).toFixed(1)}" r="${Math.max(1.4,r*k).toFixed(1)}"/>`).join('')}</svg>`;
}
export class Collection {
  private entries:SavedSpecimen[]=[];
  private snapshot:Snapshot|null=null;
  private page=0;
  private previousId:number|null=null;
  private lastList='';
  private busy=false;private couldPlace=false;
  constructor(private send:(cmd:Command)=>void,private select:(id:number)=>void,private place:(s:SavedSpecimen)=>void,private notify:(message:string)=>void,private open:(tab:string)=>void,private placeOnPlanet:(s:SavedSpecimen)=>void,private changed:(count:number)=>void,private drawn:()=>void=()=>{}){
    document.querySelector<HTMLDetailsElement>('.organism-browser')!.addEventListener('toggle',()=>this.browser());
    for(const id of ['organism-search','organism-sort'])$(id).addEventListener(id==='organism-search'?'input':'change',()=>{this.page=0;this.browser();});
    $('organism-prev').addEventListener('click',()=>{this.page--;this.browser();});$('organism-next').addEventListener('click',()=>{this.page++;this.browser();});
    $('organism-list').addEventListener('click',e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-body]');if(b){this.select(Number(b.dataset.body));this.open('life');}});
    $('capture-organism').addEventListener('click',()=>{const o=this.snapshot?.selected;if(!o||this.busy)return;const name=$<HTMLInputElement>('specimen-label').value.trim()||`Body ${o.id} · G${o.generation}`;this.busy=true;this.buttons();this.send({type:'specimen',action:'capture',id:o.id,name});});
    $('inspect-diet').addEventListener('click',()=>this.open('ecology'));$('inspect-function').addEventListener('click',()=>this.open('evidence'));
    $('import-organism').addEventListener('click',()=>$('organism-file').click());
    $('organism-file').addEventListener('change',async()=>{const input=$<HTMLInputElement>('organism-file'),file=input.files?.[0];input.value='';if(!file)return;try{if(file.size>64*1024*1024)throw new Error('Organism file exceeds 64 MB.');this.send({type:'specimen',action:'inspect',json:await file.text()});}catch(e){this.notify(String(e));}});
    $('specimen-list').addEventListener('click',async e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-specimen]');if(!b)return;const s=this.entries.find(s=>s.id===b.dataset.specimen);if(!s)return;try{
      if(b.dataset.action==='paint'){if(this.snapshot?.nursery&&b.dataset.planet)this.placeOnPlanet(s);else this.place(s);}
      else if(b.dataset.action==='keep'){await deleteSpecimen(s.id);const kept:SavedSpecimen={...s,id:crypto.randomUUID(),savedAt:new Date().toISOString(),auto:undefined};await storeSpecimen(kept);this.entries=[kept,...this.entries.filter(entry=>entry.id!==s.id)];this.render();this.notify(`${s.summary.name} is now yours to keep.`);}
      else if(b.dataset.action==='export'){this.export(s);}
      else if(b.dataset.action==='delete'){await deleteSpecimen(s.id);this.entries=this.entries.filter(entry=>entry.id!==s.id);this.render();this.notify(`Removed ${s.summary.name}.`);}
    }catch(error){this.notify(String(error));}});
    listSpecimens().then(entries=>{this.entries=entries.sort((a,b)=>b.savedAt.localeCompare(a.savedAt));this.render();}).catch(()=>{$('collection-status').textContent='This browser cannot store creatures. Export the ones you want to keep.';});
  }
  update(s:Snapshot){
    const redraw=!this.snapshot||!!s.nursery!==!!this.snapshot.nursery||s.config.seed!==this.snapshot.config.seed||this.canPlace()!==this.couldPlace;this.snapshot=s;this.couldPlace=this.canPlace();if(redraw)this.render();const o=s.selected;
    if((o?.id??null)!==this.previousId){this.previousId=o?.id??null;$<HTMLInputElement>('specimen-label').value=o?`Body ${o.id} · G${o.generation}`:'';}
    const flux=o?.observation?.flux,origins=(o as typeof o&{collectionSources?:string[]})?.collectionSources??[];
    $('selected-exchanges').innerHTML=o?`<p>Selected <strong>#${o.id}</strong> · ${o.cells.toLocaleString()} cells · G${o.generation}${origins.length?`<br>Collection ancestry: ${origins.map(esc).join(', ')}`:''}</p>${flux?`<small>Current cells' totals: fuel ${(flux.fuel+flux.absorbed).toFixed(3)} · prey ${flux.prey.toFixed(3)} · detritus ${flux.detritus.toFixed(3)} · light-made fuel ${flux.photosynthesis.toFixed(3)} · waste ${flux.waste.toFixed(3)}</small>`:''}`:'Select a living organism to inspect or preserve it.';
    this.buttons();this.browser();
    document.querySelectorAll<HTMLButtonElement>('[data-action="paint"]').forEach(b=>b.disabled=!this.canPlace());
  }
  canPlace(){return !!this.snapshot?.autonomous?.physics&&!this.snapshot.assessment?.running&&!this.snapshot.assessment?.control&&!this.snapshot.autonomous.observationControl;}
  refresh(){this.browser();}
  private buttons(){const o=this.snapshot?.selected;$<HTMLButtonElement>('capture-organism').disabled=this.busy||!o?.observation||!this.canPlace();$('capture-organism').textContent=this.busy?'Saving…':'Save to Creatures';for(const id of ['inspect-diet','inspect-function'])$<HTMLButtonElement>(id).disabled=!o;}
  onError(){this.busy=false;this.buttons();}
  async accept(m:SpecimenMessage){
    this.busy=false;this.buttons();const s:SavedSpecimen={id:crypto.randomUUID(),savedAt:new Date().toISOString(),json:m.json,summary:m.summary};
    try{await storeSpecimen(s);this.entries.unshift(s);this.render();this.open('collection');this.notify(`Saved ${s.summary.name} (${s.summary.cells} cells). Pick it in Creatures to paint copies.`);}catch{this.export(s);this.notify('This browser could not store it, so it was exported as a file instead.');}
  }
  /** A new record-holder arrived from the planet: keep it unless a better one of the same world is already held. */
  async acceptChampion(m:ChampionMessage){
    const id=`auto:${m.seed}:${m.slot}`,held=this.entries.find(e=>e.id===id);
    if(held?.auto&&held.auto.score>=m.score*.95&&held.auto.time<=m.time)return false;
    const s:SavedSpecimen={id,savedAt:new Date().toISOString(),json:m.json,summary:m.summary,auto:{slot:m.slot,title:m.title,note:m.note,score:m.score,seed:m.seed,time:m.time}};
    try{
      await storeSpecimen(s);this.entries=[s,...this.entries.filter(e=>e.id!==id)];
      // Older planets' records make way first.
      const autos=this.entries.filter(e=>e.auto).sort((a,b)=>Number(b.auto!.seed===m.seed)-Number(a.auto!.seed===m.seed)||b.savedAt.localeCompare(a.savedAt));
      for(const old of autos.slice(AUTO_LIMIT)){await deleteSpecimen(old.id);this.entries=this.entries.filter(e=>e.id!==old.id);}
      this.render();return !held;
    }catch{return false;}
  }
  private export(s:SavedSpecimen){const url=URL.createObjectURL(new Blob([s.json],{type:'application/json'})),a=document.createElement('a');a.href=url;a.download=`primordia-organism-${s.summary.name.replace(/[^a-z0-9_-]+/gi,'-').slice(0,70)}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
  private browser(){
    const s=this.snapshot;if(!s)return;$('browser-count').textContent=`(${s.ids.length.toLocaleString()})`;
    if(!document.querySelector<HTMLDetailsElement>('.organism-browser')!.open||$('life-content').hidden)return;
    const query=$<HTMLInputElement>('organism-search').value.replace('#','').trim(),sort=$<HTMLSelectElement>('organism-sort').value;
    const rows=Array.from(s.ids,(id,index)=>({id,cells:s.data[index*STRIDE+3]})).filter(r=>!query||String(r.id).includes(query));
    rows.sort((a,b)=>sort==='large'?b.cells-a.cells||a.id-b.id:sort==='small'?a.cells-b.cells||a.id-b.id:a.id-b.id);
    const pages=Math.max(1,Math.ceil(rows.length/20));this.page=Math.max(0,Math.min(pages-1,this.page));
    $('browser-count').textContent=`(${s.ids.length.toLocaleString()})`;$('organism-page').textContent=`${this.page+1} / ${pages}`;$<HTMLButtonElement>('organism-prev').disabled=this.page===0;$<HTMLButtonElement>('organism-next').disabled=this.page===pages-1;
    const html=rows.slice(this.page*20,(this.page+1)*20).map(r=>`<button data-body="${r.id}" class="organism-row ${s.selected?.id===r.id?'active':''}"><strong>#${r.id}</strong><span>${r.cells.toLocaleString()} cell${r.cells===1?'':'s'}</span></button>`).join('')||'<p>No matching living organisms.</p>';
    if(html!==this.lastList){$('organism-list').innerHTML=html;this.lastList=html;}
  }
  private render(){
    this.changed(this.entries.length);const seed=this.snapshot?.config.seed,nursery=!!this.snapshot?.nursery,can=this.canPlace();
    const mine=this.entries.filter(e=>!e.auto),order=['advanced','largest','hunter','swimmer'],best=this.entries.filter(e=>e.auto&&e.auto.seed===seed).sort((a,b)=>order.indexOf(a.auto!.slot)-order.indexOf(b.auto!.slot)),earlier=this.entries.filter(e=>e.auto&&e.auto.seed!==seed);
    $('collection-status').textContent=this.entries.length?'Kept in this browser. Export a creature to back it up.':'';
    const card=(s:SavedSpecimen)=>{const o=s.summary,a=s.auto;return `<article class="creature-row"><button class="creature-card" data-specimen="${s.id}" data-action="paint" aria-pressed="false" ${can?'':'disabled'} title="Paint copies onto the map">${silhouette(o)}<b>${esc(a?a.title:o.name)}</b><small>${a?esc(a.note):`${o.cells.toLocaleString()} cell${o.cells===1?'':'s'} · generation ${o.generation}`}</small></button><span class="creature-actions">${nursery?`<button data-specimen="${s.id}" data-action="paint" data-planet="1" ${can?'':'disabled'} title="Leave the nursery and paint it on your planet">To planet</button>`:''}${a?`<button data-specimen="${s.id}" data-action="keep" title="Stop this record being replaced">Keep</button>`:''}<button data-specimen="${s.id}" data-action="export" title="Download as a file">Export</button><button data-specimen="${s.id}" data-action="delete" title="Remove">×</button></span></article>`;};
    const group=(title:string,list:SavedSpecimen[],empty='')=>list.length||empty?`<div class="power-head"><span class="eyebrow">${title}</span></div>${list.map(card).join('')||`<p class="power-note">${empty}</p>`}`:'';
    $('specimen-list').innerHTML=group('THIS PLANET’S BEST · KEPT FOR YOU',best,this.snapshot?.autonomous?.physics?'The most advanced bodies will appear here by themselves as life develops.':'')+group('SAVED BY YOU',mine,'Inspect a body and choose Save to Creatures.')+group('FROM EARLIER PLANETS',earlier);
    this.drawn();
  }
}
