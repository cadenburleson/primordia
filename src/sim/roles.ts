/** Cell types in the physical world, indexed by the engine's material role. */
export const CELL_TYPES=[
  {name:'Feeding',color:'#7fb8a4'},
  {name:'Skeleton',color:'#f3ecd8'},
  {name:'Muscle',color:'#e8584f'},
  {name:'Senses & nerves',color:'#4fb4ff'},
  {name:'Transport & repair',color:'#5fe0c2'},
  {name:'Digestive',color:'#b77bf0'},
  {name:'Fat reserve',color:'#f2b84b'},
  {name:'Skin',color:'#a9c76a'},
  {name:'Connective fibre',color:'#d9c29a'},
  {name:'Fluid-filled',color:'#56cfe6'},
  {name:'Reproductive',color:'#f58fc0'},
  {name:'Matrix',color:'#86b996'},
  {name:'Swimmer',color:'#ff9d3c'},
  {name:'Venom',color:'#c6f03e'},
  {name:'Spines',color:'#8f9bb8'},
  {name:'Mouth',color:'#d6249f'},
  {name:'Vent feeder',color:'#c98f5a'},
  {name:'Walled cell',color:'#4f9e4a'},
  {name:'Fin',color:'#2f6be0'},
  {name:'Glass shell',color:'#d9b45a'},
  {name:'Holdfast',color:'#9a7b55'},
  {name:'Gill or lung',color:'#ff7fb0'},
  {name:'Heart or vessel',color:'#c0392b'},
  {name:'Wing',color:'#b9e3ff'},
  {name:'Gland',color:'#e0a3ff'},
  {name:'Immune',color:'#fff2a8'}
] as const;
export const CHOSEN_COLOR='#ffd27a';
/** Display flags the engine adds to a role: 64 for the player's chosen lineage, 32 for a complex cell carrying a captured symbiont, 128 for a cell committed to a fate. */
export const cellType=(role:number)=>role&31;
export const isChosen=(role:number)=>(role&64)!==0;
export const isComplex=(role:number)=>(role&32)!==0;
export const isCommitted=(role:number)=>(role&128)!==0;
/** What the drawing marks on a cell, one bit each, as the engine packs them after a cell's rod axis. */
export const MARK={mouth:1,digesting:2,eye:4,bone:8,fin:16,muscle:32,firing:64,gamete:128,zygote:256,stem:512,midline:1024,founder:2048} as const;
