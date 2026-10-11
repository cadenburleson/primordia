import type {Command, WorldConfig} from './sim/types';
import {geometry,normal,coordinates} from './sim/geometry';
export type BrushKind='raise'|'lower'|'flatten'|'fuel'|'mineral'|'detritus'|'water'|'life';
/** For the life brush, `amount` is organisms per dab and `life` names the registered creature. */
export interface BrushSettings {tool:BrushKind;radius:number;amount:number;aspect:number;angle:number;level?:number;life?:string}
export interface BrushPoint {x:number;z:number;height:number}
export interface BrushTool {settings:BrushSettings;begin:(p:BrushPoint)=>boolean;move:(p:BrushPoint)=>void;end:()=>void}
export interface BrushAck {type:'brush-ack';id:number;done:boolean;error?:string;note?:string}

// Arc-length sampling: dose follows distance, not pointer-event frequency.
// Globe paths follow the short great-circle arc, including the date-line seam.
export class BrushPath {
  private last:[number,number]|null=null;
  private remainder=0;
  truncated=false;
  constructor(private config:WorldConfig,private spacing:number){}
  break(){this.last=null;this.remainder=0;}
  sample(p:[number,number]):[number,number][] {
    this.truncated=false;
    if(!this.last){this.last=p;return [p];}
    const a=this.last,g=geometry(this.config),na=normal(...a,this.config),nb=normal(...p,this.config);
    const arc=Math.acos(Math.max(-1,Math.min(1,na.dot(nb))));
    const distance=g.sphere?arc*g.radius:Math.hypot(p[0]-a[0],p[1]-a[1]);
    if(distance<1e-6)return [];
    // An abrupt cursor jump should not paint the unseen far side of a planet.
    if(g.sphere&&arc>Math.PI*.5){this.last=p;this.remainder=0;return [p];}
    this.truncated=Math.floor((this.remainder+distance)/this.spacing)>256;
    const points:[number,number][]=[];
    for(let d=this.spacing-this.remainder;d<=distance&&points.length<256;d+=this.spacing){
      const t=d/distance;
      if(g.sphere){const n=arc<1e-5?na.clone().lerp(nb,t).normalize():na.clone().multiplyScalar(Math.sin((1-t)*arc)/Math.sin(arc)).addScaledVector(nb,Math.sin(t*arc)/Math.sin(arc));const [x,z]=coordinates(n,this.config);points.push([x,z]);}
      else points.push([a[0]+(p[0]-a[0])*t,a[1]+(p[1]-a[1])*t]);
    }
    this.remainder=(this.remainder+distance)%this.spacing;this.last=p;return points;
  }
}

// One worker request in flight. Batches are bounded; a whole drag has one undo.
// No timers or work while the brush is merely armed.
export class BrushStroke {
  private serial=0;private id=0;private points:[number,number][]=[];private settings:BrushSettings|null=null;
  private path:BrushPath|null=null;private flight=false;private first=true;private released=false;
  private timer:ReturnType<typeof setTimeout>|null=null;private clipped=false;
  constructor(private send:(c:Command)=>void,private changed:()=>void,private report:(s:string)=>void){}
  get busy(){return this.settings!==null;}
  get finishing(){return this.busy&&this.released;}
  begin(p:BrushPoint,settings:BrushSettings,config:WorldConfig){
    if(this.busy)return false;
    this.id=++this.serial;this.settings={...settings,level:settings.tool==='flatten'?p.height:undefined};this.first=true;this.released=false;this.clipped=false;
    // Terrain dabs follow the field grid; creatures are scattered by the brush's own size.
    this.path=new BrushPath(config,settings.tool==='life'?Math.max(.3,settings.radius*.7):Math.max(geometry(config).spacing*.4,settings.radius*settings.aspect*.3));
    this.points=this.path.sample([p.x,p.z]);this.flush();this.changed();return true;
  }
  move(p:BrushPoint){if(!this.path||this.released)return;const next=this.path.sample([p.x,p.z]);const room=256-this.points.length;if(next.length>room||this.path.truncated)this.clipped=true;this.points.push(...next.slice(0,room));this.schedule();}
  end(){if(!this.busy)return;this.released=true;if(this.timer){clearTimeout(this.timer);this.timer=null;}this.flush();this.changed();}
  // Stop adds no more paint; already accepted points finish before other edits.
  stop(){this.end();}
  accept(m:BrushAck){
    if(m.id!==this.id||!this.busy)return;
    this.flight=false;
    if(m.error||m.done){const clipped=this.clipped;this.reset();this.changed();this.report(m.error??m.note??(clipped?'Stroke saved. The input queue filled; paint shorter strokes for full coverage.':'Stroke saved · undo restores the whole stroke.'));return;}
    if(this.released)this.flush();else this.schedule();this.changed();
  }
  private reset(){if(this.timer)clearTimeout(this.timer);this.timer=null;this.points=[];this.settings=null;this.path=null;this.flight=false;}
  private schedule(){if(this.timer||this.flight||!this.points.length)return;this.timer=setTimeout(()=>{this.timer=null;this.flush();},140);}
  private flush(){
    if(this.flight||!this.settings||(!this.points.length&&!this.released))return;
    const points=this.points.splice(0,12),end=this.released&&!this.points.length;
    this.flight=true;this.send({type:'brush-stroke',id:this.id,start:this.first,end,settings:this.settings,points});this.first=false;
  }
}
