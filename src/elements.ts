/** The periodic table: every element, the state it is in at a chosen temperature, and where the planet's stock of the ones this world tracks is. */
import {ELEMENT_ROWS,type ElementRow} from './elements-data';
import type {Snapshot} from './sim/types';
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;

/** What living things use each element for. An element not listed has no established role in life. */
const ROLE:Record<string,string>={
  H:'In water and in every molecule life builds',C:'The backbone of every molecule life builds',N:'In protein and in DNA',O:'In water; breathed to burn food',
  P:'In DNA, in the energy carrier every cell runs on, in cell membranes and in bone',S:'In two of the building blocks of protein',
  Na:'Nerve signals and the balance of fluids',K:'Nerve signals; the main charged atom inside a cell',Ca:'Bone and shell; makes muscle contract',Mg:'At the centre of chlorophyll',Cl:'The balance of fluids; stomach acid',
  Fe:'Carries oxygen in blood',Zn:'Works in hundreds of enzymes',Cu:'In enzymes that handle oxygen',I:'Thyroid hormones',Se:'Protective enzymes',Mn:'In the enzyme that splits water in photosynthesis',Mo:'In the enzyme that fixes nitrogen from the air',Co:'Vitamin B12',Si:'The glass shells of diatoms'};
/** The eleven elements that make up almost all of a body; the rest of `ROLE` are needed only in traces. */
const BULK=new Set(['H','C','N','O','P','S','Na','Mg','K','Ca','Cl']);
/** Planet temperature (0 frozen, 1 scalding) in degrees Celsius. Water freezes at 0.3; "mild" runs from about 13 to 27 degrees. */
export const celsius=(t:number)=>(t-.3)*108;
type State='solid'|'liquid'|'gas'|'unknown';
/** A pure element at one atmosphere. Where no boiling point is known it is taken to stay liquid. */
function stateAt(e:ElementRow,kelvin:number):State{const [,,,,melt,boil]=e;return melt===null?'unknown':kelvin<melt?'solid':boil===null||kelvin<boil?'liquid':'gas';}
const degrees=(kelvin:number)=>`${Math.round(kelvin-273.15).toLocaleString()} °C`;
/** The slider is cubic, so that the temperatures a planet can have take up a good part of its travel. */
const HOTTEST=6000,toKelvin=(v:number)=>HOTTEST*(v/1000)**3,toSlider=(k:number)=>1000*Math.cbrt(k/HOTTEST);
const pct=(v:number)=>v>0&&v<.005?'<1%':`${Math.round(v*100)}%`;

export const elementsHTML=`<dialog id="elements-dialog" aria-labelledby="elements-title"><button id="close-elements" class="icon-button" aria-label="Close the periodic table">×</button>
<span class="eyebrow">THE ELEMENTS</span><h2 id="elements-title">What a planet is made of.</h2>
<div class="elements-heat"><label for="elements-heat">Temperature</label><input id="elements-heat" type="range" min="0" max="1000" step="1" aria-label="Temperature"><strong id="elements-heat-readout"></strong><button id="elements-heat-reset" class="text-button">Your planet now</button></div>
<div id="elements-chart" class="elements-chart" role="group" aria-label="Periodic table"></div>
<div class="elements-key"><span class="el solid"></span>Solid<span class="el liquid"></span>Liquid<span class="el gas"></span>Gas<span class="el unknown"></span>Unknown<span class="el solid life"></span>Life uses it<span class="el solid tracked"></span>Tracked on your planet</div>
<div id="element-detail" class="element-detail" aria-live="polite"></div>
<p class="power-note">Drag the temperature to watch each element melt and boil. On a planet almost none of them is found pure: carbon travels as carbon dioxide, hydrogen and oxygen as water, nitrogen as a gas of paired atoms. The states shown are for the pure element at the pressure of Earth’s air. Data: PubChem.</p></dialog>`;

type Pools={symbol:string;note:string;air:number;water:number;ground:number;life:number};
export class ElementsTable {
  private kelvin=288;private following=true;private chosen=6;private planet=288;private pools=new Map<string,Pools>();private signature='';
  constructor(){
    $('elements-chart').innerHTML=ELEMENT_ROWS.map(([z,symbol,name,,,,,row,column])=>`<button class="el" data-z="${z}" style="grid-row:${row};grid-column:${column}" title="${name}"><i>${z}</i><b>${symbol}</b></button>`).join('');
    $('elements-chart').onclick=e=>{const b=(e.target as HTMLElement).closest<HTMLElement>('[data-z]');if(b){this.chosen=Number(b.dataset.z);this.render();}};
    $<HTMLInputElement>('elements-heat').oninput=e=>{this.following=false;this.kelvin=toKelvin(Number((e.target as HTMLInputElement).value));this.render();};
    $('elements-heat-reset').onclick=()=>{this.following=true;this.render();};
    $('close-elements').onclick=()=>$<HTMLDialogElement>('elements-dialog').close();
  }
  open(){this.following=true;this.render();$<HTMLDialogElement>('elements-dialog').showModal();}
  update(s:Snapshot){
    const p=s.autonomous?.planet,list=(s.autonomous?.steering?.cycles?.elements??[]) as Pools[];
    if(p)this.planet=celsius(p.temperature)+273.15;
    const signature=JSON.stringify([Math.round(this.planet),list.map(e=>[e.symbol,e.air,e.water,e.ground,e.life].map(v=>typeof v==='number'?Math.round(v):v))]);
    if(signature===this.signature)return;this.signature=signature;
    this.pools=new Map(list.map(e=>[e.symbol,e]));
    if($<HTMLDialogElement>('elements-dialog').open)this.render();
  }
  private render(){
    // Until the slider is moved, the chart shows the planet as it is now.
    if(this.following)this.kelvin=this.planet;
    $<HTMLInputElement>('elements-heat').value=String(toSlider(this.kelvin));
    $('elements-heat-readout').textContent=`${degrees(this.kelvin)}${this.following?' · your planet now':''}`;
    document.querySelectorAll<HTMLElement>('#elements-chart [data-z]').forEach(b=>{
      const e=ELEMENT_ROWS[Number(b.dataset.z)-1];
      b.className=`el ${stateAt(e,this.kelvin)}${ROLE[e[1]]?' life':''}${this.pools.has(e[1])?' tracked':''}${e[0]===this.chosen?' chosen':''}`;
    });
    const e=ELEMENT_ROWS[this.chosen-1],[z,symbol,name,mass,melt,boil,family]=e,now=stateAt(e,this.kelvin),held=this.pools.get(symbol);
    const points=melt===null?'Too short-lived to have been melted.':`Melts at ${degrees(melt)}${boil===null?'':` · boils at ${degrees(boil)}`}.`;
    const life=ROLE[symbol]?`<p><strong>${BULK.has(symbol)?'One of the eleven elements a body is mostly made of':'Life needs a trace of it'}.</strong> ${ROLE[symbol]}.</p>`:'<p>Life has no known use for it.</p>';
    const total=held?held.air+held.water+held.ground+held.life||1:1;
    const where=held?`<div class="element-pools" aria-label="Where it is on your planet">${([['air','In the air'],['water','In the water'],['ground','In the ground and ice'],['life','In the living']] as const).map(([k,label])=>`<div><span>${label}</span><i><b style="width:${held[k]/total*100}%"></b></i><strong>${pct(held[k]/total)}</strong></div>`).join('')}</div><p class="power-note">${held.note}</p>`:'<p class="power-note">Not tracked on your planet.</p>';
    $('element-detail').innerHTML=`<div class="element-card el ${now}"><i>${z}</i><b>${symbol}</b></div><div><h3>${name}</h3><p>${family} · atomic mass ${mass}. ${points} ${now==='unknown'?'':`At ${degrees(this.kelvin)} the pure element is a <strong>${now}</strong>.`}</p>${life}${where}</div>`;
  }
}
