import type {Bearing,Snapshot,PhysicalBody,ProgramCell} from './sim/types';
import {cellType} from './sim/roles';

/** Size of the largest patch of joined mouth cells in a body: its biggest mouth, if it has muscle to open it. */
function largestMouth(body:PhysicalBody){
  const mouth=body.nodes.map(n=>cellType(n.role)===15),seen=new Array<boolean>(mouth.length).fill(false);
  const next:number[][]=mouth.map(()=>[]);
  for(const l of body.links)if(mouth[l.a]&&mouth[l.b]){next[l.a].push(l.b);next[l.b].push(l.a);}
  let best=0;
  for(let i=0;i<mouth.length;i++){
    if(!mouth[i]||seen[i])continue;
    let size=0;const stack=[i];seen[i]=true;
    while(stack.length){const k=stack.pop()!;size++;for(const j of next[k])if(!seen[j]){seen[j]=true;stack.push(j);}}
    best=Math.max(best,size);
  }
  return {largest:best,cells:mouth.filter(Boolean).length};
}
/** How much of a working material a cell has built, as the engine measures it. */
const built=(c:ProgramCell,action:number)=>{const m=c.material,p=m?.proteins[action]??0;return m&&p>0?p/(.015*m.biomass+p):0;};
const has=(c:ProgramCell,action:number)=>built(c,action)>.25;
/** The working parts a single cell can carry: its organs. */
const PARTS:[string,(c:ProgramCell)=>boolean][]=[
  ['Chloroplasts (light harvest)',c=>has(c,0)],['Captured light-feeder',c=>(c.state?.symbiont??0)>0],['Breathing membrane',c=>has(c,48)],['Nitrogen-fixing enzyme',c=>has(c,49)],['Wings',c=>has(c,51)],['Heat-making machinery (warm blood)',c=>has(c,52)],['Mitochondria (breathers kept inside)',c=>(c.state?.mitochondria??0)>0],['Vent catalyst',c=>has(c,37)],['Mouth',c=>has(c,34)],['Lysosomes (digestive enzymes)',c=>has(c,9)||has(c,15)],
  ['Cell wall',c=>has(c,41)],['Glass shell',c=>has(c,45)],['Holdfast',c=>has(c,46)],['Large vacuole',c=>!!c.material?.body&&c.material.body.water>c.material.biomass*.1],['Skin',c=>has(c,14)],['Spines',c=>has(c,36)],['Venom',c=>has(c,35)],['Cilia (motor)',c=>has(c,32)],['Rudder',c=>has(c,33)],['Muscle fibre',c=>has(c,4)],['Fin',c=>has(c,42)],
  ['Sensors',c=>has(c,11)||has(c,16)||has(c,24)],['Signal channels',c=>has(c,12)],['Glue',c=>has(c,2)],['Growth compass',c=>has(c,20)],['Copying machinery',c=>has(c,7)],['Fat store',c=>!!c.material&&c.material.lipid/Math.max(c.material.biomass,1e-9)>.08]];
const FATE_NAMES=['stem','fate A','fate B','fate C'];
/** Which broad kind of life a body is, by the engine's own rule. */
function kindOfLife(cells:ProgramCell[],own:number,intake:number){
  if(cells.filter(c=>has(c,41)).length*2>=cells.length&&cells.length)return intake<=0?null:own*2>=intake?['Plant-like','boxed in a cell wall, and it makes its own food from light. It cannot attack; it is food for grazers']:['Fungus-like','walled, and it absorbs or digests its food'];
  const light=cells.filter(c=>has(c,0)).length*2>=cells.length;
  if(cells.some(c=>has(c,34)||has(c,9)))return light?['A half-and-half','it harvests light and carries a mouth or enzymes, so it bites weakly. To hunt in earnest it has to give the light up']:['Animal-like','no wall and no chloroplasts: it lives by eating other life'];
  if(cells.some(c=>has(c,32)||has(c,42)))return light?['A swimming alga','it feeds on light and moves under its own power. It cannot attack']:['Animal-like','bare-skinned, with cilia or fins'];
  return light?[cells.length>1?'A colony of light-feeders':'A light-feeding microbe','soft-walled and plant-like in how it lives. It cannot attack; it is food for grazers']:cells.length>1?['A simple colony','alike cells with no wall, mouth, motor or fin']:['A free microbe','no wall, mouth, motor or fin'];
}
/** Where a body is, and how much of its weight the water is carrying. */
function carried(b:Bearing|undefined,breath:number|null){
  if(!b)return '';
  const share=Math.round(Math.max(0,b.weight)*100),light=b.sinks<-.002,heavy=b.sinks>.002;
  const where=b.depth<.05?'On dry land':b.submerged>.95?'Under water':b.submerged>.05?(b.depth<.5?'In the shallows':'At the surface'):'On wet ground';
  const load=b.submerged>.95?(light?'Its fat makes it lighter than water, so it rises.':heavy?`Its mineral makes it heavier than water: it sinks slowly, at ${share<1?'under 1':share}% of its land weight.`:'The water carries all of its weight. It hangs where it is.'):b.submerged>.05?`The water carries part of it. It bears ${share}% of its weight.`:'Nothing carries it here. It bears its full weight, and the ground grips it.';
  const air=breath===null?'':breath>.85?' It breathes freely.':breath>.4?` Oxygen is thin here: it gets ${Math.round((.5+.5*breath)*100)}% of the energy in its food.`:` There is almost no oxygen here: it gets ${Math.round((.5+.5*breath)*100)}% of the energy in its food.`;
  // Out of the water a body stands on its own footing and dry air draws its water out.
  const out=b.submerged<.5,feet=b.feet??0,held=b.load??0;
  const stand=!out||!feet?'':held>1?` <strong>Its footing is being crushed</strong>: ${feet} cell${feet===1?'':'s'} on the ground carry ${held.toFixed(1)} times what they can bear. Bone, a wall or fibre would hold it up.`:` Its ${feet} cell${feet===1?'':'s'} on the ground carry ${Math.round(held*100)}% of what they can bear.`;
  const dry=!out?'':(b.dry??0)>.5?' <strong>It is drying out.</strong> Skin or a wall would keep its water in.':(b.dry??0)>.02?' Its skin keeps most of its water in.':(b.thirst??0)<.05?' The ground under it is wet.':'';
  const thirst=out&&(b.thirst??0)>.5?' With no water it cannot build with light here.':'';
  return `<span class="eyebrow">WHERE IT LIVES</span><p><strong>${where}</strong>${b.depth>=.05?`, in water ${b.depth.toFixed(1)} deep`:''}. ${load}${air}${stand}${dry}${thirst}</p>`;
}
/** Its place in the food chain, and what is holding its growth back. */
function chain(f:{level:string;steps:number;scavenged:number;nitrogenShort:number;phosphorusShort?:number;stature?:number;bone:number;fat:number}|undefined,nerves:number){
  if(!f)return '';
  const what:Record<string,string>={Producers:'It builds itself from carbon dioxide, water, nitrogen and light, and gives off oxygen','Plant-eaters':'It is built from the producers it eats','Meat-eaters':'It is built from plant-eaters','Top predators':'It is built from other meat-eaters',Decomposers:'It is built from remains and dissolved matter, and frees the nitrogen in them'};
  const short=f.nitrogenShort>.3?` <strong>Short of nitrogen</strong>: its growth is running at ${Math.round((1-f.nitrogenShort)*100)}% of what its food allows.`:(f.phosphorusShort??0)>.3?` <strong>Short of phosphorus</strong>: its growth is running at ${Math.round((1-f.phosphorusShort!)*100)}% of what its food allows. Bone would lend it some.`:'';
  const sheath=nerves>0&&f.fat>.08?` Fat sheathes its nerves: signals cross between its cells ${(1+3*f.fat).toFixed(1)} times faster.`:'';
  // Cell size: small cells feed and breed fast for their size; big ones swallow the small and cost less to run for their size.
  const st=f.stature??1,size=st<.6?` Its cells are small, ${st.toFixed(2)} of the usual size: quick to feed and breed.`:st>1.6?` Its cells are large, ${st.toFixed(1)} times the usual size: slower to breed, able to swallow the small.`:'';
  return `<p class="body-function-note"><strong>${f.level}</strong> · level ${(f.steps+1).toFixed(1)} of the food chain. ${what[f.level]??''}.${short}${sheath}${size}</p>`;
}
/** A body's build: bone, joints, muscle, feet and stores, counted from its cells. */
function bodyPlan(cells:ProgramCell[],joints:number){
  if(cells.length<3)return '';
  const bone=cells.filter(c=>!!c.material&&c.material.mineral/Math.max(c.material.biomass+c.material.fuel+c.material.mineral+c.material.lipid,1e-9)>.025).length;
  const count=(test:(c:ProgramCell)=>boolean)=>cells.filter(test).length,muscle=count(c=>has(c,4)),fins=count(c=>has(c,42)),beat=count(c=>has(c,12)),fat=count(c=>!!c.material&&c.material.lipid/Math.max(c.material.biomass,1e-9)>.08),aimed=count(c=>has(c,20));
  const parts=[bone?`${bone} bone cell${bone===1?'':'s'}${joints?` meeting at ${joints} joint${joints===1?'':'s'}`:''}`:'',muscle?`${muscle} muscle`:'',beat?`${beat} that can set a beat`:'',fins?`${fins} fin${fins===1?'':'s'} or feet`:'',fat?`${fat} storing fat`:''].filter(Boolean);
  const moves=muscle&&fins&&beat?'It has everything it needs to row or crawl.':muscle&&!fins?'Its muscle has nothing to push with: no fins or feet.':muscle&&!beat?'Its muscle has nothing to set a beat.':'';
  return `<span class="eyebrow">BODY PLAN</span><p>${parts.length?`${parts.join(', ')}.`:'No bone, muscle, fins or fat yet: a soft body.'} ${aimed*2>=cells.length?'Its cells aim their divisions, so it grows to a shape.':'It grows as a lump: its cells divide whichever way they happen to face.'} ${moves}</p>`;
}
function insides(cells:ProgramCell[],total:number){
  if(!cells.length)return '';
  const parts=PARTS.map(([name,test])=>[name,cells.filter(test).length] as const).filter(([,n])=>n>0);
  const fates=FATE_NAMES.map((name,f)=>[name,cells.filter(c=>(c.state?.fate??0)===f)] as const).filter(([,list])=>list.length);
  const workers=cells.filter(c=>(c.state?.fate??0)>0&&!has(c,7)).length,shown=cells.length<total?` (first ${cells.length} cells)`:'';
  return `<span class="eyebrow">INSIDE ITS CELLS</span><p>A cell is a body in miniature. These are its organs${shown}:</p><div class="cell-parts">${parts.length?parts.map(([name,n])=>`<span>${name}${cells.length>1?` <b>${n}</b>`:''}</span>`).join(''):'<span>Nothing built yet</span>'}</div>${fates.length>1||fates[0]?.[0]!=='stem'?`<p class="body-function-note"><strong>${fates.length} kind${fates.length===1?'':'s'} of cell by fate:</strong> ${fates.map(([name,list])=>`${list.length} ${name}`).join(', ')}. Each fate reads a different part of the same genome.${workers?` ${workers} committed cell${workers===1?' has':'s have'} given up dividing: workers.`:''}</p>`:''}`;
}
const $=<T extends HTMLElement=HTMLElement>(id:string)=>document.getElementById(id) as T;
const descriptions:Record<string,string>={progress:'The story so far',life:'Select a body. Discover its strategy.',ecology:'What is available, eaten, and produced?',evidence:'Does this mechanism actually help?',notes:'A record of this world’s changes'};
type StudioPanel='guide'|'powers'|'shape'|'observe'|'creatures'|null;
let panel:StudioPanel=null,currentTab='progress';
export function setStudioPanel(next:StudioPanel){
  panel=next;const workspace=document.querySelector<HTMLElement>('.workspace')!;
  workspace.classList.toggle('hide-shape',next!=='shape');workspace.classList.toggle('hide-observe',next!=='observe');workspace.classList.toggle('panel-open',next!==null);
  document.querySelector<HTMLElement>('.world-panel')!.hidden=next!=='shape';document.querySelector<HTMLElement>('.life-panel')!.hidden=next!=='observe';$('evolution-guide').hidden=next!=='guide';$('powers-panel').hidden=next!=='powers';$('creatures-panel').hidden=next!=='creatures';if(next==='creatures')$('toggle-collection').classList.remove('has-news');
  for(const [id,active] of [['toggle-guide',next==='guide'],['toggle-powers',next==='powers'],['toggle-shape',next==='shape'],['toggle-observe',next==='observe'],['toggle-collection',next==='creatures']] as const){$(id).classList.toggle('active',active);$(id).setAttribute('aria-expanded',String(active));}
}
export function updateStudio(tab:string){
  currentTab=tab;$('observer-context').textContent=descriptions[tab]??'';
  $<HTMLSelectElement>('observer-view').value=['evidence','notes'].includes(tab)?tab:'';
  setStudioPanel('observe');
}
export function initStudio(openTab:(tab:string)=>void){
  $('toggle-guide').onclick=()=>setStudioPanel(panel==='guide'?null:'guide');
  $('guide-hint').onclick=()=>setStudioPanel('guide');
  $('toggle-powers').onclick=()=>setStudioPanel(panel==='powers'?null:'powers');
  $('toggle-shape').onclick=()=>setStudioPanel(panel==='shape'?null:'shape');
  $('toggle-observe').onclick=()=>{if(panel==='observe')setStudioPanel(null);else openTab(currentTab);};
  $('toggle-collection').onclick=()=>setStudioPanel(panel==='creatures'?null:'creatures');
  document.querySelectorAll<HTMLButtonElement>('[data-close-panel]').forEach(b=>b.onclick=()=>setStudioPanel(null));
  $('observer-view').onchange=()=>{const value=$<HTMLSelectElement>('observer-view').value;if(value)openTab(value);};
  $('progress-inspect').onclick=()=>$('observe').click();$('progress-resources').onclick=()=>openTab('ecology');
  const menu=$<HTMLDetailsElement>('studio-menu');
  menu.addEventListener('click',e=>{if((e.target as HTMLElement).closest('button'))menu.open=false;});
  document.addEventListener('click',e=>{if(!menu.contains(e.target as Node))menu.open=false;});
  document.addEventListener('keydown',e=>{if(e.key==='Escape'){menu.open=false;if(!document.querySelector('dialog[open]'))setStudioPanel(null);}});
  setStudioPanel(null);
}

export function describeBody(s:Snapshot){
  const host=$('body-story'),o=s.selected?.observation;host.hidden=!o;if(!o)return;
  const sources=[['Light',o.flux.photosynthesis],['Dissolved fuel',o.flux.fuel+o.flux.absorbed],['Organic remains',o.flux.detritus],['Living prey',o.flux.prey],['Vents',o.flux.chemosynthesis??0]] as const;
  const total=sources.reduce((sum,[,v])=>sum+v,0),ranked=sources.filter(([,v])=>v>0).sort((a,b)=>b[1]-a[1]);
  const work=s.selected!.body.work,functions=o.functions.reduce((n,v)=>n+Number(v>0),0),gut=o.flux.shared??0,mouth=largestMouth(s.selected!.body);
  const cells=s.selected!.programCells??[],kind=cells.length?kindOfLife(cells,o.flux.photosynthesis+(o.flux.chemosynthesis??0),total):null;
  host.innerHTML=`${kind?`<span class="eyebrow">WHAT IT IS</span><p><strong>${kind[0]}</strong>: ${kind[1]}.</p>`:''}${carried(s.selected!.bearing,cells.length?cells.reduce((sum,c)=>sum+(c.state?.breath??1),0)/cells.length:null)}<span class="eyebrow">HOW IT FEEDS</span><p>${total>0?`It lives mostly on <strong>${ranked[0][0].toLowerCase()}</strong>.`:'These cells have not fed yet.'}</p>${chain(s.selected!.feeding,cells.filter(c=>has(c,12)).length)}${total>0?`<div class="diet-bar" aria-label="Measured fuel sources">${ranked.map(([label,v])=>`<span style="flex:${v/total}" title="${label}: ${(v/total*100).toFixed(1)}%" data-diet="${label}"></span>`).join('')}</div><div class="diet-key">${ranked.map(([label,v])=>`<span>${label} <b>${Math.round(v/total*100)}%</b></span>`).join('')}</div>`:''}${mouth.cells?`<p class="body-function-note"><strong>${mouth.cells} mouth cell${mouth.cells===1?'':'s'}</strong>${mouth.largest>1?`, the largest patch of them ${mouth.largest} cells across`:''}. A patch with muscle in it opens as one mouth, and its size, not the size of the body behind it, decides what it can swallow. With no muscle each mouth cell takes only what it can alone.</p>`:''}${gut>0&&total>0?`<p class="body-function-note"><strong>It has a gut.</strong> Its mouth cells have passed ${Math.round(Math.min(1,gut/total)*100)}% of everything this body took in to cells that have no mouth.</p>`:''}<p class="body-function-note">${functions} kind${functions===1?'':'s'} of working material · ${work.toFixed(3)} spent on movement.</p><small>Totals since each living cell was born, so they shift as cells die and divide.</small>${bodyPlan(cells,s.selected!.body.joints.length)}${insides(cells,s.selected!.cells)}`;
}
