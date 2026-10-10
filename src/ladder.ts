/** Life's ascent: measured steps from single cells to complex bodies. */
import type {BreedingGoal,LadderRung,Snapshot,SteerAction} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const pct=(v:number)=>v>0&&v<.01?'<1%':`${Math.round(v*100)}%`;

interface RungCopy {name:string;blurb:string;hint:string;goal?:BreedingGoal}
export const RUNGS:Record<string,RungCopy>={
  cells:{name:'Living cells',blurb:'Single cells feed, grow and divide.',hint:'Give them light and time.'},
  colonies:{name:'Colonies',blurb:'Daughter cells stay attached instead of drifting apart.',hint:'Breed for bigger bodies.',goal:'bodies'},
  bodies:{name:'Multicellular bodies',blurb:'Two dozen cells or more holding together as one body.',hint:'Breed for bigger bodies.',goal:'bodies'},
  tissues:{name:'Specialised tissues',blurb:'Cells in one body take on different jobs.',hint:'Breed for bigger bodies: mixed tissues score higher.',goal:'bodies'},
  senses:{name:'Senses',blurb:'Cells that detect light, touch and chemistry.',hint:'Breed for senses.',goal:'senses'},
  swimming:{name:'Swimming',blurb:'Cells with a motor that pushes them through water, or crawls them over ground.',hint:'Breed for swimmers.',goal:'swimming'},
  steering:{name:'Guided swimming',blurb:'Swimmers that turn toward food, light or the scent of other life.',hint:'Breed for swimmers: steering scores higher.',goal:'swimming'},
  muscle:{name:'Muscle',blurb:'Contractile fibres that pull on their neighbours.',hint:'Breed for muscle. It only counts inside bodies.',goal:'movement'},
  skeleton:{name:'Skeleton',blurb:'Mineral laid down as stiff support.',hint:'Breed for skeleton. Mineral-rich worlds help.',goal:'support'},
  hunters:{name:'Hunters',blurb:'Bodies that get a quarter or more of their fuel by eating other life. Those that eat light-feeders are herbivores.',hint:'Breed for hunters. Big, mobile bodies catch far more than lone cells.',goal:'hunting'},
  carnivores:{name:'Carnivores',blurb:'Hunters whose prey are mostly other hunters: a second link in the food chain.',hint:'Needs herbivores in numbers first. Keep breeding for hunters.',goal:'hunting'},
  complex:{name:'Complex cells',blurb:'A hunter swallowed a light-feeder and kept it alive inside. On Earth that one event made eukaryotes: every plant, animal and fungus.',hint:'Breed for hunters. Each swallowed light-feeder has a small chance of being kept.',goal:'hunting'},
  defences:{name:'Defences',blurb:'Skin, armour, spines and venom that make a cell hard to eat.',hint:'Breed for defences.',goal:'defence'},
  nerves:{name:'Nerve nets',blurb:'Cells wired together, passing signals body-wide.',hint:'Breed for senses once bodies exist.',goal:'senses'},
  movement:{name:'Powered movement',blurb:'Bodies that spend energy to move themselves.',hint:'Breed for swimmers or muscle.',goal:'swimming'},
  reflexes:{name:'Reflex arcs',blurb:'A sensing cell wired straight to a moving one: the first step toward a brain.',hint:'Breed for senses in bodies that already move.',goal:'senses'},
  organs:{name:'Organs',blurb:'Bodies running four or more tissues at once: guts, pumps, gas exchange.',hint:'Breed for bigger bodies.',goal:'bodies'},
  sex:{name:'Sexual reproduction',blurb:'Two parents mix their genes in one offspring.',hint:'Raise the pace of change and keep lineages close together.'}
};
const STAGES=['Not yet','Gene appeared','First working cells','Established'] as const;
const MARKS=['○','◔','◑','●'];
const DIETS=[{name:'Light feeders',color:'#e8cf6a'},{name:'Soup feeders',color:'#6fb8a8'},{name:'Decomposers',color:'#a08a6c'},{name:'Vent feeders',color:'#c98f5a'},{name:'Herbivores',color:'#8fd15a'},{name:'Omnivores',color:'#f08a4b'},{name:'Carnivores',color:'#d6249f'}];

export type LadderAction={kind:'steer';action:SteerAction;note:string}|{kind:'powers'};

/** Highest established step, and the first step still to be reached. */
export function standing(ladder:LadderRung[]){
  // Read as a ladder: the era is the last step of the unbroken established run.
  const gap=ladder.findIndex(r=>r.stage<3),reached=gap<0?ladder.length:gap;
  return {era:reached>0?ladder[reached-1]:null,next:gap<0?null:ladder[gap]};
}
/** What is holding life back right now, read from measured deaths and vitals. */
export function diagnose(s:Snapshot,recent:{births:number;starved:number}|null=null):{title:string;detail:string;label:string;action:LadderAction}|null{
  const st=s.autonomous?.steering;if(!st||!s.stats.cells)return null;
  const births=s.stats.births;
  // More cells starving than being born, over the recent window.
  if(recent&&recent.starved>8&&recent.starved>recent.births){
    const lost=`${recent.starved.toLocaleString()} cells starved while ${recent.births.toLocaleString()} were born.`;
    if(st.light<6)return {title:'They are starving',detail:`${lost} The starting fuel is being eaten faster than this star can replace it.`,label:'Brighten the star',action:{kind:'steer',action:{kind:'light',scale:6},note:'The star brightens. Light-fed cells can now pay their way.'}};
    if(st.mixing<3)return {title:'Food lies out of reach',detail:`${lost} Fuel sits elsewhere on the planet and nothing here can go and get it yet. Stronger currents bring that reserve within reach; it will not last forever.`,label:'Stir the oceans',action:{kind:'steer',action:{kind:'mixing',scale:3},note:'Stronger currents carry fuel and waste across the planet.'}};
    return {title:'A hungry season',detail:`${lost} The star is bright and the oceans are stirred, so this is the world finding its balance. A gift of fuel buys them time.`,label:'Send a rain of plenty',action:{kind:'steer',action:{kind:'nourish',amount:.6},note:'Fuel rains down where life already is.'}};
  }
  if(st.mutation<3&&births>200&&st.ladder.filter(r=>r.stage>=1).length<6)return {title:'Too few new ideas',detail:'Most of the gene pool is still the four founder genes. Copies need to vary more often before selection has anything to choose.',label:'Quicken the pace of change',action:{kind:'steer',action:{kind:'mutation',scale:3},note:'Copies now vary three times as often.'}};
  return null;
}

export const ladderHUD=`<span class="eyebrow" id="hud-eyebrow">LIFE HAS REACHED</span><strong id="guide-hint-title">A waiting world</strong><span id="guide-hint-detail">Create a planet to begin.</span><span class="hud-bar" aria-hidden="true"><b id="hud-progress"></b></span>`;
export const ladderHTML=`<section class="ascent" aria-label="Life's ascent"><div class="ascent-head"><span class="eyebrow">LIFE’S ASCENT</span><span id="ascent-summary"></span></div><article id="ascent-now" class="ascent-now" hidden><span class="eyebrow">HOLDING LIFE BACK</span><h3 id="ascent-now-title"></h3><p id="ascent-now-detail"></p><button id="ascent-now-action" class="primary-button"></button></article><div id="diet" class="diet" hidden><span class="eyebrow">HOW LIFE FEEDS</span><div id="diet-bar" class="diet-track" role="img"></div><div id="diet-key" class="diet-legend"></div></div><ol id="ascent-rungs" class="ascent-rungs"></ol><p class="setup-note">Every step is measured from living cells: which genes they carry and which materials they have actually built. Nothing is awarded.</p></section>`;
export const milestoneHTML=`<div id="milestone" class="milestone" role="status" aria-live="polite" hidden><span class="eyebrow" id="milestone-eyebrow"></span><strong id="milestone-title"></strong><span id="milestone-detail"></span></div>`;

export class Ladder {
  private world='';private seen=new Set<string>();private queue:{eyebrow:string;title:string;detail:string}[]=[];private showing=false;
  private advice:ReturnType<typeof diagnose>=null;private signature='';private dietSignature='';
  private anchor={time:0,births:0,starved:0};private recent:{births:number;starved:number}|null=null;
  constructor(private act:(action:LadderAction)=>void){
    $('ascent-now-action').onclick=()=>{if(this.advice)this.act(this.advice.action);};
    $('ascent-rungs').onclick=e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-breed]');if(!b)return;const goal=b.dataset.breed as BreedingGoal;this.act({kind:'steer',action:{kind:'goal',goal,strength:.25,period:30},note:`Breeding for ${RUNGS[b.dataset.rung!].name.toLowerCase()}.`});};
  }
  update(s:Snapshot|null){
    const st=s?.autonomous?.steering;
    if(!s||!st){$('hud-eyebrow').textContent='YOUR WORLD';$('guide-hint-title').textContent=s?'Outside the main world':'A waiting world';$('guide-hint-detail').textContent=s?'Return to your planet to follow life’s ascent.':'Create a planet to begin.';$('hud-progress').style.width='0';$('ascent-rungs').innerHTML='';$('ascent-summary').textContent='';$('ascent-now').hidden=true;$('diet').hidden=true;this.signature='';this.dietSignature='';return;}
    this.announce(s);
    const {era,next}=standing(st.ladder.filter(r=>Object.hasOwn(RUNGS,r.key))),extinct=!s.stats.cells;
    $('hud-eyebrow').textContent=extinct?'YOUR WORLD':'LIFE HAS REACHED';
    $('guide-hint-title').textContent=extinct?'Silence':era?RUNGS[era.key].name:'First stirrings';
    $('guide-hint-detail').textContent=extinct?'No cells survive. Restore a save or seed new life.':next?`Next: ${RUNGS[next.key].name.toLowerCase()} · ${STAGES[next.stage].toLowerCase()}`:'Every step on the ladder is established.';
    $('hud-progress').style.width=`${next?Math.round(next.progress*100):100}%`;
    const established=st.ladder.filter(r=>r.stage===3).length;
    $('ascent-summary').textContent=`${established} of ${st.ladder.length} established`;
    this.feeding(st.diet??[]);
    // Compare births with starvation over a rolling window of at least 30 cycles.
    const t=s.stats.time,now={time:t,births:s.stats.births,starved:st.losses[0]};
    if(t<this.anchor.time||this.anchor.time===0&&this.anchor.births===0){this.anchor=now;this.recent=null;}
    else if(t-this.anchor.time>=30){this.recent={births:now.births-this.anchor.births,starved:now.starved-this.anchor.starved};this.anchor=now;}
    this.advice=extinct?null:diagnose(s,this.recent);
    $('ascent-now').hidden=!this.advice;
    if(this.advice){$('ascent-now-title').textContent=this.advice.title;$('ascent-now-detail').textContent=this.advice.detail;$('ascent-now-action').textContent=this.advice.label;}
    // Rebuild only when something a reader can see has changed.
    const signature=st.ladder.map(r=>`${r.stage}${Math.round(r.progress*50)}${Math.round(r.genes*100)}${Math.round(r.built*100)}`).join('|')+st.goal+(next?.key??'');
    if(signature===this.signature)return;this.signature=signature;
    $('ascent-rungs').innerHTML=st.ladder.filter(r=>Object.hasOwn(RUNGS,r.key)).map(r=>{
      const c=RUNGS[r.key],isNext=r.key===next?.key,first=r.first[Math.max(0,r.stage-1)];
      const facts=[r.genes>0&&r.key!=='cells'&&r.key!=='tissues'?`gene in ${pct(r.genes)} of cells`:'',r.built>0&&r.stage<3?`${pct(Math.min(1,r.progress))} of the way`:'',r.stage>0&&first!=null?`since cycle ${Math.round(first).toLocaleString()}`:''].filter(Boolean).join(' · ');
      const breeding=c.goal&&st.goal===c.goal;
      return `<li class="rung stage-${r.stage}${isNext?' next':''}"><span class="rung-mark" aria-hidden="true">${MARKS[r.stage]}</span><div><div class="rung-top"><strong>${c.name}</strong><em>${STAGES[r.stage]}</em></div>${isNext||r.stage===2?`<p>${c.blurb}</p>`:''}${r.stage<3?`<span class="rung-bar"><b style="width:${Math.round(r.progress*100)}%"></b></span>`:''}${facts?`<small>${facts}</small>`:''}${isNext&&r.key!=='cells'?(c.goal?`<button data-breed="${c.goal}" data-rung="${r.key}" ${breeding?'disabled':''}>${breeding?'Breeding for this now':c.hint.replace(/\.$/,'')} <span>↗</span></button>`:`<small class="rung-hint">${c.hint}</small>`):''}</div></li>`;
    }).join('');
  }
  /** Share of living cells by what their body mostly eats. */
  private feeding(diet:number[]){
    const total=diet.reduce((a,b)=>a+b,0),host=$('diet');host.hidden=!total;if(!total)return;
    const rows=DIETS.map((d,i)=>({...d,share:(diet[i]??0)/total})).filter(d=>d.share>0);
    const signature=rows.map(d=>`${d.name}${Math.round(d.share*200)}`).join();if(signature===this.dietSignature)return;this.dietSignature=signature;
    $('diet-bar').setAttribute('aria-label',rows.map(d=>`${d.name} ${pct(d.share)}`).join(', '));
    $('diet-bar').innerHTML=rows.map(d=>`<span style="flex:${d.share};background:${d.color}" title="${d.name}: ${pct(d.share)}"></span>`).join('');
    $('diet-key').innerHTML=rows.map(d=>`<span><i style="background:${d.color}"></i>${d.name} <b>${pct(d.share)}</b></span>`).join('');
  }
  /** Celebrate each step the first time this world reaches it. */
  private announce(s:Snapshot){
    const st=s.autonomous!.steering!,world=`${s.config.seed}/${s.config.name}/${s.nursery?'nursery':'planet'}`;
    const key=(m:{key:string;stage:number})=>`${m.key}:${m.stage}`;
    if(world!==this.world||s.stats.time<4){this.world=world;this.seen=new Set(st.milestones.map(key));this.anchor={time:0,births:0,starved:0};this.recent=null;return;}
    for(const m of st.milestones){
      if(this.seen.has(key(m)))continue;this.seen.add(key(m));
      if(m.key==='cells'||!Object.hasOwn(RUNGS,m.key))continue;
      const name=RUNGS[m.key].name;
      this.queue.push({eyebrow:m.stage===3?'A NEW AGE':m.stage===2?'NEW IN YOUR WORLD':'A NEW POSSIBILITY',title:m.stage===3?`${name} established`:m.stage===2?`${name}: first working cells`:`${name}: the gene has appeared`,detail:`Cycle ${Math.round(m.time).toLocaleString()} · generation ${m.generation}`});
    }
    // A long fast-forward can cross many steps at once; keep the latest few.
    if(this.queue.length>4)this.queue.splice(0,this.queue.length-4);
    this.next();
  }
  private next(){
    if(this.showing||!this.queue.length)return;
    const m=this.queue.shift()!,el=$('milestone');this.showing=true;
    $('milestone-eyebrow').textContent=m.eyebrow;$('milestone-title').textContent=m.title;$('milestone-detail').textContent=m.detail;
    el.hidden=false;el.classList.remove('show');void el.offsetWidth;el.classList.add('show');
    setTimeout(()=>{el.classList.remove('show');setTimeout(()=>{el.hidden=true;this.showing=false;this.next();},350);},4200);
  }
}
