/** Family trees: the lines each starter kind has split into, each hung from the line it arose from. */
import type {Snapshot} from './sim/types';
/** What marks one line out from another, in the engine's order (`TRAITS` in engine/src/steering.rs). */
const TRAITS=['feeds on light','a mouth','digests living tissue','rots remains','venom','spines','a skeleton','a wall','a skin','eyes','steering','cilia','muscle','fins','wings','a holdfast','warm blood','sex','fixes nitrogen','junctions that learn'] as const;
interface Branch {traits:number;from:number|null;since:number|null;cells:number}
export interface FamilyTree {species:string;cells:number;lines:number;branches:Branch[]}
const named=(mask:number)=>TRAITS.filter((_,bit)=>mask>>bit&1);
const el=<K extends keyof HTMLElementTagNameMap>(tag:K,cls:string,text='')=>{const e=document.createElement(tag);if(cls)e.className=cls;if(text)e.textContent=text;return e;};
export const treesHTML=`<section class="family-trees" aria-label="Family trees"><div class="ascent-head"><span class="eyebrow">FAMILY TREES</span><span id="trees-summary"></span></div><div id="trees-list" class="trees-list"></div><p id="trees-note" class="trees-note" hidden>A line is every cell of one starter kind that carries the genes for the same set of adaptations. Each hangs from the line it arose from. A gene carried is not always a part built.</p></section>`;
export class Trees {
  private signature='';private open=new Set<string>();
  update(s:Snapshot|null){
    const list=document.getElementById('trees-list'),summary=document.getElementById('trees-summary'),note=document.getElementById('trees-note');
    if(!list||!summary||!note)return;
    const trees=(s?.autonomous?.steering?.trees??[]) as FamilyTree[];
    const signature=JSON.stringify(trees);
    if(signature===this.signature)return;this.signature=signature;
    list.replaceChildren();note.hidden=!trees.length;
    summary.textContent=trees.length?`${trees.length} kind${trees.length===1?'':'s'} have split`:'No kind has split yet';
    for(const tree of trees){
      // The name of a kind comes from the planet, which may have been loaded from a file: it is set as text, never as markup.
      const box=el('details','tree');box.open=this.open.has(tree.species);
      box.addEventListener('toggle',()=>{if(box.open)this.open.add(tree.species);else this.open.delete(tree.species);});
      const head=el('summary','tree-head');head.append(el('strong','',tree.species),el('em','',`${tree.cells.toLocaleString()} cells · ${tree.lines.toLocaleString()} line${tree.lines===1?'':'s'}`));
      box.append(head);
      const children=new Map<number,Branch[]>();const known=new Set(tree.branches.map(b=>b.traits));
      const roots:Branch[]=[];
      for(const b of tree.branches){if(b.from===null||!known.has(b.from))roots.push(b);else children.set(b.from,[...(children.get(b.from)??[]),b]);}
      const draw=(b:Branch,root:boolean,depth:number):HTMLLIElement=>{
        const li=el('li',`tree-line${b.cells?'':' gone'}`);const row=el('div','tree-row');
        if(root){row.append(el('span','tree-what',b.from===null?'As it began':'An early line'),el('span','tree-has',named(b.traits).join(', ')||'nothing of note'));}
        else{
          const gained=named(b.traits&~(b.from??0)),lost=named((b.from??0)&~b.traits);
          for(const g of gained)row.append(el('span','tree-gain',`+ ${g}`));
          for(const l of lost)row.append(el('span','tree-loss',`− ${l}`));
          if(!gained.length&&!lost.length)row.append(el('span','tree-what','the same set, by another road'));
        }
        row.append(el('em','tree-count',`${b.cells?`${b.cells.toLocaleString()} cells`:'died out'}${root?'':b.since===null?' · before records began':` · since cycle ${Math.round(b.since).toLocaleString()}`}`));
        li.append(row);
        const kids=(children.get(b.traits)??[]).sort((x,y)=>y.cells-x.cells);
        if(kids.length&&depth<12){const ul=el('ul','tree-branches');for(const k of kids)ul.append(draw(k,false,depth+1));li.append(ul);}
        return li;
      };
      const ul=el('ul','tree-branches tree-root');
      for(const r of roots.sort((x,y)=>Number(y.from===null)-Number(x.from===null)||y.cells-x.cells))ul.append(draw(r,true,0));
      box.append(ul);list.append(box);
    }
  }
}
