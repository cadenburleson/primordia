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
  {name:'Matrix',color:'#86b996'}
] as const;
export const CHOSEN_COLOR='#ffd27a';
/** The engine adds 64 to the role of cells in the player's chosen lineage. */
export const cellType=(role:number)=>role&63;
export const isChosen=(role:number)=>(role&64)!==0;
