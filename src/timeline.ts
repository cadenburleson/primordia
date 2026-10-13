import type {Snapshot} from './sim/types';

/** Earth's history, period by period, and what a planet here has to have done to count as having reached each. */
export interface Period {key:string;name:string;span:string;earth:string;needs:string;reached:(p:Planet)=>boolean;color:string}
interface Land {plants:number;animals:number;dry:number;woody:number;flowering:number;grass:number;aloft:number;warm:number;warmAloft:number;largest:number;minds:number}
interface Planet {cells:number;ozone:number;land:Land;stage:(key:string)=>number}
const done=(p:Planet,...keys:string[])=>keys.every(k=>p.stage(k)===3);
export const PERIODS:Period[]=[
  {key:'hadean',name:'Hadean',span:'4,600–4,000 million years ago',color:'#6b5a52',earth:'The planet forms and is struck again and again. The air is thin, the seas are new, and nothing lives.',needs:'A planet.',reached:()=>true},
  {key:'archean',name:'Archean',span:'4,000–2,500',color:'#7d6a5c',earth:'The first life: single cells at vents and in the shallows, then mats of them living on light. No free oxygen, so no ozone, and the surface is bathed in ultraviolet.',needs:'Living cells that hold their numbers.',reached:p=>p.cells>0&&done(p,'cells')},
  {key:'proterozoic',name:'Proterozoic',span:'2,500–541',color:'#93826f',earth:'Light-feeders fill the air with oxygen. Ozone forms. Cells learn to breathe, swallow one another and keep what they swallow: the complex cell. With oxygen they can make the matrix that holds a body together, and at the very end come sponges and the first soft animals.',needs:'Oxygen breathing, complex cells, and ozone stopping half the ultraviolet.',reached:p=>done(p,'breathing','complex')&&p.ozone>=.5},
  {key:'cambrian',name:'Cambrian',span:'541–485',color:'#2f6f93',earth:'Animals appear almost at once: bodies with muscle, nerves, eyes, mouths and armour, hunting one another. An arms race between eater and eaten.',needs:'Animals, hunters, defences, senses and movement by muscle, all established.',reached:p=>done(p,'bodies','animals','hunters','defences','senses','movement')},
  {key:'ordovician',name:'Ordovician',span:'485–444',color:'#3a82a8',earth:'Shells, reefs and the first fish, jawless and armoured in bone. Animals eat animals. The first plants edge onto wet ground.',needs:'Skeletons, and bodies that eat other animals (a third of their prey or more), established.',reached:p=>done(p,'skeleton','carnivores')},
  {key:'silurian',name:'Silurian',span:'444–419',color:'#4a94b8',earth:'Plants with stems stand up on land, and small animals follow them out of the water. In the sea, fish get jaws.',needs:'Twenty cells of walled plant living out of the water. A mat of bacteria on a wet shore does not count.',reached:p=>p.land.plants>=20},
  {key:'devonian',name:'Devonian',span:'419–359',color:'#3f8fca',earth:'The age of fishes: backbones, fins and a wave of muscle. The first forests, of plants that branch. A fish with limbs walks out onto the mud.',needs:'Branching plants, and ten cells of animal bodies on land. An animal here is a body of four cells or more with a mouth or digestive enzymes, not walled and not living on light.',reached:p=>p.land.woody>=20&&p.land.animals>=10},
  {key:'carboniferous',name:'Carboniferous',span:'359–299',color:'#55b7d6',earth:'Swamp forests bury so much carbon that oxygen reaches a third of the air and the planet cools. Dragonflies the size of hawks. The first eggs that can be laid on dry land.',needs:'A forest (200 cells of branching plant ashore) and something on the wing.',reached:p=>p.land.woody>=200&&p.land.aloft>=3},
  {key:'permian',name:'Permian',span:'299–252',color:'#63c4e8',earth:'One great dry continent. Reptiles and the sail-backed ancestors of mammals. It ends in the worst dying there has been: volcanoes, heat and seas without oxygen kill nine species in ten.',needs:'Fifty cells of animal on land with a waterproof skin.',reached:p=>p.land.dry>=50},
  {key:'triassic',name:'Triassic',span:'252–201',color:'#6aa85a',earth:'Life recovers. The first dinosaurs and the first mammals, both small.',needs:'Ten cells that make their own heat, in the bodies of animals.',reached:p=>p.land.warm>=10},
  {key:'jurassic',name:'Jurassic',span:'201–145',color:'#86bb62',earth:'Giant dinosaurs. The first birds.',needs:'A land animal of thirty cells or more, and something warm on the wing.',reached:p=>p.land.largest>=30&&p.land.warmAloft>=3},
  {key:'cretaceous',name:'Cretaceous',span:'145–66',color:'#b9d98a',earth:'Flowers, and the insects that carry their pollen. It ends when a body from space ten kilometres across strikes.',needs:'Land plants that breed by fusing cells from two parents.',reached:p=>p.land.flowering>=20},
  {key:'paleogene',name:'Paleogene',span:'66–23',color:'#f0a23c',earth:'With the great reptiles gone, mammals grow large and take every way of living.',needs:'A hundred warm-blooded cells in the bodies of animals.',reached:p=>p.land.warm>=100},
  {key:'neogene',name:'Neogene',span:'23–2.6',color:'#f28a2e',earth:'The world cools and dries. Grass spreads, and grazers and the cats that hunt them with it. Apes walk upright.',needs:'Fifty cells of glassy, unbranched plant on land: grass.',reached:p=>p.land.grass>=50},
  {key:'quaternary',name:'Quaternary',span:'2.6–now',color:'#ee5a24',earth:'Ice comes and goes. Mammoths. People.',needs:'Warm-blooded bodies wired with fifty or more nerve junctions that learn. This is as near to a mind as the engine goes; it does not make people.',reached:p=>p.land.minds>=50},
];
/** How far up Earth's own history this planet's life has come: the last period reached with every one before it. */
export function reached(s:Snapshot):number{
  const st=s.autonomous?.steering;if(!st)return 0;
  const stages=new Map(st.ladder.map(r=>[r.key,r.stage]));
  const none:Land={plants:0,animals:0,dry:0,woody:0,flowering:0,grass:0,aloft:0,warm:0,warmAloft:0,largest:0,minds:0};
  const planet:Planet={cells:s.stats.cells,ozone:st.sky?.ozone??0,land:{...none,...(st.land??{})},stage:k=>stages.get(k)??0};
  let at=0;
  for(let i=1;i<PERIODS.length&&PERIODS[i].reached(planet);i++)at=i;
  return at;
}
export const timelineHTML=`<section class="deep-time" aria-label="Deep time"><div class="ascent-head"><span class="eyebrow">DEEP TIME</span><span id="deep-time-now"></span></div><ol id="deep-time-periods" class="deep-time-periods"></ol></section>`;
export class Timeline {
  private signature='';
  update(s:Snapshot|null){
    const now=document.getElementById('deep-time-now'),list=document.getElementById('deep-time-periods');
    if(!now||!list)return;
    if(!s?.autonomous?.steering){now.textContent='';list.innerHTML='';this.signature='';return;}
    const at=reached(s);
    if(String(at)===this.signature)return;this.signature=String(at);
    now.textContent=`This planet is in its ${PERIODS[at].name}`;
    // Newest at the top, as rock is laid down. Every string here is written above; nothing comes from a saved world.
    list.innerHTML=PERIODS.map((p,i)=>({p,i})).reverse().map(({p,i})=>`<li class="period ${i<at?'past':i===at?'now':'ahead'}" style="--period:${p.color}"><div class="period-top"><strong>${p.name}</strong><em>${p.span}</em></div>${i===at||i===at+1?`<p>${p.earth}</p>`:''}${i===at+1?`<p class="period-needs">To reach it: ${p.needs}</p>`:''}</li>`).join('');
  }
}
