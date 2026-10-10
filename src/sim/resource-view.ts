import type {Snapshot} from './types';
import {geometry} from './geometry';

export const resourceLabels={water:'Surface water depth',fuel:'Available fuel',detritus:'Organic detritus',mineral:'Dissolved minerals',waste:'Oxidized waste',light:'Available light'};
export type ResourceLayer=keyof typeof resourceLabels;
export const resourceDescriptions:Record<ResourceLayer,string>={
  water:'Water flows over terrain and transports dissolved material. This map shows actual surface depth, not cloud cover.',
  fuel:'Dissolved food available to cells with uptake machinery. Bright patches can sustain chemistry and division until depleted.',
  detritus:'Organic remains released into the environment. Suitable digestive machinery can reclaim this material.',
  mineral:'Dissolved building material. Mineral-binding machinery can deposit it into support; minerals alone do not make a skeleton.',
  waste:'Oxidized material produced as fuel is used. Compare this with body output and the water that moves it.',
  light:'Energy reaching each site after planetary orientation, clouds and shade. Photosynthetic machinery is needed to turn it into fuel.'
};
export const resourceColors:Record<ResourceLayer,[number,number,number]>={water:[70,170,240],fuel:[120,220,85],detritus:[218,159,81],mineral:[174,143,240],waste:[232,110,96],light:[250,224,112]};
export const resourceUnit=(kind:ResourceLayer)=>kind==='water'?'units deep':kind==='light'?'relative light':'mass / unit²';
export function resourceValues(s:Snapshot,kind:ResourceLayer){
  const r=s.autonomous?.resources,g=geometry(s.config);
  const raw=kind==='water'?s.autonomous?.planet?.waterDepth:kind==='fuel'?s.field:kind==='light'?r?.light:kind==='waste'?r?.waste:kind==='mineral'?r?.mineral:r?.detritus;
  return Array.from(raw??[],(v,i)=>kind==='light'||kind==='water'?v:v/(r?.area?.[i]??g.spacing**2));
}
export function resourceRGB(kind:ResourceLayer,value:number,max:number):[number,number,number]{
  const t=max>0?Math.log1p(Math.max(0,value))/Math.log1p(max):0,high=resourceColors[kind],low=[24,31,36];
  return high.map((v,i)=>Math.round(low[i]+(v-low[i])*t)) as [number,number,number];
}
