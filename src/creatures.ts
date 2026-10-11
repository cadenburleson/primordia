/** Creatures: every lifeform you can release, in one palette. Pick one, then paint it onto the map. */
import type {Command,Snapshot,SteerAction} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;

/** What the life brush is loaded with. A starter species is built by the engine; anything else is a snapshot. */
export interface LifeSource {token:string;name:string;species?:string;json?:string}
/** Starter species: key, name, colour on the planet's charts, what it is. */
export const SPECIES:[string,string,string,string][]=[
  ['alga','Alga','#4f9e4a','A walled plant. Grows in clumps'],
  ['swimmer','Swimmer','#ff9d3c','Swims toward the light'],
  ['grazer','Grazer','#b77bf0','Digests what it touches'],
  ['hunter','Hunter','#d6249f','A big cell that draws in and swallows smaller ones'],
  ['thornback','Thornback','#8f9bb8','Spines and skin: hard to eat'],
  ['rower','Rower','#2f6be0','Beats muscle and rows on fins'],
  ['courter','Courter','#f58fc0','Its gametes find a mate and fuse'],
  ['decomposer','Decomposer','#a9865b','Tiny. Rots remains and frees their nitrogen'],
  ['plankton','Plankton','#9bd66a','Tiny free light-feeder. Breeds fast; food for anything with a mouth'],
  ['leviathan','Leviathan','#7a2f8f','A very big cell that swallows hunters. Slow to breed'],
  ['volvox','Volvox','#3fbf8f','An egg that divides, without growing, into a ball of small cells'],
  ['drifter','Drifter','#7fb8a4','The plain first cell']];

/** Starters left out of "fill the seas": the plain first cell, which every other starter is built from. It can still be painted. */
const SOLO=new Set(['drifter']);

export const creaturesHTML=`<aside id="creatures-panel" class="guide-panel creatures-panel" aria-label="Creatures" hidden>
<div class="drawer-heading"><span class="eyebrow">CREATURES</span><button data-close-panel class="icon-button" aria-label="Close creatures">×</button></div>
<h2>Paint life<br>onto your planet.</h2><p class="guide-intro">Pick a creature, then drag across the map.</p>
<p id="creatures-unavailable" class="power-note" hidden>Creatures are released on your main planet and its nursery. Return there first.</p>
<section class="creature-group"><div class="power-head"><span class="eyebrow">STARTER SPECIES</span><strong id="species-readout"></strong></div>
<div class="creature-grid" id="species-choices" role="group" aria-label="Starter species">${SPECIES.map(([key,name,color,note])=>`<button class="creature-card" data-species="${key}" aria-pressed="false" title="${note}"><i style="background:${color}"></i><b>${name}</b><small>${note}</small><em data-living="${name}"></em></button>`).join('')}</div>
<button id="seed-all" class="observe-button">Fill the seas with all of them <span>↗</span></button>
<label class="power-toggle"><input type="checkbox" id="sustain-species" checked> Restock a species if it is about to die out</label></section>
<section class="creature-group"><div id="specimen-list"></div></section>
<section class="creature-group creature-tools"><button id="copy-organism" class="observe-button" disabled>Paint copies of the selected body</button><p id="world-tool-selection" class="power-note">Click a body on the planet to copy it.</p>
<button id="import-organism" class="text-button">Import a creature file…</button><input id="organism-file" type="file" accept="application/json,.json" hidden><p id="collection-status" class="power-note"></p></section>
<p class="power-note">Starter species are single cells built from ordinary genes. Everything you release has to feed, grow and breed like the rest, and evolves from its first division.</p>
</aside>`;

export class Creatures {
  private snapshot:Snapshot|null=null;private active:string|null=null;private signature='';
  /** `arm` loads the life brush; it answers whether the brush could be armed. */
  constructor(private send:(c:Command)=>void,private arm:(source:LifeSource)=>boolean,private toast:(m:string)=>void){
    $('species-choices').onclick=e=>{
      const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-species]');if(!b||b.disabled)return;
      const [key,name]=SPECIES.find(s=>s[0]===b.dataset.species)!;
      this.paint({token:`species:${key}`,name,species:key});
    };
    $('seed-all').onclick=()=>{const sustain=$<HTMLInputElement>('sustain-species').checked;for(const [species] of SPECIES)if(!SOLO.has(species))this.steer({kind:'seed',species,count:24,sustain});};
    $('sustain-species').onchange=()=>this.steer({kind:'sustain',on:$<HTMLInputElement>('sustain-species').checked});
  }
  get sustain(){return $<HTMLInputElement>('sustain-species').checked;}
  private steer(action:SteerAction){this.send({type:'steer',action});}
  /** Load the brush with a creature and mark its card. */
  paint(source:LifeSource){
    if(!this.available()){this.toast('Return to your main planet to release creatures.');return;}
    if(this.arm(source)){this.active=source.token;this.mark();}
  }
  /** The brush was put down. */
  released(){if(this.active!==null){this.active=null;this.mark();}}
  get painting(){return this.active;}
  private mark(){document.querySelectorAll<HTMLElement>('#creatures-panel [data-species],#creatures-panel [data-specimen]').forEach(el=>{const on=(el.dataset.species?`species:${el.dataset.species}`:el.dataset.specimen)===this.active;el.classList.toggle('active',on);el.setAttribute('aria-pressed',String(on));});}
  private available(){const s=this.snapshot;return !!s?.autonomous?.physics&&!s.assessment?.control&&!s.assessment?.running&&(s.config.physicalAssay??'full')==='full';}
  /** Re-mark cards after the saved list has been redrawn. */
  refresh(){this.mark();}
  update(s:Snapshot){
    this.snapshot=s;const st=s.autonomous?.steering,available=this.available();
    const signature=JSON.stringify([available,st?.species??[],st?.restocked??0]);
    if(signature===this.signature)return;this.signature=signature;
    $('creatures-unavailable').hidden=available;
    document.querySelectorAll<HTMLButtonElement>('#species-choices button,#seed-all').forEach(b=>b.disabled=!available);
    const living=new Map(st?.species??[]);
    $('species-readout').textContent=living.size?`${living.size} alive${st?.restocked?` · ${st.restocked} restocked`:''}`:'';
    document.querySelectorAll<HTMLElement>('#species-choices [data-living]').forEach(el=>{const n=living.get(el.dataset.living!);el.textContent=n?`${n.toLocaleString()} living`:'';});
  }
}
