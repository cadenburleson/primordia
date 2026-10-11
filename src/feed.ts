/** Happening now: who is eating whom and who has mated, newest first. Click a line to go there. */
import {CELL_TYPES,cellType} from './sim/roles';
import type {Snapshot} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
export const feedHTML=`<div id="feed" class="feed" hidden><span class="eyebrow">HAPPENING NOW</span><ol id="feed-list"></ol></div>`;
interface Item {key:string;time:number;text:string;body:number;kind:'meal'|'mating'}
const VERBS:Record<string,string>={engulfed:'swallowed','living biomass':'is digesting','matrix / fibers':'is eating into'};
const article=(name:string)=>`${/^[aeiou]/i.test(name)?'an':'a'} ${name}`;
/** A starter species by name, or else the kind of cell it was. */
function who(species:string|undefined,role:number|undefined){
  if(species)return article(species);
  const type=CELL_TYPES[cellType(role??0)]?.name??'Feeding';
  return article(type==='Feeding'?'drifting cell':`${type.toLowerCase()} cell`);
}
export class Feed {
  private items:Item[]=[];private world='';private signature='';
  constructor(pick:(body:number)=>void){
    $('feed-list').onclick=e=>{const li=(e.target as HTMLElement).closest<HTMLElement>('[data-body]');if(li&&Number(li.dataset.body))pick(Number(li.dataset.body));};
  }
  update(s:Snapshot|null){
    const a=s?.autonomous,host=$('feed');
    if(!s||!a?.physics){host.hidden=true;this.items=[];this.signature='';return;}
    const world=`${s.config.seed}/${s.config.name}/${s.nursery?'nursery':'planet'}`,now=s.stats.time;
    if(world!==this.world||this.items.some(i=>i.time>now+1)){this.world=world;this.items=[];}
    const add=(item:Item)=>{const old=this.items.findIndex(i=>i.key===item.key);if(old>=0)this.items.splice(old,1);this.items.push(item);};
    for(const m of a.feeding??[]){
      // A nibble is not news: show only what was eaten in earnest.
      if(m.amount<.01)continue;
      const eater=who(m.attacker_kind,m.attacker_role),eaten=who(m.prey_kind,m.prey_role);
      add({key:`m${m.attacker}:${m.prey}:${m.material}`,time:m.time,kind:'meal',body:m.attacker_body??0,text:`${eater[0].toUpperCase()}${eater.slice(1)} ${VERBS[m.material]??'is eating'} ${eaten}`});
    }
    for(const z of a.development?.recent??[])add({key:`z${z.child}`,time:z.time,kind:'mating',body:0,text:`Two gametes fused: a new life with two parents, generation ${z.generation}`});
    // Keep what happened in the last sixty cycles, newest first, five lines at most.
    this.items=this.items.filter(i=>now-i.time<60).sort((x,y)=>y.time-x.time).slice(0,5);
    const signature=this.items.map(i=>`${i.key}${i.body}`).join('|');
    host.hidden=!this.items.length;
    if(signature===this.signature)return;this.signature=signature;
    $('feed-list').innerHTML=this.items.map(i=>`<li class="${i.kind}"${i.body?` data-body="${i.body}" role="button" tabindex="0" title="Go to it"`:''}><i aria-hidden="true"></i><span>${i.text}</span></li>`).join('');
  }
}
