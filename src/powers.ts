/** Creator powers: decide how fast life varies and which variations get a future. */
import type {BreedingGoal,Command,Snapshot,SteerAction} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;

const dial=(id:string,key:string,label:string,choices:[number|string,string][])=>`<div class="segmented" id="${id}" role="group" aria-label="${label}">${choices.map(([value,name])=>`<button data-${key}="${value}" aria-pressed="false">${name}</button>`).join('')}</div>`;
const GOALS:[BreedingGoal|'',string,string][]=[['bodies','Bigger bodies','Size and mixed tissues'],['movement','Movement','Muscle that does real work'],['support','Skeleton','Mineral support'],['senses','Senses & nerves','Receptors and signals'],['hunting','Hunters','Bodies that eat others'],['','Let nature decide','No breeding']];
const INTENSITY={gentle:{strength:.15,period:40},firm:{strength:.25,period:30},ruthless:{strength:.4,period:20}} as const;
type Intensity=keyof typeof INTENSITY;
const ACTS:[string,string,string][]=[['nourish','Rain of plenty','Fuel falls where life already is'],['storm','Mutation storm','8× variation for 60 cycles'],['winter','Ice age','Deep cold for 120 cycles'],['meteor','Meteor strike','Click the planet to aim'],['dying','Great dying','Half of all cells perish']];
const GOAL_NAMES:Record<string,string>={bodies:'Bigger bodies',movement:'Movement',support:'Skeleton',senses:'Senses & nerves',hunting:'Hunters'};

export const powersHTML=`<aside id="powers-panel" class="guide-panel powers-panel" aria-label="Creator powers" hidden>
<div class="drawer-heading"><span class="eyebrow">CREATOR POWERS</span><button data-close-panel class="icon-button" aria-label="Close powers">×</button></div>
<h2>Decide what<br>gets a future.</h2><p class="guide-intro">You never write a gene. You set how fast life varies, and which variations survive.</p>
<p id="powers-unavailable" class="power-note" hidden>Powers act on your main planet and its nursery. Return there to use them.</p>
<section class="power"><div class="power-head"><span class="eyebrow">BREED FOR A PURPOSE</span><strong id="goal-readout">Off</strong></div>
<div class="choice-grid" id="goal-choices" role="group" aria-label="Breeding goal">${GOALS.map(([goal,name,note])=>`<button data-goal="${goal}" aria-pressed="false"><b>${name}</b><small>${note}</small></button>`).join('')}</div>
${dial('goal-intensity','intensity','Selection strength',[['gentle','Gentle'],['firm','Firm'],['ruthless','Ruthless']])}
<p class="power-note" id="goal-note">Every few cycles, the bodies showing the least of this trait are returned to the soil. The rest inherit their room and material.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">PACE OF CHANGE</span><strong id="mutation-readout">Natural</strong></div>
${dial('mutation-choices','mutation','Mutation rate',[[0,'Still'],[.5,'Calm'],[1,'Natural'],[3,'Restless'],[8,'Wild']])}
<p class="power-note" id="mutation-note">How often a copied gene comes out different. Faster brings more new ideas, and more broken ones.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">THE STAR</span><strong id="light-readout">Natural</strong></div>
${dial('light-choices','light','Star brightness',[[.5,'Dim'],[1,'Natural'],[3,'Warm'],[6,'Bright'],[8,'Blazing']])}
<p class="power-note">Light is the only income a world has. A brighter star feeds more cells, which means more births and more chances to change.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">THE OCEANS</span><strong id="mixing-readout">Natural</strong></div>
${dial('mixing-choices','mixing','Ocean mixing',[[.5,'Still'],[1,'Natural'],[3,'Stirred'],[8,'Churning']])}
<p class="power-note">Currents carry fuel, waste and minerals to cells that cannot yet go and find them.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">CHOSEN LINEAGE</span><strong id="favour-readout">None</strong></div>
<button id="favour-selected" class="observe-button">Favour the selected body</button><button id="favour-release" class="observe-button" hidden>Release the chosen lineage</button>
<p class="power-note" id="favour-note">Click a body you like, then favour it. Its descendants glow gold and are spared while every other cell is slowly thinned out.</p></section>
<section class="power"><div class="power-head"><span class="eyebrow">BLESSINGS &amp; DISASTERS</span><strong id="act-readout"></strong></div>
<div class="choice-grid" id="act-choices">${ACTS.map(([act,name,note])=>`<button data-act="${act}"><b>${name}</b><small>${note}</small></button>`).join('')}</div>
<p class="power-note">A disaster clears room for whatever survives it. Meteor strike and Great dying can be undone from World → Undo.</p></section>
</aside>`;

export class Powers {
  private snapshot:Snapshot|null=null;private intensity:Intensity='firm';private signature='';
  constructor(private send:(c:Command)=>void,private selected:()=>number|null,private aim:(note:string,strike:(x:number,z:number)=>void)=>void,private toast:(m:string)=>void){
    $('powers-panel').addEventListener('click',e=>{
      const b=(e.target as HTMLElement).closest<HTMLButtonElement>('button');if(!b||b.disabled)return;const d=b.dataset;
      if(d.mutation!==undefined)this.steer({kind:'mutation',scale:Number(d.mutation)});
      else if(d.light!==undefined)this.steer({kind:'light',scale:Number(d.light)});
      else if(d.mixing!==undefined)this.steer({kind:'mixing',scale:Number(d.mixing)});
      else if(d.intensity!==undefined){this.intensity=d.intensity as Intensity;const goal=this.snapshot?.autonomous?.steering?.goal;if(goal)this.breed(goal);else this.render();}
      else if(d.goal!==undefined)this.breed(d.goal as BreedingGoal|'');
      else if(d.act!==undefined)this.act(d.act);
    });
    $('favour-selected').onclick=()=>{const body=this.selected();if(!body){this.toast('Click a body on the planet first, then favour it.');return;}this.steer({kind:'favour',body,pressure:.03});};
    $('favour-release').onclick=()=>this.steer({kind:'unfavour'});
  }
  steer(action:SteerAction){this.send({type:'steer',action});}
  breed(goal:BreedingGoal|''){this.steer({kind:'goal',goal,...INTENSITY[this.intensity]});}
  private act(act:string){
    if(act==='nourish')this.steer({kind:'nourish',amount:.6});
    else if(act==='storm')this.steer({kind:'storm',scale:8,duration:60});
    else if(act==='winter')this.steer({kind:'winter',depth:.25,duration:120});
    else if(act==='dying')this.steer({kind:'dying',fraction:.5});
    else if(act==='meteor'){const size=this.snapshot?.config.worldSize??64,radius=Math.min(40,Math.max(5,size*.04));this.aim('Meteor strike · click the planet to choose the impact site',(x,z)=>this.steer({kind:'meteor',x,z,radius}));}
  }
  update(s:Snapshot){this.snapshot=s;this.render();}
  private render(){
    const s=this.snapshot,st=s?.autonomous?.steering;
    const available=!!st&&!s!.assessment?.control&&!s!.assessment?.running&&(s!.config.physicalAssay??'full')==='full';
    $('powers-unavailable').hidden=available||!s;
    const signature=JSON.stringify([available,this.intensity,this.selected(),st&&[st.mutation,st.light,st.mixing,st.goal,st.strength,st.rounds,st.note,st.scored,st.bodies,st.culled,Math.ceil(st.nextRound),st.favoured,st.favourPressure,Math.ceil(st.storm),Math.ceil(st.winter),s!.stats.cells]]);
    if(signature===this.signature)return;this.signature=signature;
    document.querySelectorAll<HTMLButtonElement>('#powers-panel .power button').forEach(b=>b.disabled=!available);
    if(!st)return;
    const press=(id:string,key:string,value:string)=>{let name='Custom';document.querySelectorAll<HTMLButtonElement>(`#${id} button`).forEach(b=>{const on=b.dataset[key]===value;b.classList.toggle('active',on);b.setAttribute('aria-pressed',String(on));if(on)name=b.querySelector('b')?.textContent??b.textContent??name;});return name;};
    $('mutation-readout').textContent=`${press('mutation-choices','mutation',String(st.mutation))} · ${st.mutation}×${st.storm>0?` · storm ×${(st.mutationFactor/Math.max(st.mutation,1e-6)).toFixed(0)} for ${Math.ceil(st.storm)} more`:''}`;
    $('light-readout').textContent=`${press('light-choices','light',String(st.light))} · ${st.light}×`;
    $('mixing-readout').textContent=`${press('mixing-choices','mixing',String(st.mixing))} · ${st.mixing}×`;
    press('goal-choices','goal',st.goal);
    const strength=Object.entries(INTENSITY).find(([,v])=>Math.abs(v.strength-st.strength)<1e-3&&v.period===st.period)?.[0];
    if(st.goal&&strength)this.intensity=strength as Intensity;
    press('goal-intensity','intensity',this.intensity);
    $('goal-readout').textContent=st.goal?`${GOAL_NAMES[st.goal]} · round ${st.rounds}`:'Off';
    const wait=`Next round in ${Math.ceil(st.nextRound)} cycles.`;
    $('goal-note').textContent=!st.goal?'Every few cycles, the bodies showing the least of this trait are returned to the soil. The rest inherit their room and material.'
      :st.note==='no-variation'?`No body differs on this trait yet, so nothing was removed. Quicken the pace of change and wait for the first mutation. ${wait}`
      :st.note==='too-few'?`Too few cells to breed from. Let them recover: brighten the star or send a rain of plenty. ${wait}`
      :st.note==='recovering'?`Survivors are regrowing. ${st.culled.toLocaleString()} cells returned to the soil so far. ${wait}`
      :st.note==='selecting'?`${st.scored.toLocaleString()} of ${st.bodies.toLocaleString()} bodies show the trait. ${st.culled.toLocaleString()} cells returned to the soil so far. ${wait}`
      :`First round in ${Math.ceil(st.nextRound)} cycles.`;
    const share=s!.stats.cells?st.favoured/s!.stats.cells:0;
    $('favour-readout').textContent=st.favoured?`${st.favoured.toLocaleString()} cells · ${Math.round(share*100)}%`:'None';
    $('favour-release').hidden=!st.favoured;
    $('favour-selected').textContent=st.favoured?'Add the selected body':'Favour the selected body';
    $('favour-note').textContent=st.favoured?(st.favourPressure>0&&share<1?'Your chosen lineage glows gold. Every other cell is slowly being thinned out to make room for it.':'Your chosen lineage has inherited the world.'):'Click a body you like, then favour it. Its descendants glow gold and are spared while every other cell is slowly thinned out.';
    $('act-readout').textContent=[st.storm>0?`Storm · ${Math.ceil(st.storm)}`:'',st.winter>0?`Ice age · ${Math.ceil(st.winter)}`:''].filter(Boolean).join(' · ');
    document.querySelector<HTMLButtonElement>('[data-act="winter"]')!.disabled=!available||st.winter>0;
  }
}
