import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'Autonomous cells',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,capacity:1000,origin:'seeded'};
const report=[];
for(const seed of [8042,1197,6219]){
 const w=new World(JSON.stringify({...config,seed}));w.start_autonomous();const first=JSON.parse(w.save());
 assert.equal(first.organisms.length,0);assert.equal(first.autonomous.cells.length,64);assert.equal(first.autonomous.bonds.length,0);
 const genes=first.autonomous.cells[0].genome.genes;assert.equal(genes.length,5);
 assert.ok(first.autonomous.cells.every(c=>JSON.stringify(c.genome.genes)===JSON.stringify(genes)&&c.generation===0));
 assert.ok(genes.every(g=>[0,1,2,7,48].includes(g.action)));
 let checkpoint='',peak=0,expressionChanges=0,maxGenomes=0;
 for(let sample=0;sample<30;sample++){
  w.step(40);const saved=w.save(),state=JSON.parse(saved),a=state.autonomous,m=JSON.parse(w.metadata(0));
  assert.ok(a.cells.length<=1024);assert.equal(m.stats.cells,a.cells.length);assert.equal(w.ids().length,m.stats.population);
  const index=new Set(a.cells.map(c=>c.id));assert.ok(a.bonds.every(b=>index.has(b.a)&&index.has(b.b)));
  for(const cell of a.cells){assert.ok(cell.genome.genes.length>=1&&cell.genome.genes.length<=48);assert.ok(cell.energy>=0&&Number.isFinite(cell.energy));}
  maxGenomes=Math.max(maxGenomes,m.autonomous.genomes);
  // Same inherited sequence can produce different expression in local contexts.
  const seen=new Map();for(const c of a.cells){const key=c.genome.id;const prev=seen.get(key);if(prev&&prev.some((x,i)=>Math.abs(x-c.expression[i])>.02))expressionChanges++;seen.set(key,c.expression);}
  w.load(saved);assert.equal(w.save(),saved);
  const cellData=w.cell_data(),links=w.link_data();assert.equal(cellData.length%13,0);assert.equal(links.length%6,0);for(let i=0;i<links.length;i+=6){assert.ok(links[i]>=0&&links[i]<cellData.length/13&&links[i+1]>=0&&links[i+1]<cellData.length/13);}
  if(seed===8042&&m.autonomous.peakBody>peak){checkpoint=saved;peak=m.autonomous.peakBody;}
  if(sample%10===0)console.log(JSON.stringify({seed,cycles:state.time,cells:a.cells.length,divisions:a.divisions,genomes:m.autonomous.genomes,peakBody:a.peak_body,innovations:a.innovations}));
 }
 const m=JSON.parse(w.metadata(0));const result={seed,cycles:m.stats.time,survivors:m.stats.cells,bodies:m.stats.population,...m.autonomous,recent:undefined,maxGenomes,expressionChanges};report.push(result);console.log(JSON.stringify(result));
 assert.ok(result.divisions>0&&result.pointEdits>0&&result.duplications>0&&result.deletions>0&&result.innovations>0);
 assert.ok(result.attachedDivisions>0&&result.peakBody>=2&&expressionChanges>0&&maxGenomes>1);
 const copy=new World(JSON.stringify(config));copy.load(w.save());w.step(12);copy.step(12);assert.equal(w.save(),copy.save());assert.deepEqual(w.cell_data(),copy.cell_data());
 const bad=JSON.parse(w.save());if(bad.autonomous.cells.length){bad.autonomous.cells[0].genome.genes[0].action=1000;assert.throws(()=>copy.load(JSON.stringify(bad)));}
 if(seed===8042&&checkpoint)await writeFile(new URL('../examples/autonomous-world.json',import.meta.url),checkpoint+'\n');
 w.free();copy.free();
}
await writeFile(new URL('../docs/autonomous-results.json',import.meta.url),JSON.stringify({protocol:'64 identical single cells, four supplied basic genes, fixed physical/chemical primitives and replication-error probabilities. 300 cycles, three seeds. No fixed phenotype chromosomes, assigned tissues, body-size target, forward colony thrust, fitness optimizer or automatic rescue.',results:report},null,2)+'\n');
console.log('Autonomous cellular WASM: variable inherited genomes, structural gene mutations, cell-local expression, retained division graphs, bounded physics/render arrays, valid sampled saves, exact continuation and corruption rejection passed. No organ or walking claim.');
