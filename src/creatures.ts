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
  ['ulothrix','Ulothrix','#2e8f6b','A walled thread one cell wide. Grows at its ends'],
  ['lettuce','Sea lettuce','#7ccf3a','A walled sheet one cell thick. Spreads at its edge'],
  ['seaweed','Seaweed','#5c8f3a','A walled thread that holds on to the bottom'],
  ['diatom','Diatom','#d9b45a','A light-feeder in a shell of glass, built from silica in the water'],
  ['cyanobacterium','Cyanobacterium','#3fb7a8','A bacterium: a tiny rod that lives on light'],
  ['bacillus','Bacillus','#c08a4f','A bacterium: a tiny rod that rots remains'],
  ['archaeon','Archaeon','#c98f5a','A vent microbe. Lives on the chemistry of hot rock, without light'],
  ['euglena','Euglena','#8fd15a','A long green swimmer with an eye-spot'],
  ['dinoflagellate','Dinoflagellate','#e0603a','A swimming light-feeder that makes poison'],
  ['paramecium','Paramecium','#c77bf0','A slipper-shaped swimmer that draws small cells into its mouth'],
  ['heliozoan','Heliozoan','#b9c2dd','A drifting ball of spines that swallows what touches it'],
  ['radiolarian','Radiolarian','#e6d28a','A drifting hunter with spines and a skeleton of glass'],
  ['vorticella','Vorticella','#e58fd0','A mouth on a stalk. Grips the bottom and draws food to it'],
  ['hydra','Hydra','#f0a04b','A small animal fixed to the bottom. Stings and swallows'],
  ['jelly','Jelly','#ff6f61','A small animal that swims. Its cells stay together, follow the scent of life, sting and swallow'],
  ['mat','Mat','#6f8f5a','Archean. Light-feeding bacteria that stick to the rock and to each other in the shallows, under a sheath that keeps ultraviolet out'],
  ['sponge','Sponge','#e3b23c','Proterozoic. The first animal: fixed to the bottom, every cell a mouth pumping water through, held together by matrix it needs oxygen to make'],
  ['coral','Coral','#ff8fa3','A stalk of stinging mouths on a stony base, with light-feeders kept alive inside its cells'],
  ['dickinsonia','Dickinsonia','#b8a07a','Ediacaran. A flat quilted body with no mouth, lying on the bed and digesting what is under it'],
  ['worm','Worm','#e8554e','Cambrian. The first animal that moves by muscle. Its larva swims with cilia; grown, it twitches along on bristles, stings and swallows'],
  ['trilobite','Trilobite','#8c6f4f','Cambrian. An armoured animal of the sea bed that walks by muscle, has eyes, and eats remains'],
  ['shrimp','Shrimp','#e8836b','Cambrian. A small crustacean: a jointed body in a thin shell, with limbs that row it and sweep drifting cells into the mouths along them. Bigger hunters live on it'],
  ['copepod','Copepod','#f2b38a','Cambrian. The smallest crustacean: a soft body of a few cells with filtering limbs. It breeds fast, and the young of bigger animals live on it'],
  ['scorpion','Sea scorpion','#7a4fb5','Ordovician. A large armoured hunter that swims by muscle, with spines for claws and strong venom'],
  ['fish','Fish','#4f8fd6','Silurian and Devonian. A long stiffened body that swims by muscle with fins, with eyes, nerves that learn and a store of fat'],
  ['moss','Moss','#5fa64a','Silurian. The first plant to stand on land: walled cells in a line, a waxy skin and a grip on the ground'],
  ['fungus','Fungus','#b08fc7','Silurian. Walled threads that branch over the ground, rot what lies on it and take mineral from rock. Pressed against a plant, the two trade across the wall they share'],
  ['fern','Fern','#3f8f4f','Devonian. A land plant that branches by rule, each frond a smaller copy of the one it grew from'],
  ['amphibian','Amphibian','#5aa39a','Devonian. A fish that can leave the water: skin that holds some of its water, and fins strong enough to be feet'],
  ['tree','Tree','#2f6f3a','Carboniferous. Large woody cells that branch often: a forest in the making'],
  ['dragonfly','Dragonfly','#3fc7d9','Carboniferous. Its young swim; grown, it has a dry skin, wings and muscle to beat them'],
  ['reptile','Reptile','#a3b35a','Permian. An animal that never needs the water: waterproof skin, bone and young that start with a yolk'],
  ['dinosaur','Dinosaur','#7f9f3f','Triassic and Jurassic. A reptile grown large, with heavier bone and some heat of its own'],
  ['bird','Bird','#f2c94c','Jurassic. Small, warm-blooded and winged'],
  ['flower','Flower','#f58fc0','Cretaceous. A branching land plant that breeds by fusing cells from two parents'],
  ['mammal','Mammal','#c98a5a','Paleogene. Warm-blooded, fat under the skin, more nerve than a reptile, and young that stay longest'],
  ['grass','Grass','#a8d65a','Neogene. A land plant with glass in its cells that does not branch: fast-growing and hard to chew'],
  ['ape','Ape','#b07a4a','Quaternary. A mammal with all the nerve its cells can carry'],
  ['anabaena','Anabaena','#57c7b0','A thread of bacteria that makes its own nitrogen from the air, in the dark behind its walls. It feeds the whole sea'],
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
<p class="power-note">Starter species are built from ordinary genes. A single-celled one is released as one cell. One that grows into a body is raised from one cell in a nursery and set down grown. Everything you release has to feed, grow and breed like the rest, and evolves from its first division.</p>
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
