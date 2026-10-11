import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import init,{World} from '../src/wasm/primordia_engine.js';
await init({module_or_path:await readFile(new URL('../src/wasm/primordia_engine_bg.wasm',import.meta.url))});
const config={seed:8042,kind:'tidal',name:'Inherited development check',sunlight:.75,nutrients:.7,minerals:.45,temperature:.48,radiation:.18,solvent:.85,mutation:.09,initialCells:16,capacity:4096,origin:'seeded',physicalAssay:'full',founderSpread:'global',worldSize:64,worldShape:'sphere'};
const results=[];
function inspect(w){
 const m=JSON.parse(w.metadata(0));
 const p=m.autonomous.physics,planet=m.autonomous.planet;
 for(const e of [p.massError,p.mineralError,p.energyError,planet.waterError])assert.ok(Number.isFinite(e)&&Math.abs(e)<1e-6);
 const display=w.display_world();assert.ok([...display.cell_data(),...display.link_data(),...display.joint_data()].every(Number.isFinite));display.free();
 return m;
}
for(const seed of [8042,12019]){
 const w=new World(JSON.stringify({...config,seed}));w.start_physical();
 const initial=JSON.parse(w.save());assert.ok(initial.autonomous.cells.every(c=>c.genome.genes.length===4&&c.material.proteins.length===48&&c.material.proteins.slice(16).every(x=>x===0)&&!c.lifecycle.homolog));
 for(let i=0;i<8;i++){w.step(40);inspect(w);}
 const m=inspect(w);assert.equal(m.autonomous.actions.length,48);assert.ok(m.autonomous.divisions>0);
 const saved=w.save(),copy=new World(JSON.stringify(config));
 if(seed===12019)await writeFile(new URL('../examples/ordinary-development-12019.json',import.meta.url),saved+'\n');
 copy.load(saved);assert.equal(copy.save(),saved);w.step(4);copy.step(4);assert.equal(w.save(),copy.save());
 results.push({kind:'ordinary founders',seed,cycles:m.stats.time,cells:m.stats.cells,colonies:m.stats.colonies,generation:m.stats.generation,development:m.autonomous.development,regions:m.autonomous.physiology.regions});
 console.log(JSON.stringify(results.at(-1)));copy.free();w.free();
}
// Controlled gametes explicitly supply fusogen to test the cycle, not its emergence.
const assay=new World(JSON.stringify({...config,initialCells:2,worldShape:'planar',founderSpread:'local'}));assay.start_physical();
const fixture=JSON.parse(assay.save());fixture.config.name='Controlled fertilization mechanism fixture';fixture.autonomous.next_cell=5;
fixture.autonomous.cells.forEach((c,i)=>{c.id=i+3;c.parent=i+1;c.p={x:i*.5,y:5,z:0};c.v={x:0,y:0,z:0};c.lifecycle={root:i+1,depth:0,phase:'gamete',mate:0,homolog:null};c.material.proteins[19]=.08;});
// The second parent deliberately has a distinct allele to trace inheritance.
fixture.autonomous.cells[1].genome.id=2;fixture.autonomous.next_genome=3;
fixture.autonomous.cells[1].genome.genes.push({action:4,promoter:Array(28).fill(0),bias:1,strength:1});
fixture.autonomous.physical.initial_organic+=.16;fixture.autonomous.physical.initial_energy+=.48;
assay.load(JSON.stringify(fixture));
for(let i=0;i<32;i++){assay.step(4);if(inspect(assay).autonomous.development.zygotes>0)break;}
const result=inspect(assay);assert.ok(result.autonomous.development.zygotes>0,'controlled physical-contact fertilization');assert.ok(result.autonomous.development.diploidCells>0);
const save=assay.save();await writeFile(new URL('../examples/controlled-reproduction.json',import.meta.url),save+'\n');const copy=new World(JSON.stringify(config));copy.load(save);assay.step(4);copy.step(4);assert.equal(assay.save(),copy.save());
const cell=JSON.parse(save).autonomous.cells.find(c=>c.lifecycle.homolog);assert.ok(cell.lifecycle.mate>0);assert.equal(cell.lifecycle.depth,0);
assert.ok(cell.lifecycle.homolog.genes.some(g=>g.action===4));
const broken=JSON.parse(save);broken.autonomous.cells.find(c=>c.lifecycle.homolog).lifecycle.homolog.genes[0].action=48;assert.throws(()=>copy.load(JSON.stringify(broken)));
results.push({kind:'supplied mechanism fixture',cycles:result.stats.time,development:result.autonomous.development});console.log(JSON.stringify(results.at(-1)));
copy.free();
// Follow the fertilized cell through real paid division in the shared engine.
assay.step(40);const grown=inspect(assay),grownState=JSON.parse(assay.save());
assert.ok(grown.autonomous.divisions>0&&grown.autonomous.development.diploidCells>=2);
assert.ok(grownState.autonomous.cells.every(c=>c.lifecycle.homolog&&c.lifecycle.root===5&&c.lifecycle.phase==='somatic'));
assert.ok(grownState.autonomous.cells.some(c=>c.lifecycle.depth>0));
results.push({kind:'supplied zygote growth fixture',cycles:grown.stats.time,cells:grown.stats.cells,diploidCells:grown.autonomous.development.diploidCells,divisions:grown.autonomous.divisions});console.log(JSON.stringify(results.at(-1)));
assay.free();
// Old same-world snapshots preserve actual molecules and gene lists, padding zeros.
const old=JSON.parse(await readFile(new URL('../examples/natural-ecosystem-8042.json',import.meta.url),'utf8'));
const migrated=new World(JSON.stringify(config));migrated.load(JSON.stringify(old));const state=JSON.parse(migrated.save());
state.autonomous.cells.forEach((c,i)=>{assert.deepEqual(c.material.proteins.slice(0,old.autonomous.cells[i].material.proteins.length),old.autonomous.cells[i].material.proteins);assert.ok(c.material.proteins.slice(16).every(x=>x===0));assert.equal(c.genome.genes.length,old.autonomous.cells[i].genome.genes.length);assert.ok(c.genome.genes.every(g=>g.promoter.length===32));assert.equal(c.lifecycle.phase,'somatic');assert.equal(c.lifecycle.homolog,null);});
assert.equal(state.autonomous.physical.development.zygotes,0);migrated.step(4);inspect(migrated);migrated.free();
await writeFile(new URL('../docs/development-results.json',import.meta.url),JSON.stringify({protocol:'Two ordinary-founder populations, then a clearly supplied physical-contact fertilization fixture. Checks conserve modeled stores, validate buffers, import old inventories and reproduce exact saved continuation. No organ, intelligence or spontaneous sexual-reproduction outcome is required.',results},null,2)+'\n');
console.log('Development WASM integration passed. Supplied mechanism fixtures are separate from evolutionary observations.');
