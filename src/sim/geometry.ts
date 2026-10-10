import { Vector3 } from 'three';
import { noise } from './random';
import type { WorldConfig } from './types';
export function geometry(c:WorldConfig){
 const size=c.worldSize??64,grid=Math.min(160,Math.round(size/1.6)),sphere=c.worldShape==='sphere'&&c.kind!=='lab';
 return {size,grid,spacing:size/grid,rows:sphere?grid/2:grid,sphere,radius:size/(Math.PI*2),half:size/2};
}
export function normal(x:number,z:number,c:WorldConfig){const r=geometry(c).radius,lon=x/r,lat=z/r;return new Vector3(Math.sin(lon)*Math.cos(lat),Math.sin(lat),Math.cos(lon)*Math.cos(lat));}
export function coordinates(p:Vector3,c:WorldConfig){const g=geometry(c);return g.sphere?[Math.atan2(p.x,p.z)*g.radius,Math.asin(Math.max(-1,Math.min(1,p.y/p.length())))*g.radius]:[p.x,p.z];}
export function ground(x:number,z:number,c:WorldConfig){
 const g=geometry(c);
 if(g.sphere){const n=normal(x,z,c);return noise(n.x*12+20,n.y*12+20,c.seed)*4.2+noise(n.z*18+20,n.x*18+20,c.seed+72)*1.2-2.3;}
 return noise(x*.065+20,z*.065+20,c.seed)*5.2+noise(x*.16,z*.16,c.seed+72)*1.2-2.15-Math.pow(Math.hypot(x,z)/(g.half-1),4)*4;
}
export function point(i:number,c:WorldConfig){const g=geometry(c);return [(i%g.grid+.5)*g.spacing-g.half,(Math.floor(i/g.grid)+.5)*g.spacing-(g.sphere?g.half/2:g.half)];}
export function position(x:number,z:number,height:number,c:WorldConfig){const g=geometry(c);return g.sphere?normal(x,z,c).multiplyScalar(g.radius+height):new Vector3(x,height,z);}
export function site(x:number,z:number,c:WorldConfig){const g=geometry(c),u=g.sphere?((x+g.half)%g.size+g.size)%g.size:x+g.half;return Math.max(0,Math.min(g.rows-1,Math.floor((z+(g.sphere?g.half/2:g.half))/g.spacing)))*g.grid+Math.max(0,Math.min(g.grid-1,Math.floor(u/g.spacing)));}
export function sample(field:number[]|undefined,x:number,z:number,c:WorldConfig){
 if(!field)return 0;const g=geometry(c),raw=(x+g.half)/g.spacing-.5,gx=g.sphere?raw:Math.max(0,Math.min(g.grid-1,raw)),gz=Math.max(0,Math.min(g.rows-1,(z+(g.sphere?g.half/2:g.half))/g.spacing-.5));
 const ix=Math.floor(gx),iz=Math.floor(gz),jx=ix+1,jz=Math.min(g.rows-1,iz+1),u=gx-ix,v=gz-iz;
 const wrap=(i:number)=>g.sphere?(i%g.grid+g.grid)%g.grid:Math.max(0,Math.min(g.grid-1,i));
 return ((field[iz*g.grid+wrap(ix)]??0)*(1-u)+(field[iz*g.grid+wrap(jx)]??0)*u)*(1-v)+((field[jz*g.grid+wrap(ix)]??0)*(1-u)+(field[jz*g.grid+wrap(jx)]??0)*u)*v;
}
