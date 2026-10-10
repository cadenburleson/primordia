import type {Command,Snapshot,SavedSpecimen,SpecimenMessage} from './sim/types';
import {STRIDE} from './sim/types';
import {listSpecimens,storeSpecimen,deleteSpecimen} from './storage';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const esc=(s:string)=>s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
export const organismBrowserHTML=`<details class="organism-browser"><summary>Find an organism <span id="browser-count"></span></summary><div class="browser-filters"><input id="organism-search" type="search" placeholder="Search body ID" aria-label="Search living body ID"><select id="organism-sort" aria-label="Order living organisms"><option value="large">Largest first</option><option value="small">Smallest first</option><option value="id">Body ID</option></select></div><div id="organism-list"></div><div class="browser-pages"><button id="organism-prev">Previous</button><span id="organism-page"></span><button id="organism-next">Next</button></div></details>`;
export const saveOrganismHTML=`<section class="capture-organism"><label for="specimen-label">Save this organism</label><input id="specimen-label" maxlength="80" placeholder="Give this mutation a name"><button id="capture-organism" class="observe-button" disabled>Save to collection</button><small>Captures every cell and chromosome, structure, material stores and signaling state. Pauses the world.</small><div id="selected-exchanges"></div><div class="selected-inspect"><button id="inspect-diet">Diet &amp; exchanges</button><button id="inspect-function">Evidence of function</button></div></section>`;
export const collectionHTML=`<div class="inspection-intro"><span class="eyebrow">PRESERVED ORGANISMS</span><h3>Your collection</h3><p>Frozen body snapshots. Place several copies in the same world to observe competition, feeding and reproduction.</p></div><button id="import-organism" class="observe-button">Import organism JSON</button><input id="organism-file" type="file" accept="application/json,.json" hidden><p id="collection-status" class="setup-note"></p><div id="specimen-list"></div><p class="setup-note">Copies preserve mutations, chromosomes, age, material stores and learned contact weights. They start at rest with fresh activity counters. Placement pauses the world and adds external material. Resume when your organisms are arranged. An attack or mating outcome depends on their genes and conditions.</p>`;
export class Collection {
  private entries:SavedSpecimen[]=[];
  private snapshot:Snapshot|null=null;
  private page=0;
  private previousId:number|null=null;
  private lastList='';
  private busy=false;
  constructor(private send:(cmd:Command)=>void,private select:(id:number)=>void,private place:(s:SavedSpecimen)=>void,private notify:(message:string)=>void,private open:(tab:string)=>void,private placeOnPlanet:(s:SavedSpecimen)=>void,private changed:(count:number)=>void){
    document.querySelector<HTMLDetailsElement>('.organism-browser')!.addEventListener('toggle',()=>this.browser());
    for(const id of ['organism-search','organism-sort'])$(id).addEventListener(id==='organism-search'?'input':'change',()=>{this.page=0;this.browser();});
    $('organism-prev').addEventListener('click',()=>{this.page--;this.browser();});$('organism-next').addEventListener('click',()=>{this.page++;this.browser();});
    $('organism-list').addEventListener('click',e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-body]');if(b){this.select(Number(b.dataset.body));this.open('life');}});
    $('capture-organism').addEventListener('click',()=>{const o=this.snapshot?.selected;if(!o||this.busy)return;const name=$<HTMLInputElement>('specimen-label').value.trim()||`Body ${o.id} · G${o.generation}`;this.busy=true;this.buttons();this.send({type:'specimen',action:'capture',id:o.id,name});});
    $('inspect-diet').addEventListener('click',()=>this.open('ecology'));$('inspect-function').addEventListener('click',()=>this.open('evidence'));
    $('import-organism').addEventListener('click',()=>$('organism-file').click());
    $('organism-file').addEventListener('change',async()=>{const input=$<HTMLInputElement>('organism-file'),file=input.files?.[0];input.value='';if(!file)return;try{if(file.size>64*1024*1024)throw new Error('Organism file exceeds 64 MB.');this.send({type:'specimen',action:'inspect',json:await file.text()});}catch(e){this.notify(String(e));}});
    $('specimen-list').addEventListener('click',async e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-specimen]');if(!b)return;const s=this.entries.find(s=>s.id===b.dataset.specimen);if(!s)return;try{if(b.dataset.action==='place'){this.place(s);}else if(b.dataset.action==='planet'){this.placeOnPlanet(s);}else if(b.dataset.action==='export'){this.export(s);}else if(b.dataset.action==='delete'){await deleteSpecimen(s.id);this.entries=this.entries.filter(entry=>entry.id!==s.id);this.render();this.notify(`Removed ${s.summary.name} from the collection.`);}}catch(error){this.notify(String(error));}});
    listSpecimens().then(entries=>{this.entries=entries.sort((a,b)=>b.savedAt.localeCompare(a.savedAt));this.render();}).catch(()=>{$('collection-status').textContent='Local collection storage is unavailable. Export captures to keep them.';});
  }
  update(s:Snapshot){
    const redraw=!this.snapshot||!!s.nursery!==!!this.snapshot.nursery;this.snapshot=s;if(redraw)this.render();const o=s.selected;
    if((o?.id??null)!==this.previousId){this.previousId=o?.id??null;$<HTMLInputElement>('specimen-label').value=o?`Body ${o.id} · G${o.generation}`:'';}
    const flux=o?.observation?.flux,origins=(o as typeof o&{collectionSources?:string[]})?.collectionSources??[];
    $('selected-exchanges').innerHTML=o?`<p>Selected <strong>#${o.id}</strong> · ${o.cells.toLocaleString()} cells · G${o.generation}${origins.length?`<br>Collection ancestry: ${origins.map(esc).join(', ')}`:''}</p>${flux?`<small>Current cells' totals: fuel ${(flux.fuel+flux.absorbed).toFixed(3)} · prey ${flux.prey.toFixed(3)} · detritus ${flux.detritus.toFixed(3)} · light-made fuel ${flux.photosynthesis.toFixed(3)} · waste ${flux.waste.toFixed(3)}</small>`:''}`:'Select a living organism to inspect or preserve it.';
    this.buttons();this.browser();
    document.querySelectorAll<HTMLButtonElement>('[data-action="place"],[data-action="planet"]').forEach(b=>b.disabled=!this.canPlace());
  }
  canPlace(){return !!this.snapshot?.autonomous?.physics&&!this.snapshot.assessment?.running&&!this.snapshot.assessment?.control&&!this.snapshot.autonomous.observationControl;}
  refresh(){this.browser();}
  private buttons(){const o=this.snapshot?.selected;$<HTMLButtonElement>('capture-organism').disabled=this.busy||!o?.observation||!this.canPlace();$('capture-organism').textContent=this.busy?'Preserving…':'Save to collection';for(const id of ['inspect-diet','inspect-function'])$<HTMLButtonElement>(id).disabled=!o;}
  onError(){this.busy=false;this.buttons();}
  async accept(m:SpecimenMessage){
    this.busy=false;this.buttons();const s:SavedSpecimen={id:crypto.randomUUID(),savedAt:new Date().toISOString(),json:m.json,summary:m.summary};
    try{await storeSpecimen(s);this.entries.unshift(s);this.render();this.open('collection');this.notify(`Preserved ${s.summary.name} (${s.summary.cells} cells).${m.action==='capture'?' World paused.':''}`);}catch{this.export(s);this.notify('Local storage failed; your organism snapshot was exported instead.');}
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
    this.changed(this.entries.length);const actions=this.snapshot?.autonomous?.actions;
    $('collection-status').textContent=`${this.entries.length} saved organism${this.entries.length===1?'':'s'} · stored in this browser · export for backup`;
    $('specimen-list').innerHTML=this.entries.map(s=>{const o=s.summary,fn=o.functions.map((n,i)=>n?`${esc(actions?.[i]?.replace(' synthesis','')??`Function ${i+1}`)}: ${n} cells`:null).filter(Boolean);return `<article class="specimen-card"><h4>${esc(o.name)}</h4><p>${o.cells.toLocaleString()} cells · G${o.generation} · ${o.genomes} genome${o.genomes===1?'':'s'}</p><small>Body #${o.sourceBody} · seed ${o.sourceSeed} · cycle ${o.cycle.toFixed(1)}<br>${o.bonds} bonds · ${o.joints} joints · ${o.synapses} signaling contacts</small><details><summary>Preserved genes &amp; material functions</summary><p>${fn.join('<br>')||'No material functions above the display threshold.'}</p><small>${o.genes.map((n,i)=>n?`${esc(actions?.[i]??`Gene action ${i+1}`)}: ${n}`:null).filter(Boolean).join('<br>')}</small></details><div class="collection-actions">${this.snapshot?.nursery?`<button data-specimen="${s.id}" data-action="planet" ${this.canPlace()?'':'disabled'}>Return &amp; place on planet</button>`:''}<button data-specimen="${s.id}" data-action="place" ${this.canPlace()?'':'disabled'}>${this.snapshot?.nursery?'Place in nursery':'Place on planet'}</button><button data-specimen="${s.id}" data-action="export">Export</button><button data-specimen="${s.id}" data-action="delete">Remove</button></div></article>`;}).join('')||'<p class="setup-note">Select an organism in Life, name it, then choose Save to collection.</p>';
  }
}
