import {PRESETS,type WorldConfig} from './types';

/** Holds both worlds; the worker simulates only the currently active one. */
export class NurserySession {
  parent:string|null=null;
  parentName='';
  private habitat:string|null=null;
  get active(){return this.parent!==null;}
  get savedHabitat(){return this.habitat;}
  begin(parent:string,name:string){if(this.active)throw new Error('Return to your planet before opening another nursery.');this.parent=parent;this.parentName=name;}
  finish(habitat:string){if(!this.parent)throw new Error('No planet is held.');this.habitat=habitat;this.parent=null;this.parentName='';}
  reset(){this.parent=null;this.parentName='';this.habitat=null;}
}
export function nurseryConfig(parent:WorldConfig):WorldConfig {
  return {...PRESETS.tidal,name:'Evolution nursery',seed:(parent.seed^0x6e757273)>>>0,worldSize:128,worldShape:'sphere',initialCells:64,capacity:4096,founderSpread:'local',resourceLayout:'patchy',physicalAssay:'full'};
}
