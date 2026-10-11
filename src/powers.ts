/** Creator powers: decide how fast life varies and which variations get a future. */
import type {BreedingGoal,Command,Snapshot,SteerAction} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;

const dial=(id:string,key:string,label:string,choices:[number|string,string][])=>`<div class="segmented" id="${id}" role="group" aria-label="${label}">${choices.map(([value,name])=>`<button data-${key}="${value}" aria-pressed="false">${name}</button>`).join('')}</div>`;
const GOALS:[BreedingGoal|'',string,string][]=[['bodies','Bigger bodies','Size and mixed tissues'],['swimming','Swimmers','Motors that get somewhere'],['movement','Muscle','Bodies that row with muscle'],['support','Skeleton','Mineral support'],['limbs','Limbs','Bone, joints and the muscle to swing them'],['reserves','Fat reserves','Stores laid down for lean times'],['senses','Senses & nerves','Receptors and signals'],['hunting','Hunters','Prunes armed bodies that catch nothing'],['defence','Defences','Skin, spines and venom']];
const INTENSITY={gentle:{strength:.15,period:40},firm:{strength:.25,period:30},ruthless:{strength:.4,period:20}} as const;
type Intensity=keyof typeof INTENSITY;
/** One dial for the pace of evolution: how often copied genes vary, and whether touching cells swap them. Survival does the selecting. */
const PUSH={natural:{mutation:1,sharing:0},fast:{mutation:2,sharing:1},fastest:{mutation:4,sharing:1}} as const;
type Push=keyof typeof PUSH;
const PUSH_NOTES:Record<Push|'custom',string>={natural:'Genes vary at their natural rate.',fast:'Copied genes vary twice as often and touching cells now and then swap one. More new ideas for survival to test.',fastest:'Copied genes vary four times as often. The most new ideas, and the most broken copies, so a world can end up smaller.',custom:'Your own mix, set below.'};
const ACTS:[string,string,string][]=[['nourish','Rain of plenty','Fuel falls where life already is'],['storm','Mutation storm','8× variation for 60 cycles'],['winter','Ice age','Deep cold for 120 cycles'],['flood','Great flood','New water falls everywhere'],['drought','Long drought','A third of the water is lost'],['meteor','Meteor strike','Click the planet to aim'],['dying','Great dying','Half of all cells perish']];
const GOAL_NAMES:Record<string,string>={bodies:'Bigger bodies',swimming:'Swimmers',movement:'Muscle',support:'Skeleton',senses:'Senses & nerves',hunting:'Hunters',defence:'Defences',limbs:'Limbs',reserves:'Fat reserves'};
export const powersHTML=`<aside id="powers-panel" class="guide-panel powers-panel" aria-label="Steer evolution" hidden>
<div class="drawer-heading"><span class="eyebrow">EVOLVE</span><button data-close-panel class="icon-button" aria-label="Close evolve">×</button></div>
<h2>Survival decides.<br>You set the pace.</h2><p class="guide-intro">Nothing here picks winners. Whatever feeds, escapes and breeds best inherits the world. You choose how fast new ideas arrive.</p>
<p id="powers-unavailable" class="power-note" hidden>These act on your main planet and its nursery. Return there to use them.</p>
<section class="power"><div class="power-head"><span class="eyebrow">SPEED OF EVOLUTION</span><strong id="push-readout"></strong></div>
${dial('push-choices','push','Speed of evolution',[['natural','Natural'],['fast','Fast'],['fastest','Fastest']])}
<p class="power-note" id="push-note"></p></section>
<section class="power"><div class="power-head"><span class="eyebrow">YOUR FAVOURITE</span><strong id="favour-readout">None</strong></div>
<button id="favour-selected" class="observe-button">Favour the selected body</button><button id="favour-release" class="observe-button" hidden>Release your favourite</button>
<p class="power-note" id="favour-note"></p></section>
<details class="setup-details evolve-tuning"><summary>Fine tuning</summary>
<section class="power"><div class="power-head"><span class="eyebrow">HOW OFTEN GENES VARY</span><strong id="mutation-readout">Natural</strong></div>
${dial('mutation-choices','mutation','Mutation rate',[[0,'Still'],[.5,'Calm'],[1,'Natural'],[2,'Lively'],[4,'Restless'],[8,'Wild']])}
<p class="power-note">Faster means more new ideas, and more broken ones.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">GENE SWAPPING</span><strong id="sharing-readout">Off</strong></div>
${dial('sharing-choices','sharing','Gene sharing',[[0,'Off'],[1,'Rare'],[4,'Common'],[8,'Rampant']])}
<p class="power-note">Touching cells pass single genes to each other. Common or more blurs species into each other.</p></section>
</details>
<details class="setup-details evolve-tuning"><summary>Selective breeding <span id="goal-readout">Off</span></summary>
<p class="power-note">An optional experiment, as a farmer breeds cattle: every few cycles the bodies showing least of one trait are removed. It overrides survival, and in tests it shrank worlds and wiped out the animals.</p>
<section class="power">
<div class="choice-grid" id="goal-choices" role="group" aria-label="Trait to breed for">${GOALS.map(([goal,name,note])=>`<button data-goal="${goal}" aria-pressed="false" title="${note}"><b>${name}</b><small>${note}</small></button>`).join('')}<button data-goal="auto" aria-pressed="false"><b>Whatever is next</b><small id="auto-note">The next step life has not reached</small></button><button data-goal="" class="wide" aria-pressed="false"><b>Off</b><small>Survival alone decides</small></button></div>
${dial('goal-intensity','intensity','Selection strength',[['gentle','Gentle'],['firm','Firm'],['ruthless','Ruthless']])}
<p class="power-note" id="goal-note"></p></section>
</details>
</aside>`;
/** The planet's own dials: its star, its interior and its oceans. */
export const planetPowersHTML=`<section class="power"><div class="power-head"><span class="eyebrow">THE STAR</span><strong id="light-readout">Natural</strong></div>
${dial('light-choices','light','Star brightness',[[.5,'Dim'],[1,'Natural'],[3,'Warm'],[6,'Bright'],[8,'Blazing']])}
<p class="power-note">Light is a world’s income: a brighter star feeds more life.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">THE AIR</span><strong id="oxygen-readout"></strong></div>
${dial('oxygen-choices','oxygen','Set the air',[[0,'No oxygen'],[.3,'Thin'],[1,'Breathable']])}
<p class="power-note">Light-feeders make oxygen; everything that breathes it gets twice the energy from its food. These set the air once: after that it is life’s to change.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">NITROGEN</span><strong id="nitrogen-readout"></strong></div>
${dial('nitrogen-choices','nitrogen','Move nitrogen',[[.02,'Lightning: fix some'],[.1,'Fix a lot'],[-.5,'Send half back to the air']])}
<p class="power-note">Every living thing is built with nitrogen. Light-feeders take it from the water, animals get it in their food, and remains hold it until something rots them. Most of a planet’s nitrogen is gas in the air that no cell can use.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">CURRENTS</span><strong id="mixing-readout">Natural</strong></div>
${dial('mixing-choices','mixing','Ocean mixing',[[.5,'Still'],[1,'Natural'],[3,'Stirred'],[8,'Churning']])}
<p class="power-note">Currents carry food to cells that cannot go and find it.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">DEEP VENTS</span><strong id="vents-readout">Natural</strong></div>
${dial('vents-choices','vents','Vent activity',[[.5,'Quiet'],[1,'Natural'],[4,'Active'],[8,'Violent']])}
<p class="power-note">Chemistry from the planet’s interior that needs no light.</p></section>`;
export const eventsHTML=`<section class="power"><div class="power-head"><span class="eyebrow">BLESSINGS &amp; DISASTERS</span><strong id="act-readout"></strong></div>
<div class="choice-grid" id="act-choices">${ACTS.map(([act,name,note])=>`<button data-act="${act}"><b>${name}</b><small>${note}</small></button>`).join('')}</div>
<p class="power-note">A disaster clears room for whatever survives it. Floods, droughts, meteors and the great dying can be undone below.</p></section>`;

export class Powers {
  private snapshot:Snapshot|null=null;private intensity:Intensity='gentle';private signature='';
  constructor(private send:(c:Command)=>void,private selected:()=>number|null,private aim:(note:string,strike:(x:number,z:number)=>void)=>void,private toast:(m:string)=>void){
    // The dials live in two panels: evolution's own and the planet's.
    document.addEventListener('click',e=>{
      const b=(e.target as HTMLElement).closest<HTMLButtonElement>('.power button');if(!b||b.disabled)return;const d=b.dataset;
      if(d.mutation!==undefined)this.steer({kind:'mutation',scale:Number(d.mutation)});
      else if(d.light!==undefined)this.steer({kind:'light',scale:Number(d.light)});
      else if(d.mixing!==undefined)this.steer({kind:'mixing',scale:Number(d.mixing)});
      else if(d.sharing!==undefined)this.steer({kind:'sharing',scale:Number(d.sharing)});
      else if(d.vents!==undefined)this.steer({kind:'vents',scale:Number(d.vents)});
      else if(d.oxygen!==undefined)this.steer({kind:'oxygen',level:Number(d.oxygen)});
      else if(d.nitrogen!==undefined)this.steer({kind:'nitrogen',change:Number(d.nitrogen)});
      else if(d.push!==undefined)this.push(d.push as Push);
      else if(d.intensity!==undefined){this.intensity=d.intensity as Intensity;const st=this.snapshot?.autonomous?.steering;if(st?.auto)this.reward('auto');else if(st?.goal)this.reward(st.goal);else this.render();}
      else if(d.goal!==undefined)this.reward(d.goal as BreedingGoal|'auto'|'');
      else if(d.act!==undefined)this.act(d.act);
    });
    $('favour-selected').onclick=()=>{const body=this.selected();if(!body){this.toast('Click a body on the planet first, then favour it.');return;}this.steer({kind:'favour',body,pressure:.03});};
    $('favour-release').onclick=()=>this.steer({kind:'unfavour'});
  }
  steer(action:SteerAction){this.send({type:'steer',action});}
  /** Reward one trait, or `auto` for whatever step life has not reached yet. Rewarding nothing ends selection. */
  reward(goal:BreedingGoal|'auto'|''){this.steer({kind:'goal',goal,...INTENSITY[this.intensity]});}
  /** Set how often genes vary and how freely they are swapped. Selection is left to survival. */
  push(level:Push){const p=PUSH[level];this.steer({kind:'sharing',scale:p.sharing});this.steer({kind:'mutation',scale:p.mutation});}
  private act(act:string){
    if(act==='nourish')this.steer({kind:'nourish',amount:.6});
    else if(act==='storm')this.steer({kind:'storm',scale:8,duration:60});
    else if(act==='winter')this.steer({kind:'winter',depth:.25,duration:120});
    else if(act==='dying')this.steer({kind:'dying',fraction:.5});
    else if(act==='flood')this.steer({kind:'water',change:.3});
    else if(act==='drought')this.steer({kind:'water',change:-.33});
    else if(act==='meteor'){const size=this.snapshot?.config.worldSize??64,radius=Math.min(40,Math.max(5,size*.04));this.aim('Meteor strike · click the planet to choose the impact site',(x,z)=>this.steer({kind:'meteor',x,z,radius}));}
  }
  update(s:Snapshot){this.snapshot=s;this.render();}
  private render(){
    const s=this.snapshot,st=s?.autonomous?.steering;
    const available=!!st&&!s!.assessment?.control&&!s!.assessment?.running&&(s!.config.physicalAssay??'full')==='full';
    $('powers-unavailable').hidden=available||!s;
    const signature=JSON.stringify([available,this.intensity,this.selected(),st&&[st.mutation,st.light,st.vents,st.mixing,st.sharing,st.goal,st.auto,Math.round((st.oxygen??0)*100),Math.round((st.cycles?.nitrogen.dissolved??0)),st.strength,st.rounds,st.note,st.scored,st.bodies,st.culled,Math.ceil(st.nextRound),st.favoured,st.favourPressure,Math.ceil(st.storm),Math.ceil(st.winter),s!.stats.cells]]);
    if(signature===this.signature)return;this.signature=signature;
    document.querySelectorAll<HTMLButtonElement>('.power button').forEach(b=>b.disabled=!available);
    if(!st)return;
    const press=(id:string,key:string,value:string)=>{let name='Custom';document.querySelectorAll<HTMLButtonElement>(`#${id} button`).forEach(b=>{const on=b.dataset[key]===value;b.classList.toggle('active',on);b.setAttribute('aria-pressed',String(on));if(on)name=b.querySelector('b')?.textContent??b.textContent??name;});return name;};
    $('mutation-readout').textContent=`${press('mutation-choices','mutation',String(st.mutation))} · ${st.mutation}×${st.storm>0?` · storm ×${(st.mutationFactor/Math.max(st.mutation,1e-6)).toFixed(0)} for ${Math.ceil(st.storm)} more`:''}`;
    $('light-readout').textContent=`${press('light-choices','light',String(st.light))} · ${st.light}×`;
    $('mixing-readout').textContent=`${press('mixing-choices','mixing',String(st.mixing))} · ${st.mixing}×`;
    $('sharing-readout').textContent=press('sharing-choices','sharing',String(st.sharing??0));
    $('vents-readout').textContent=`${press('vents-choices','vents',String(st.vents))} · ${st.vents}×`;
    $('oxygen-readout').textContent=st.oxygen===undefined?'':`${Math.round(st.oxygen*100)}% free now`;
    const n=st.cycles?.nitrogen;$('nitrogen-readout').textContent=n?`${Math.round(n.dissolved/(n.dissolved+n.living+n.remains||1)*100)}% of the usable is free`:'';
    const selecting=!!st.goal||!!st.auto;
    press('goal-choices','goal',st.auto?'auto':st.goal);
    const strength=Object.entries(INTENSITY).find(([,v])=>Math.abs(v.strength-st.strength)<1e-3&&v.period===st.period)?.[0];
    if(selecting&&strength)this.intensity=strength as Intensity;
    press('goal-intensity','intensity',this.intensity);
    // Which preset the three settings add up to, if any.
    const level=(Object.keys(PUSH) as Push[]).find(k=>st.mutation===PUSH[k].mutation&&(st.sharing??0)===PUSH[k].sharing)??'custom';
    $('push-readout').textContent=press('push-choices','push',level);$('push-note').textContent=PUSH_NOTES[level];
    const name=st.goal?GOAL_NAMES[st.goal]:'';
    $('auto-note').textContent=st.auto&&name?`Now: ${name.toLowerCase()}`:'The next step life has not reached';
    $('goal-readout').textContent=st.auto?`${name||'Starting'} · round ${st.rounds}`:name?`${name} · round ${st.rounds}`:'Off';
    const wait=`Next round in ${Math.ceil(st.nextRound)} cycles.`;
    $('goal-note').textContent=!selecting?''
      :st.note==='no-variation'?`No body differs on this trait yet, so none made way. New genes now lean toward it. ${wait}`
      :st.note==='too-few'?`Too few cells to choose among. Let them recover: brighten the star or send a rain of plenty. ${wait}`
      :st.note==='recovering'?`Survivors are regrowing. ${st.culled.toLocaleString()} cells have made way so far. ${wait}`
      :st.note==='selecting'?`${st.scored.toLocaleString()} of ${st.bodies.toLocaleString()} bodies show the trait. ${st.culled.toLocaleString()} cells have made way so far. ${wait}`
      :`First round in ${Math.ceil(st.nextRound)} cycles.`;
    const share=s!.stats.cells?st.favoured/s!.stats.cells:0;
    $('favour-readout').textContent=st.favoured?`${st.favoured.toLocaleString()} cells · ${Math.round(share*100)}%`:'None';
    $('favour-release').hidden=!st.favoured;
    $('favour-selected').textContent=st.favoured?'Add the selected body':'Favour the selected body';
    $('favour-note').textContent=st.favoured?(st.favourPressure>0&&share<1?'Your favourite’s line glows gold. Everything else is slowly thinned out to make room.':'Your favourite’s line has inherited the world.'):'Click a body you like, then favour it. Its descendants glow gold and slowly take over.';
    $('act-readout').textContent=[st.storm>0?`Storm · ${Math.ceil(st.storm)}`:'',st.winter>0?`Ice age · ${Math.ceil(st.winter)}`:''].filter(Boolean).join(' · ');
    document.querySelector<HTMLButtonElement>('[data-act="winter"]')!.disabled=!available||st.winter>0;
  }
}
