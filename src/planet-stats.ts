/** The planet at a glance: its age, its weather, and where its water is. */
import type {CyclesView,Snapshot} from './sim/types';
import {geometry} from './sim/geometry';

const pct=(v:number)=>v>0&&v<.005?'<1%':`${Math.round(v*100)}%`;
/** Water deeper than this has pooled; anything less is damp ground. */
const POOLED=.1;

/** Standing water sorted into seas and lakes: the largest body, and any nearly as large, are sea. */
export function waterBodies(s:Snapshot){
  const depth=s.autonomous?.planet?.waterDepth,g=geometry(s.config);
  if(!depth?.length||!g.sphere)return null;
  const areas=s.autonomous?.resources?.area,area=(i:number)=>areas?.[i]??g.spacing**2;
  const seen=new Uint8Array(depth.length),bodies:number[]=[];let surface=0,deepest=0;
  for(let i=0;i<depth.length;i++){
    surface+=area(i);deepest=Math.max(deepest,depth[i]);
    if(seen[i]||depth[i]<=POOLED)continue;
    // Flood outward through neighbouring wet sites; the globe wraps east to west.
    let size=0;const stack=[i];seen[i]=1;
    while(stack.length){
      const k=stack.pop()!,x=k%g.grid,z=(k-x)/g.grid;size+=area(k);
      for(const n of [z*g.grid+(x+1)%g.grid,z*g.grid+(x+g.grid-1)%g.grid,z>0?k-g.grid:-1,z<g.rows-1?k+g.grid:-1])if(n>=0&&!seen[n]&&depth[n]>POOLED){seen[n]=1;stack.push(n);}
    }
    bodies.push(size);
  }
  bodies.sort((a,b)=>b-a);
  const seas=bodies.filter((b,i)=>i===0||b>=bodies[0]*.25),lakes=bodies.slice(seas.length);
  const sum=(list:number[])=>list.reduce((a,b)=>a+b,0);
  return {seas:seas.length,sea:sum(seas)/surface,lakes:lakes.length,lake:sum(lakes)/surface,deepest};
}
function season(phase:number,strength:number){
  if(!strength)return 'No seasons';
  return phase>=.125&&phase<.375?'Summer':phase<.625&&phase>=.375?'Autumn':phase>=.625&&phase<.875?'Winter':'Spring';
}
const warmth=(t:number)=>t<.3?'freezing':t<.42?'cold':t<.55?'mild':t<.7?'warm':'hot';

const tile=(label:string,value:string,note='')=>`<div><span>${label}</span><strong>${value}</strong>${note?`<small>${note}</small>`:''}</div>`;
function cycleTiles(c:CyclesView|undefined){
  if(!c)return '';
  const carbon=c.carbon.total||1,fixed=c.nitrogen.dissolved+c.nitrogen.living+c.nitrogen.remains||1;
  return tile('CARBON DIOXIDE',`${pct(c.carbon.dioxide/carbon)} of all carbon`,`${pct(c.carbon.living/carbon)} is alive · ${pct(c.carbon.remains/carbon)} lies in remains`)
    +tile('NITROGEN',`${pct(c.nitrogen.dissolved/fixed)} free in the water`,`${pct(c.nitrogen.living/fixed)} in the living · ${pct(c.nitrogen.remains/fixed)} locked in remains`);
}
const amount=(v:number)=>v>=100?Math.round(v).toLocaleString():v>=1?v.toFixed(1):v>=.01?v.toFixed(2):v>0?'<0.01':'0';
/** Who eats whom, as a pyramid, and the planet's books: every amount counted, none made or lost. */
export function foodWebHTML(s:Snapshot){
  const c=s.autonomous?.steering?.cycles;if(!c)return '';
  const chain=c.levels.slice(0,4),rot=c.levels[4],widest=Math.max(...chain.map(l=>l.carbon),1e-9);
  const row=(l:CyclesView['levels'][number],k:number)=>`<div class="web-row" data-level="${k}"><i style="width:${l.carbon>0?Math.max(4,Math.sqrt(l.carbon/widest)*100):0}%"></i><b>${l.name}</b><span>${l.cells.toLocaleString()} cell${l.cells===1?'':'s'} · takes in ${amount(l.intake)} a cycle</span></div>`;
  const pass=(k:number)=>chain[k].intake>1e-6?`<div class="web-pass">↑ ${c.efficiency[k]>=.001?(c.efficiency[k]*100).toFixed(c.efficiency[k]<.1?1:0):'<0.1'}% passed up</div>`:'';
  const rows=[3,2,1,0].map(k=>row(chain[k],k)+(k>0?pass(k-1):'')).join('');
  const e=c.energy,inflow=e.sunlight+e.vents||1,part=(v:number,of:number)=>pct(v/(of||1));
  const n=c.nitrogen,all=n.air+n.dissolved+n.living+n.remains||1,carbon=c.carbon.total||1;
  const miss=Math.max(Math.abs(c.carbon.error)/carbon,Math.abs(n.error)/all,Math.abs(e.error)/Math.max(inflow+e.start,1));
  return `<div class="power-head"><span class="eyebrow">WHO EATS WHOM</span><strong>carbon held at each level</strong></div>
<div class="web-pyramid">${rows}${row(rot,4)}</div>
<p class="power-note">Producers build themselves from carbon dioxide, water, nitrogen and light, and give off oxygen. Everything above them is built from what it eats. On Earth each level passes up about a tenth of what it takes in; the rest is burned for energy and lost as heat.</p>
<div class="power-head"><span class="eyebrow">NOTHING MADE, NOTHING LOST</span><strong>${miss<1e-6?'the books balance':`${(miss*100).toFixed(4)}% unaccounted`}</strong></div>
<dl class="web-books"><dt>Carbon</dt><dd>${part(c.carbon.dioxide,carbon)} carbon dioxide · ${part(c.carbon.living,carbon)} alive · ${part(c.carbon.remains,carbon)} remains · ${part(c.carbon.dissolved,carbon)} dissolved food</dd>
<dt>Nitrogen</dt><dd>${part(n.air,all)} in the air · ${part(n.dissolved,all)} in the water · ${part(n.living,all)} alive · ${part(n.remains,all)} remains</dd>
<dt>Oxygen</dt><dd>${part(c.oxygen.free,c.oxygen.free+c.oxygen.bound)} free · the rest bound in water and carbon dioxide</dd>
<dt>Energy</dt><dd>${amount(e.start)} held at the start + ${amount(inflow)} in from the star and vents = ${amount(e.heat)} given off as heat + ${amount(e.work)} spent moving + ${amount(e.stored)} still held in food and life</dd></dl>`;
}
export function planetStatsHTML(s:Snapshot){
  const p=s.autonomous?.planet;if(!p)return '';
  const t=s.stats.time,period=p.seasonPeriod??240,w=waterBodies(s),st=s.autonomous?.steering,o=st?.oxygen;
  const stored=(p.water??0)+(p.snow??0)+(p.atmosphere??0)||1;
  return tile('AGE',`${Math.floor(t).toLocaleString()} cycles`,`day ${Math.floor(t/48)+1} · year ${Math.floor(t/period)+1}`)
    +tile('SEASON',season(p.season,p.seasonStrength??1),(p.seasonStrength??1)?`${Math.round(p.season*100)}% through a ${period}-cycle year`:'steady all year')
    +tile('WEATHER',p.weather,`${pct(p.cloudCover??0)} cloud`)
    +tile('TEMPERATURE',`${Math.round(p.temperature*100)} · ${warmth(p.temperature)}`,'0 frozen · 100 scalding')
    +(w?tile(w.seas===1?'OCEAN':'OCEANS',`${pct(w.sea)} of the surface`,`deepest ${w.deepest.toFixed(1)}`)+tile('LAKES',w.lakes?`${w.lakes.toLocaleString()}`:'None',w.lakes?`${pct(w.lake)} of the surface`:''):'')
    +(o!==undefined?tile('OXYGEN',`${pct(o)} free`,o<.03?'a young planet: life is slow':o<.25?`rising · life breathes at ${pct(st?.breath??0)}`:`life breathes at ${pct(st?.breath??0)}`):'')
    +cycleTiles(st?.cycles)
    +tile('CLOUDS & VAPOUR',`${pct((p.atmosphere??0)/stored)} of all water`,'rains out when the air is full')
    +tile('SNOW & ICE',`${pct((p.snow??0)/stored)} of all water`);
}
