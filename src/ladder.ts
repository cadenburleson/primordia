/** Life's ascent: measured steps from single cells to complex bodies. */
import type {BreedingGoal,LadderRung,Snapshot,SteerAction} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const pct=(v:number)=>v>0&&v<.01?'<1%':`${Math.round(v*100)}%`;

interface RungCopy {name:string;blurb:string;hint:string;goal?:BreedingGoal}
export const RUNGS:Record<string,RungCopy>={
  cells:{name:'Living cells',blurb:'Single cells feed, grow and divide.',hint:'Give them light and time.'},
  colonies:{name:'Colonies',blurb:'Daughter cells stay attached instead of drifting apart.',hint:'Staying together pays where hunters take lone cells.',goal:'bodies'},
  bodies:{name:'Multicellular bodies',blurb:'Two dozen cells or more holding together as one body.',hint:'A big body is a refuge from small hunters.',goal:'bodies'},
  fates:{name:'Cell fates',blurb:'Cells in a body that switch into a lasting role and read a different part of the same genome. It is how one genome builds many kinds of cell.',hint:'Comes to bodies that last: time and variation.',goal:'bodies'},
  castes:{name:'Workers and breeders',blurb:'Bodies whose committed cells have given up dividing to do a job, while stem cells carry the line on. Your own body is split the same way.',hint:'Comes to bodies that last: time and variation.',goal:'bodies'},
  walls:{name:'Walled life',blurb:'Cells in a rigid casing. Nothing can swallow them and sisters hold together without glue, but they can neither swallow nor swim. This is the road to plant-like and fungus-like life.',hint:'Walls pay where grazers and hunters roam.',goal:'defence'},
  shapes:{name:'Shaped bodies',blurb:'Bodies whose cells carry a compass and aim each division: forward to lengthen, sideways to widen. With cell fates this is how a body gets a length, a width and limbs instead of growing as a lump.',hint:'Comes to bodies that last: time and variation.',goal:'bodies'},
  tissues:{name:'Specialised tissues',blurb:'Cells in one body take on different jobs.',hint:'Comes to bodies that last: time and variation.',goal:'bodies'},
  senses:{name:'Senses',blurb:'Cells that detect light, touch and chemistry.',hint:'Pays for anything that has to find food or flee.',goal:'senses'},
  swimming:{name:'Swimming',blurb:'Cells with a motor that pushes them through water, or crawls them over ground. A motor works only on a body’s surface, so big bodies outgrow it.',hint:'Pays where food is patchy or has to be chased.',goal:'swimming'},
  steering:{name:'Guided swimming',blurb:'Swimmers that turn toward food, light or the scent of other life.',hint:'Pays where food is patchy or has to be chased.',goal:'swimming'},
  muscle:{name:'Muscle',blurb:'Springy fibres that pull on their neighbours and snap back. Wired to excitable cells, a muscle beats.',hint:'A body too big for cilia needs muscle to move.',goal:'movement'},
  fins:{name:'Fins and feet',blurb:'Flaps and bristles that slip forward and catch the water, or the ground, on the way back. They make no push of their own: they turn a beating muscle into travel.',hint:'A body too big for cilia needs muscle and fins to move.',goal:'movement'},
  skeleton:{name:'Skeleton',blurb:'Mineral laid down as stiff support. Where three bony cells meet, the joint holds its angle, and muscle at the joint swings it.',hint:'Mineral-rich water helps. Armour and levers both use it.',goal:'support'},
  hunters:{name:'Hunters',blurb:'Bodies that get a quarter or more of their fuel by eating other life. In a food web that lasts they are few: about one body in a hundred.',hint:'They arise where prey is plentiful and soft.',goal:'hunting'},
  carnivores:{name:'Carnivores',blurb:'Hunters whose prey are mostly other hunters: a second link in the food chain.',hint:'Needs herbivores in numbers first, and time.'},
  guts:{name:'Guts',blurb:'Bodies whose mouth cells feed the cells behind them. A fed interior is what lets a body be thick, and be something other than all mouth.',hint:'Needs hunters that live as bodies. So far hunters have stayed single-celled, because their prey are mostly single cells too.'},
  complex:{name:'Complex cells',blurb:'A hunter swallowed a light-feeder and kept it alive inside. On Earth that one event made eukaryotes: every plant, animal and fungus.',hint:'Happens by chance wherever hunters feed: each swallowed light-feeder has a small chance of being kept.'},
  defences:{name:'Defences',blurb:'Skin, armour, spines and venom that make a cell hard to eat.',hint:'Pays once something is hunting.',goal:'defence'},
  nerves:{name:'Nerve nets',blurb:'Cells wired together, passing signals body-wide.',hint:'Pays in bodies that already move.',goal:'senses'},
  movement:{name:'Powered movement',blurb:'Bodies with a muscle at work and a fin to push with: they row themselves along. Unlike motors, muscle gets stronger as a body grows.',hint:'A body too big for cilia needs muscle and fins to move.',goal:'movement'},
  reflexes:{name:'Reflex arcs',blurb:'A sensing cell wired straight to a moving one: the first step toward a brain.',hint:'Pays in bodies that already sense and move.',goal:'senses'},
  organs:{name:'Organs',blurb:'Bodies running four or more tissues at once: guts, pumps, gas exchange.',hint:'Comes to big bodies that last.',goal:'bodies'},
  sex:{name:'Sexual reproduction',blurb:'Two parents mix their genes in one offspring.',hint:'Raise the pace of change and keep lineages close together.'}
};
const STAGES=['Not yet','Gene appeared','First working cells','Established'] as const;
const MARKS=['○','◔','◑','●'];
const KINDS=[{name:'Simple cells',color:'#7fb8a4'},{name:'Plant-like',color:'#4f9e4a'},{name:'Fungus-like',color:'#c9a27a'},{name:'Animal-like',color:'#ff9d3c'}];
const SPECIES_COLORS:Record<string,string>={Drifter:'#7fb8a4',Alga:'#4f9e4a',Swimmer:'#ff9d3c',Grazer:'#b77bf0',Hunter:'#d6249f',Thornback:'#8f9bb8',Rower:'#2f6be0',Courter:'#f58fc0'};
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
  if(st.mutation<4&&births>200&&st.ladder.filter(r=>r.stage>=1).length<6)return {title:'Too few new ideas',detail:'Most of the gene pool is still the four founder genes. Copies need to vary more often before selection has anything to choose.',label:'Quicken the pace of change',action:{kind:'steer',action:{kind:'mutation',scale:4},note:'Copies now vary four times as often.'}};
  return null;
}

export const ladderHUD=`<span class="eyebrow" id="hud-eyebrow">LIFE HAS REACHED</span><strong id="guide-hint-title">A waiting world</strong><span id="guide-hint-detail">Create a planet to begin.</span><span class="hud-bar" aria-hidden="true"><b id="hud-progress"></b></span>`;
export const ladderHTML=`<section class="ascent" aria-label="Life's ascent"><div class="ascent-head"><span class="eyebrow">LIFE’S ASCENT</span><span id="ascent-summary"></span></div><article id="ascent-now" class="ascent-now" hidden><span class="eyebrow">HOLDING LIFE BACK</span><h3 id="ascent-now-title"></h3><p id="ascent-now-detail"></p><button id="ascent-now-action" class="primary-button"></button></article><div id="diet" class="diet" hidden><span class="eyebrow">HOW LIFE FEEDS</span><div id="diet-bar" class="diet-track" role="img"></div><div id="diet-key" class="diet-legend"></div></div><div id="kinds" class="diet" hidden><span class="eyebrow">KINDS OF LIFE</span><div id="kinds-bar" class="diet-track" role="img"></div><div id="kinds-key" class="diet-legend"></div></div><div id="species" class="diet" hidden><span class="eyebrow">SPECIES YOU RELEASED</span><div id="species-bar" class="diet-track" role="img"></div><div id="species-key" class="diet-legend"></div></div><ol id="ascent-rungs" class="ascent-rungs"></ol><p class="setup-note">Every step is measured from living cells: which genes they carry and which materials they have actually built. Nothing is awarded.</p></section>`;
export const milestoneHTML=`<div id="milestone" class="milestone" role="status" aria-live="polite" hidden><span class="eyebrow" id="milestone-eyebrow"></span><strong id="milestone-title"></strong><span id="milestone-detail"></span></div>`;

export class Ladder {
  private world='';private seen=new Set<string>();private queue:{eyebrow:string;title:string;detail:string}[]=[];private showing=false;
  private advice:ReturnType<typeof diagnose>=null;private signature='';private shares:Record<string,string>={};
  private anchor={time:0,births:0,starved:0};private recent:{births:number;starved:number}|null=null;
  constructor(private act:(action:LadderAction)=>void){
    $('ascent-now-action').onclick=()=>{if(this.advice)this.act(this.advice.action);};
    $('ascent-rungs').onclick=e=>{const b=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-breed]');if(!b)return;const goal=b.dataset.breed as BreedingGoal;this.act({kind:'steer',action:{kind:'goal',goal,strength:.25,period:30},note:`Rewarding ${RUNGS[b.dataset.rung!].name.toLowerCase()}.`});};
  }
  update(s:Snapshot|null){
    const st=s?.autonomous?.steering;
    if(!s||!st){$('hud-eyebrow').textContent='YOUR WORLD';$('guide-hint-title').textContent=s?'Outside the main world':'A waiting world';$('guide-hint-detail').textContent=s?'Return to your planet to follow life’s ascent.':'Create a planet to begin.';$('hud-progress').style.width='0';$('ascent-rungs').innerHTML='';$('ascent-summary').textContent='';$('ascent-now').hidden=true;$('diet').hidden=true;$('kinds').hidden=true;$('species').hidden=true;this.signature='';this.shares={};return;}
    this.announce(s);
    const {era,next}=standing(st.ladder.filter(r=>Object.hasOwn(RUNGS,r.key))),extinct=!s.stats.cells;
    $('hud-eyebrow').textContent=extinct?'YOUR WORLD':'LIFE HAS REACHED';
    $('guide-hint-title').textContent=extinct?'Silence':era?RUNGS[era.key].name:'First stirrings';
    $('guide-hint-detail').textContent=extinct?'No cells survive. Restore a save or seed new life.':next?`Next: ${RUNGS[next.key].name.toLowerCase()} · ${STAGES[next.stage].toLowerCase()}`:'Every step on the ladder is established.';
    $('hud-progress').style.width=`${next?Math.round(next.progress*100):100}%`;
    const established=st.ladder.filter(r=>r.stage===3).length;
    $('ascent-summary').textContent=`${established} of ${st.ladder.length} established`;
    this.share('diet',DIETS,st.diet??[]);this.share('kinds',KINDS,st.kinds??[]);
    const released=st.species??[];this.share('species',released.map(([name])=>({name,color:SPECIES_COLORS[name]??'#9fb7a7'})),released.map(([,n])=>n));
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
      return `<li class="rung stage-${r.stage}${isNext?' next':''}"><span class="rung-mark" aria-hidden="true">${MARKS[r.stage]}</span><div><div class="rung-top"><strong>${c.name}</strong><em>${STAGES[r.stage]}</em></div>${isNext||r.stage===2?`<p>${c.blurb}</p>`:''}${r.stage<3?`<span class="rung-bar"><b style="width:${Math.round(r.progress*100)}%"></b></span>`:''}${facts?`<small>${facts}</small>`:''}${isNext&&r.key!=='cells'?`<small class="rung-hint">${c.hint}${breeding?' You are breeding for this now.':''}</small>`:''}</div></li>`;
    }).join('');
  }
  /** Share of living cells in each group: by what their body eats, or by kind of life. */
  private share(id:'diet'|'kinds'|'species',groups:{name:string;color:string}[],counts:number[]){
    const total=counts.reduce((a,b)=>a+b,0),host=$(id);host.hidden=!total;if(!total)return;
    const rows=groups.map((d,i)=>({...d,share:(counts[i]??0)/total})).filter(d=>d.share>0);
    const signature=rows.map(d=>`${d.name}${Math.round(d.share*200)}`).join();if(signature===this.shares[id])return;this.shares[id]=signature;
    $(`${id}-bar`).setAttribute('aria-label',rows.map(d=>`${d.name} ${pct(d.share)}`).join(', '));
    $(`${id}-bar`).innerHTML=rows.map(d=>`<span style="flex:${d.share};background:${d.color}" title="${d.name}: ${pct(d.share)}"></span>`).join('');
    $(`${id}-key`).innerHTML=rows.map(d=>`<span><i style="background:${d.color}"></i>${d.name} <b>${pct(d.share)}</b></span>`).join('');
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
