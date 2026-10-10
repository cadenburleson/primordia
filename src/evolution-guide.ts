import {STRIDE,type Snapshot} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
export type GuideAction='powers'|'nursery'|'return'|'resume'|'inspect'|'resources'|'capacity'|'collection'|'function'|'food'|'climate'|'create'|'save'|'founder'|'seasons';
export type ExperimentAim='structure'|'movement'|'survival';
export interface Advice {title:string;detail:string;why:string;action:GuideAction;label:string}

export function advise(s:Snapshot|null,aim:ExperimentAim,saved:number,declining=false):Advice {
  if(!s)return {title:'Begin with a living planet',detail:'Choose a world and plant the first cells. Everything else grows from them.',why:'Nothing evolves until there is something alive to copy itself.',action:'create',label:'Create a planet'};
  if(s.assessment?.control)return {title:'Finish the tissue test',detail:'A test is running with one mechanism switched off. Return to its checkpoint before changing anything else.',why:'A fair test needs both runs to start from the same moment.',action:'function',label:'Open the test'};
  if(!s.autonomous?.physics||s.lab||s.study||s.competition||(s.config.physicalAssay??'full')!=='full')return {title:'Return to your planet',detail:'This is a lab. Use the banner above the world to go back to the life you are growing.',why:'Life’s ascent is measured on your main planet and its nursery.',action:'function',label:'View observations'};
  if(!s.stats.cells)return {title:'Your world has fallen silent',detail:'No cells survive. Check what the planet can offer, then plant life again.',why:'Hard conditions do not create adaptations on demand. They can simply end a lineage.',action:saved?'collection':'founder',label:saved?'Choose a saved organism':'Plant a starting cell'};
  if(s.stats.cellBudgetReached){
    if((s.stats.cellBudget??s.config.capacity)>=64000)return {title:'Your world is full',detail:'This planet holds as many cells as the engine allows. Save a body you like and keep evolving its descendants in the nursery.',why:'A full world can only change by replacing what is already there. The nursery gives a lineage fresh room.',action:'inspect',label:'Find a body to keep'};
    return {title:'Your world is full',detail:'Every space is taken. Raise the cell limit for more room, or breed for a purpose so the room goes to the bodies you want.',why:'No space means no births, and no births means no new mutations.',action:'capacity',label:'Raise the cell limit'};
  }
  if(declining)return {title:'Life is losing ground',detail:'More cells are dying than being born. See what is running out before you add another pressure.',why:'One change at a time shows you what actually helped.',action:'resources',label:'See what is running out'};
  if(s.paused)return {title:'Time is standing still',detail:'Resume when you are ready. Fast-forward spends the time on generations instead of drawing.',why:'Saving, placing organisms and painting the surface all pause the world so nothing moves under your hand.',action:'resume',label:'Fast-forward'};
  if(!s.nursery&&(s.config.worldSize??64)>256&&saved===0)return {title:'Evolve faster in a nursery',detail:'Hold this planet and raise a small population in a compact world. Bring back whatever you like best.',why:'A small world runs many more generations per minute than a large one.',action:'nursery',label:'Open the nursery'};
  const a=s.autonomous;let largest=0;for(let i=0;i<s.ids.length;i++)largest=Math.max(largest,s.data[i*STRIDE+3]);
  if(aim==='movement'&&s.selected?.observation){const o=s.selected.observation,total=o.flux.fuel+o.flux.absorbed+o.flux.detritus+o.flux.prey;
    if(o.flux.photosynthesis>total)return {title:'Staying put pays, for now',detail:'This body lives on light, so it has no reason to travel. Save it, then put food somewhere it has to reach.',why:'Movement only spreads when moving feeds more offspring than sitting still.',action:'food',label:'Paint food at a distance'};
    return {title:'See what its muscle is worth',detail:'Rerun this moment with contraction switched off and compare how far the body gets and how much it eats.',why:'Water and gravity move cells too. The comparison shows what the muscle itself adds.',action:'function',label:'Test this body'};
  }
  if(largest>1&&aim!=='survival')return {title:'Look inside a body',detail:'Cells are living together. Open the largest body to see its diet and tissues, and save it before your next move.',why:'The biggest body is a good place to look. What its cells have become matters more than its size.',action:'inspect',label:'Find the largest body'};
  if(a.divisions>0)return {title:'Variation is building',detail:'Every division is a chance for something new. Let more generations pass, and save anything you like the look of.',why:'New genes arrive only through copying. More births bring more of them.',action:'inspect',label:'Look at a descendant'};
  return {title:'Give the first cells a moment',detail:'They are feeding and growing toward their first division.',why:'A cell has to gather material and energy before it can split in two.',action:'resources',label:'See the resources'};
}

const experiments={
  grow:{label:'Build bodies',mark:'01',aim:'structure',title:'Make staying together pay.',intro:'Bodies begin when daughter cells stop drifting apart. Give the ones that hold together a future.',steps:[['Breed for bigger bodies','In Powers, choose Bigger bodies. Loose cells are thinned out and bodies with mixed tissues inherit the room.','powers','Open Powers'],['Look inside the largest','Open the biggest body. Its colours show which tissues its cells have become.','inspect','Find a body'],['Keep the best','Save a body you like to your collection. You can plant copies on any planet.','collection','Open collection']],watch:'Watch Life: colonies, then multicellular bodies, then specialised tissues.'},
  move:{label:'Seek food',mark:'02',aim:'movement',title:'Make reaching food matter.',intro:'A cell fed by light has no reason to travel. Put the food somewhere else and reward the bodies that work for it.',steps:[['Breed for movement','In Powers, choose Movement. Bodies that build muscle and spend energy contracting are kept.','powers','Open Powers'],['Paint food at a distance','Leave gaps between fuel patches. Covering the whole world removes the reason to move.','food','Paint fuel'],['Test the muscle','Pick a body and rerun the same moment with contraction switched off. Compare how far it gets.','function','Test a body']],watch:'Watch the planet for red: muscle cells. Then watch Life for powered movement.'},
  defend:{label:'Survive rivals',mark:'03',aim:'survival',title:'Give armour a purpose.',intro:'Some cells learn to digest their neighbours. Skin and skeleton only matter once something is trying to eat you.',steps:[['Breed for hunters','In Powers, choose Hunters. Bodies that feed on others are kept.','powers','Open Powers'],['Bring rivals together','Plant saved bodies beside a shared patch of food and let them meet.','collection','Choose organisms'],['Keep the survivors','Save the lineages that are still dividing. Compare what they built to stay alive.','collection','Save a lineage']],watch:'Inspect shows who ate whom, what was lost, and what the defences stopped.'},
  seasons:{label:'Changing world',mark:'04',aim:'survival',title:'Change the rules of survival.',intro:'A steady world settles into one way of living. Seasons keep several strategies in play.',steps:[['Start with a long, gentle year','Half-strength seasons over 960 cycles. Leave the baseline temperature alone.','seasons','Set gentle seasons'],['Let generations live through it','Watch births and deaths through the warm and cold halves. If deaths win, soften the change.','resources','Watch resources'],['Keep more than one answer','Save distinct survivors and replant them under the same cycle.','collection','Save descendants']],watch:'Seasons change light and temperature. Genes still change only through inheritance.'}
} as const;
type ExperimentKey=keyof typeof experiments;
export const guideHTML=`<aside id="evolution-guide" class="guide-panel" aria-label="Life's ascent and guide" hidden>
<div class="drawer-heading"><span class="eyebrow">LIFE</span><button data-close-panel class="icon-button" aria-label="Close life panel">×</button></div>
<h2>From one cell<br>to everything.</h2><p class="guide-intro">What your world has built so far, and the fastest way to the next step.</p>
<article class="guide-advice"><span class="eyebrow">RIGHT NOW</span><h3 id="guide-title">Begin with a living planet</h3><p id="guide-detail">Choose the starting conditions.</p><button id="guide-action" class="primary-button">Create a planet</button><details><summary>Why?</summary><p id="guide-reason"></p></details></article>
<div id="guide-vitals" class="guide-vitals" aria-label="Life so far"></div><p id="guide-trend" class="setup-note"></p>
<div class="experiment-heading-small"><span class="eyebrow">SET A CHALLENGE</span><small>Change the world. Watch life answer.</small></div>
<div class="experiment-picker" role="group" aria-label="Challenge">${Object.entries(experiments).map(([id,e])=>`<button data-experiment="${id}" aria-pressed="${id==='grow'}" class="${id==='grow'?'active':''}"><span>${e.mark}</span>${e.label}</button>`).join('')}</div>
<div id="experiment-recipe" class="experiment-recipe"></div>
<label class="guide-aim" for="guide-aim" hidden>I want to explore<select id="guide-aim"><option value="structure">More complex bodies</option><option value="movement">Useful movement</option><option value="survival">Survival &amp; reproduction</option></select></label>
<details class="guide-course capability-guide"><summary>What this world can and cannot grow</summary><p><strong>It can</strong> Cells can invent muscle, mineral skeleton, light and touch senses, nerve-like signalling, digestion, internal fluid, gas exchange and two-parent reproduction. Bodies bend at joints and move by their own contractions.</p><p><strong>It cannot, yet</strong> Flagella, horns and image-forming eyes are not modelled. Limbs and coordinated walking are possible in principle but have not been seen to evolve here.</p></details>
<details class="guide-course"><summary>What Earth teaches us</summary><p>When predators were added to single-celled algae, some populations became multicellular within about 750 generations. <a href="https://www.nature.com/articles/s41598-019-39558-8" target="_blank" rel="noopener noreferrer">Herron et al., 2019 ↗</a></p><p>Wild fruit flies shift their gene frequencies with the seasons, year after year. Changing conditions keep alternatives alive. <a href="https://pmc.ncbi.nlm.nih.gov/articles/PMC4222749/" target="_blank" rel="noopener noreferrer">Bergland et al., 2014 ↗</a></p><p>Tiktaalik’s fins already had joints and could prop up its body. Limbs arrived through useful halfway steps. <a href="https://www.nature.com/articles/nature04637" target="_blank" rel="noopener noreferrer">Shubin et al., 2006 ↗</a></p></details>
<div class="guide-nursery"><button id="guide-nursery" class="observe-button" disabled>Open evolution nursery</button><small id="guide-nursery-note">A compact world for raising descendants quickly.</small><p id="nursery-feedback" role="status" hidden></p><button data-guide-action="save" class="observe-button">Save a checkpoint of this world</button></div></aside>`;

export class EvolutionGuide {
  private snapshot:Snapshot|null=null;private saved=0;private aim:ExperimentAim='structure';private current:Advice=advise(null,'structure',0);
  private anchor:{key:string;time:number;cells:number;births:number;deaths:number}|null=null;private declining=false;private trend='';private experiment:ExperimentKey='grow';
  constructor(private action:(action:GuideAction)=>void){
    $('guide-action').onclick=()=>this.action(this.current.action);
    $('guide-aim').onchange=()=>{this.aim=$<HTMLSelectElement>('guide-aim').value as ExperimentAim;this.render();};
    $('evolution-guide').onclick=e=>{const choice=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-experiment]');if(choice){this.experiment=choice.dataset.experiment as ExperimentKey;this.aim=experiments[this.experiment].aim;$<HTMLSelectElement>('guide-aim').value=this.aim;this.renderExperiment();this.render();return;}const button=(e.target as HTMLElement).closest<HTMLButtonElement>('[data-guide-action]');if(button)this.action(button.dataset.guideAction as GuideAction);};
    $('guide-nursery').onclick=()=>this.action(this.snapshot?.nursery?'return':'nursery');this.renderExperiment();this.render();
  }
  collectionChanged(count:number){this.saved=count;this.render();}
  update(s:Snapshot){
    const key=`${s.config.seed}/${s.nursery?'nursery':'planet'}`,t=s.stats.time;
    if(!this.anchor||this.anchor.key!==key||t<this.anchor.time){this.anchor={key,time:t,cells:s.stats.cells,births:s.stats.births,deaths:s.stats.deaths};this.declining=false;this.trend='';}
    else if(t-this.anchor.time>=10){const births=s.stats.births-this.anchor.births,deaths=s.stats.deaths-this.anchor.deaths,delta=s.stats.cells-this.anchor.cells;this.declining=delta<0&&deaths>births;this.trend=`Last ${(t-this.anchor.time).toFixed(0)} cycles: ${Math.max(0,births).toLocaleString()} births · ${Math.max(0,deaths).toLocaleString()} deaths · ${delta>=0?'+':''}${delta.toLocaleString()} living cells.`;this.anchor={key,time:t,cells:s.stats.cells,births:s.stats.births,deaths:s.stats.deaths};}
    this.snapshot=s;this.render();
  }
  feedback(message:string){$('nursery-feedback').hidden=!message;$('nursery-feedback').textContent=message;}
  private renderExperiment(){const e=experiments[this.experiment];
    document.querySelectorAll<HTMLElement>('[data-experiment]').forEach(b=>{const active=b.dataset.experiment===this.experiment;b.classList.toggle('active',active);b.setAttribute('aria-pressed',String(active));});
    $('experiment-recipe').innerHTML=`<h3>${e.title}</h3><p>${e.intro}</p><ol>${e.steps.map(([title,detail,action,label])=>`<li><strong>${title}</strong><p>${detail}</p><button data-guide-action="${action}">${label} <span>↗</span></button></li>`).join('')}</ol><div class="experiment-watch"><span class="eyebrow">WATCH FOR</span><p>${e.watch}</p></div>`;
  }
  private render(){
    this.current=advise(this.snapshot,this.aim,this.saved,this.declining);
    $('guide-title').textContent=this.current.title;$('guide-detail').textContent=this.current.detail;$('guide-reason').textContent=this.current.why;$('guide-action').textContent=this.current.label;
    $('guide-trend').textContent=this.trend;
    const s=this.snapshot;$('guide-vitals').innerHTML=s?`<div><strong>${s.stats.generation}</strong><span>generations</span></div><div><strong>${(s.autonomous?.divisions??0).toLocaleString()}</strong><span>births</span></div><div><strong>${s.stats.colonies.toLocaleString()}</strong><span>bodies</span></div>`:'';
    const available=!!s?.autonomous?.physics&&!s.assessment?.control&&!s.assessment?.running&&!s.lab&&!s.study&&!s.competition&&(s.config.physicalAssay??'full')==='full';
    $<HTMLButtonElement>('guide-nursery').disabled=!available;
    $('guide-nursery').textContent=s?.nursery?'Return to your planet':'Open the nursery';
    $('guide-nursery-note').textContent=s?.nursery?`${s.nursery.parentName} is held safely. Your collection travels with you. Returning pauses time so you can plant what you bring back.`:'A compact world with 64 founders and room for 4,096 cells. It inherits your star, oceans and pace of change. Save it before refreshing to keep this run.';
  }
}
