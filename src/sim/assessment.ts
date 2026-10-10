import type {Snapshot,ComparisonControl,ComparisonResult,Assessment} from './types';
/** Records snapshot measurements without selecting genomes or scoring fitness. */
export class AssessmentRecorder {
  start:Snapshot|null=null;last:Snapshot|null=null;control:ComparisonControl|null=null;
  duration=20;running=false;results:ComparisonResult[]=[];changed=false;lost=false;ambientTravel=0;
  reset(){this.start=null;this.last=null;this.control=null;this.results=[];this.running=false;}
  begin(s:Snapshot,control:ComparisonControl,duration:number){this.start=s;this.last=s;this.control=control;this.duration=duration;this.running=true;this.changed=false;this.lost=false;this.ambientTravel=0;}
  observe(s:Snapshot){
    if(!this.running||!this.start)return;
    const a=this.start.selected?.observation,b=s.selected?.observation,old=this.last?.selected?.observation;
    if(!b)this.lost=true;
    if(!b||b.membership!==a?.membership)this.changed=true;
    if(old&&b&&this.last){const dt=Math.max(0,s.stats.time-this.last.stats.time),v=old.ambientFlow,w=b.ambientFlow;this.ambientTravel+=Math.hypot((v.x+w.x)/2,(v.y+w.y)/2,(v.z+w.z)/2)*dt;}
    this.last=s;
  }
  finish(s:Snapshot){
    this.observe(s);const a=this.start;if(!a||!this.control)return;
    const p=a.selected?.observation,q=s.selected?.observation,stable=!this.changed&&!!p&&!!q;
    let distance:number|null=null;
    if(stable){const u=p!.centroid,v=q!.centroid;if(s.config.worldShape==='sphere'){const r=(s.config.worldSize??64)/(2*Math.PI),cross=Math.hypot(u.y*v.z-u.z*v.y,u.z*v.x-u.x*v.z,u.x*v.y-u.y*v.x),dot=u.x*v.x+u.y*v.y+u.z*v.z;distance=r*Math.atan2(cross,dot);}else distance=Math.hypot(v.x-u.x,v.z-u.z);}
    const food=(o:NonNullable<typeof p>)=>o.flux.fuel+o.flux.detritus+o.flux.prey+o.flux.absorbed;
    this.results.push({control:this.control,cycles:s.stats.time-a.stats.time,targetCycles:this.duration,body:a.selected?.id??0,membershipChanged:this.changed,bodyLost:this.lost,distance,ambientTravel:this.ambientTravel,work:stable?Math.max(0,(s.selected?.body.work??0)-(a.selected?.body.work??0)):null,food:stable?Math.max(0,food(q!)-food(p!)):null,prey:stable?Math.max(0,q!.flux.prey-p!.flux.prey):null,soluteTransport:stable?Math.max(0,q!.flux.solute_transport-p!.flux.solute_transport):null,births:s.stats.births-a.stats.births,deaths:s.stats.deaths-a.stats.deaths,cells:s.stats.cells,regions:s.autonomous?.physiology?.regions.reduce((n,r)=>n+r.regions,0)??0});
    this.results=this.results.slice(-16);this.running=false;
  }
  view(checkpoint:boolean,time:number):Assessment{return {checkpoint,running:this.running,control:this.control,elapsed:this.start?Math.max(0,time-this.start.stats.time):0,duration:this.duration,results:this.results};}
}
