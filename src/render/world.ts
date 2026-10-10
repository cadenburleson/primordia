import type {BrushTool,BrushPoint} from '../brush';
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { habitatY, noise } from '../sim/random';
import { STRIDE, CELL_STRIDE, LINK_STRIDE, JOINT_STRIDE, MAX_BODY_CELLS, type Snapshot, type WorldConfig } from '../sim/types';
import {geometry,ground,point,position,coordinates,sample} from '../sim/geometry';
import {jointGeometry} from './joints';
import {CELL_TYPES,CHOSEN_COLOR,cellType,isChosen} from '../sim/roles';
import {resourceValues,resourceRGB,type ResourceLayer} from '../sim/resource-view';

const MAX_CELLS = 65000;
const temp = new THREE.Object3D(), color = new THREE.Color();
const chosenColor=new THREE.Color(CHOSEN_COLOR),injuryColor=new THREE.Color('#653e48'),mineralColor=new THREE.Color('#b7c9bd'),snowColor=new THREE.Color('#e5ece1');
export function bodyOffset(cell: number, count: number, shape: number, branching: number): THREE.Vector3 {
  if (count === 1) return new THREE.Vector3();
  if (shape < .34) {
    const t = (cell - (count-1) / 2) * .52;
    return new THREE.Vector3(t, Math.sin(cell * 1.9) * .07, Math.sin(cell * 1.2) * branching * .5);
  }
  if (shape < .68) {
    const angle = cell * 2.39996, r = .26 * Math.sqrt(cell);
    return new THREE.Vector3(Math.cos(angle)*r, Math.sin(cell*2.1)*.16, Math.sin(angle)*r);
  }
  const arm = cell % 3, level = Math.floor(cell/3);
  const angle = arm * Math.PI*2/3 + level*branching*.45;
  return new THREE.Vector3(Math.cos(angle)*level*.43, level*.06, Math.sin(angle)*level*.43);
}

export class WorldView {
  renderer: THREE.WebGLRenderer;
  scene = new THREE.Scene();
  matter = new THREE.InstancedMesh(new THREE.IcosahedronGeometry(1,0),new THREE.MeshStandardMaterial({color:'#a08b67',roughness:.9}),4096);
  camera = new THREE.PerspectiveCamera(40, 1, .1, 600);
  controls: OrbitControls;
  cells: THREE.InstancedMesh;
  bones: THREE.InstancedMesh;
  jointCuffs:THREE.InstancedMesh;
  jointMuscles:THREE.InstancedMesh;
  terrain = new THREE.Group();
  terrainSurface: THREE.Mesh<THREE.BufferGeometry>|null=null;
  sun!: THREE.DirectionalLight;
  resourceField:ResourceLayer='fuel';
  resourceRange:{min:number;max:number;available:boolean}={min:0,max:0,available:false};
  private terrainDirty=true;
  private terrainSamples:Float32Array|null=null;
  private waterIndices:number[]=[];
  private waterIndexBuffer:THREE.BufferAttribute|null=null;
  private terrainRevision=-1;
  private snowAmount=-1;
  flow = new THREE.LineSegments(new THREE.BufferGeometry(),new THREE.LineBasicMaterial({color:0x82d6e3,transparent:true,opacity:.7,depthTest:false}));
  water: THREE.Mesh|null=null;
  trace = new THREE.Line(new THREE.BufferGeometry(),new THREE.LineBasicMaterial({color:'#a5d89d',transparent:true,opacity:.6}));
  private positions=new Float32Array(MAX_CELLS*3);
  updateMs=0;
  private linkMotion:{index:number;a:number;b:number;kind:number;activation:number}[]=[];
  private jointMotion:{a:number;b:number;c:number;flexion:number;extension:number}[]=[];
  ring: THREE.Mesh;
  snapshot: Snapshot | null = null;
  cellIds: number[] = [];
  selectedId: number | null = null;
  selected = new THREE.Group();
  private selectedBounds=new THREE.Box3Helper(new THREE.Box3(),0xf2df92);
  private brush:BrushTool|null=null;
  private painting:number|null=null;
  private lastBrushMove=0;
  private footprint=new THREE.LineLoop(new THREE.BufferGeometry(),new THREE.LineBasicMaterial({color:'#efffb7',transparent:true,opacity:.95,depthTest:false}));
  private brushCenter:BrushPoint|null=null;
  private placement:((x:number,z:number)=>void)|null=null;
  follow = false;
  overlay = false;
  quality = 'low';
  cameraMode: 'local'|'overview' = 'overview';
  private renderDirty = true;
  private lastDraw = 0;
  private lastTerrainUpdate = 0;
  private overviewPositions = new THREE.BufferAttribute(this.positions,3);
  private pickRadii=new Float32Array(MAX_CELLS);
  private overviewCells = new THREE.Points(new THREE.BufferGeometry(),new THREE.PointsMaterial({color:'#b7df9d',size:5,sizeAttenuation:false,depthTest:true}));
  config: WorldConfig | null = null;
  fps = 0;
  private frame = 0;
  private fpsTime = performance.now();
  private alive = true;
  private blurBrush=()=>this.finishBrush();
  private resizeObserver: ResizeObserver;
  private onSelect: (id: number | null) => void;

  constructor(public host: HTMLElement, onSelect: (id: number | null) => void) {
    this.onSelect = onSelect;
    this.renderer = new THREE.WebGLRenderer({ antialias: false, powerPreference: 'high-performance' });
    this.renderer.setPixelRatio(1);
    this.renderer.setClearColor('#10191d');
    this.scene.background = new THREE.Color('#10191d');
    this.matter.count=0;this.matter.frustumCulled=false;this.matter.instanceMatrix.setUsage(THREE.DynamicDrawUsage);this.scene.add(this.matter);
    this.renderer.toneMapping = THREE.NoToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.shadowMap.enabled = false;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.host.appendChild(this.renderer.domElement);
    this.scene.fog = null;
    this.camera.position.set(55, 45, 60);
    this.controls = this.createControls(new THREE.Vector3(),3,120,Math.PI*.47);
    const ambient = new THREE.HemisphereLight('#d8ffe6', '#172338', 1.8);
    this.scene.add(ambient);
    const sun = new THREE.DirectionalLight('#fff4d8', 2.8); sun.position.set(-25, 45, 15);
    sun.castShadow = false;
    Object.assign(sun.shadow.camera, { left: -38, right: 38, top: 38, bottom: -38, near: 1, far: 110 });
    sun.shadow.bias = -.0006; this.scene.add(sun);
    this.sun=sun;
    this.cells = new THREE.InstancedMesh(new THREE.SphereGeometry(1, 8, 6), new THREE.MeshStandardMaterial({roughness:1,metalness:0}), MAX_CELLS);
    this.bones = new THREE.InstancedMesh(new THREE.CylinderGeometry(.06,.06,1,5),new THREE.MeshStandardMaterial({ color:'#dfdabd',roughness:.6 }), 22000);
    this.jointCuffs=new THREE.InstancedMesh(new THREE.TorusGeometry(.23,.026,6,14),new THREE.MeshStandardMaterial({color:'#e7e0bb',roughness:.4}),8000);
    this.jointMuscles=new THREE.InstancedMesh(new THREE.CylinderGeometry(.035,.035,1,6),new THREE.MeshStandardMaterial({roughness:.4}),16000);
    for(const mesh of [this.jointCuffs,this.jointMuscles]){mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);mesh.frustumCulled=false;mesh.count=0;this.scene.add(mesh);}
    this.cells.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.bones.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.cells.frustumCulled = false; this.bones.frustumCulled = false;
    this.cells.count = 0; this.bones.count = 0;
    this.flow.visible=false;
    this.scene.add(this.cells, this.bones, this.terrain, this.selected,this.trace,this.flow);
    this.selected.add(new THREE.Mesh(new THREE.TorusGeometry(1,.025,8,64),new THREE.MeshBasicMaterial({ color:'#ebf5be',depthTest:false })));
    this.selected.rotation.x = Math.PI/2; this.selected.visible = false;
    this.selectedBounds.visible=false;this.scene.add(this.selectedBounds);
    this.ring = new THREE.Mesh(new THREE.TorusGeometry(33,.025,6,200),new THREE.MeshBasicMaterial({ color:'#577569',transparent:true,opacity:.45 }));
    this.ring.rotation.x = Math.PI/2; this.ring.position.y = -2; this.scene.add(this.ring);
    this.overviewCells.geometry.setAttribute('position',this.overviewPositions);
    this.overviewCells.geometry.setDrawRange(0,0);this.overviewCells.frustumCulled=false;this.overviewCells.visible=false;
    this.scene.add(this.overviewCells);
    this.resizeObserver = new ResizeObserver(()=>this.resize()); this.resizeObserver.observe(host);
    this.footprint.geometry.setAttribute('position',new THREE.BufferAttribute(new Float32Array(64*3),3));
    this.footprint.frustumCulled=false;this.footprint.visible=false;this.footprint.renderOrder=20;this.scene.add(this.footprint);
    const canvas=this.renderer.domElement;
    canvas.addEventListener('pointerdown',e=>{
      if(!this.brush||e.button!==0||e.altKey||e.ctrlKey||e.metaKey||this.painting!==null)return;
      e.preventDefault();e.stopImmediatePropagation();
      const p=this.brushHit(e);if(!p)return;
      this.drawBrush(p);
      if(this.brush.begin(p)){this.painting=e.pointerId;this.controls.enabled=false;canvas.setPointerCapture(e.pointerId);}
    },true);
    canvas.addEventListener('pointermove',e=>{
      if(!this.brush)return;
      if(this.painting!==null){e.preventDefault();e.stopImmediatePropagation();if(e.pointerId!==this.painting)return;}
      if(performance.now()-this.lastBrushMove<32)return;this.lastBrushMove=performance.now();
      const p=this.brushHit(e);
      if(p){this.drawBrush(p);if(this.painting!==null)this.brush.move(p);}
      else{this.footprint.visible=false;this.renderDirty=true;if(this.painting!==null)this.finishBrush();}
    },true);
    const end=(e:PointerEvent)=>{if(e.pointerId!==this.painting)return;e.preventDefault();e.stopImmediatePropagation();if(e.type==='pointerup'){const p=this.brushHit(e);if(p)this.brush?.move(p);}this.finishBrush();};
    canvas.addEventListener('pointerup',end,true);canvas.addEventListener('pointercancel',end,true);canvas.addEventListener('lostpointercapture',end,true);
    canvas.addEventListener('pointerleave',()=>{if(this.painting===null){this.footprint.visible=false;this.renderDirty=true;}});
    let downX=0,downY=0;
    this.renderer.domElement.addEventListener('pointerdown',e=>{downX=e.clientX;downY=e.clientY;});
    this.renderer.domElement.addEventListener('pointerup',e=>{if(!this.brush&&e.button===0&&Math.hypot(e.clientX-downX,e.clientY-downY)<5)this.pick(e);});
    window.addEventListener('blur',this.blurBrush);
    this.resize(); this.animate();
  }
  private createControls(target:THREE.Vector3,minDistance:number,maxDistance:number,maxPolarAngle:number):OrbitControls{
    const position=this.camera.position.clone();
    const controls=new OrbitControls(this.camera,this.renderer.domElement);
    this.camera.position.copy(position);
    controls.target.copy(target);
    controls.enableDamping=true;controls.dampingFactor=.12;
    controls.minDistance=minDistance;controls.maxDistance=maxDistance;
    controls.minPolarAngle=.02;controls.maxPolarAngle=Math.min(maxPolarAngle,Math.PI-.02);
    controls.autoRotate=false;
    if(this.brush)controls.mouseButtons.RIGHT=THREE.MOUSE.ROTATE;
    controls.addEventListener('change',()=>{this.renderDirty=true;});
    controls.addEventListener('start',()=>{controls.autoRotate=false;this.follow=false;});
    controls.update();
    return controls;
  }
  private rebindControls(){
    // OrbitControls captures camera.up on construction. Rebind after a new
    // surface frame or camera pose so orbit axes and pending drag inertia agree.
    const {target,minDistance,maxDistance,maxPolarAngle}=this.controls;
    this.controls.dispose();
    this.controls=this.createControls(target.clone(),minDistance,maxDistance,maxPolarAngle);
    this.renderDirty=true;
  }
  private resize() {
    const {width,height}=this.host.getBoundingClientRect();
    this.camera.aspect=width/Math.max(1,height);this.camera.updateProjectionMatrix();
    if(this.controls.autoRotate&&!this.follow){const g=this.config?geometry(this.config):null;this.camera.position.normalize().multiplyScalar((this.camera.aspect>1.25?72:95)*(g?.sphere?g.radius/20.37:(g?.size??64)/64));}
    this.renderer.setSize(width,height);this.renderDirty=true;
  }
  build(config: WorldConfig) {
    this.terrainSurface=null;this.terrainSamples=null;this.terrainRevision=-1;this.snowAmount=-1;
    this.config=config;
    const dimensions=geometry(config);this.scene.fog=null;this.controls.maxDistance=Math.max(120,dimensions.size*3);this.controls.maxPolarAngle=dimensions.sphere?Math.PI:Math.PI*.47;this.camera.far=Math.max(600,dimensions.size*8);this.camera.updateProjectionMatrix();this.ring.scale.setScalar(dimensions.size/64);
    const shadowSpan=dimensions.sphere?dimensions.radius+10:dimensions.half+6;
    Object.assign(this.sun.shadow.camera,{left:-shadowSpan,right:shadowSpan,top:shadowSpan,bottom:-shadowSpan,far:Math.max(110,dimensions.size*3)});this.sun.shadow.camera.updateProjectionMatrix();
    while(this.terrain.children.length){const child=this.terrain.children[0];this.terrain.remove(child);child.traverse(obj=>{if(obj instanceof THREE.Mesh||obj instanceof THREE.Points){obj.geometry.dispose();const materials=Array.isArray(obj.material)?obj.material:[obj.material];materials.forEach(m=>m.dispose());}});}
    this.water=null;this.waterIndices=[];this.waterIndexBuffer=null;this.terrainDirty=true;this.lastTerrainUpdate=0;this.ring.visible=false;
    if(config.kind==='lab'){
      const floor=new THREE.Mesh(new THREE.CircleGeometry(31,128),new THREE.MeshStandardMaterial({color:'#25372e',roughness:.9}));
      floor.rotation.x=-Math.PI/2;floor.receiveShadow=true;this.terrain.add(floor);
      const grid=new THREE.GridHelper(60,120,'#526b53','#354c3e');grid.position.y=.002;this.terrain.add(grid);
      const base=new THREE.Mesh(new THREE.CylinderGeometry(31,30,1,128),new THREE.MeshStandardMaterial({color:'#15241d',roughness:.9}));base.position.y=-.51;this.terrain.add(base);
      this.scene.background=new THREE.Color('#10191d');this.resetCamera();return;
    }
    if(dimensions.sphere){this.buildSphere(config);return;}
    const seed=config.seed,size=dimensions.size;
    const segments=Math.max(80,dimensions.grid*2);
    const geo=new THREE.PlaneGeometry(size,size,segments,segments);geo.rotateX(-Math.PI/2);
    const pos=geo.attributes.position, colors=new Float32Array(pos.count*3);
    const tidal=config.kind==='tidal', alien=config.kind==='alien';
    const low=new THREE.Color(tidal?'#394a3d':alien?'#5f4567':'#47342c');
    const high=new THREE.Color(tidal?'#aab58b':alien?'#ae98b9':'#b7a58e');
    for(let i=0;i<pos.count;i++){
      const x=pos.getX(i),z=pos.getZ(i),h=ground(x,z,config);pos.setY(i,h);
      color.copy(low).lerp(high,THREE.MathUtils.clamp((h+.1)/3.5,0,1));
      color.multiplyScalar(.86+noise(x*.8,z*.8,seed)*.22);color.toArray(colors,i*3);
    }
    const indices:number[]=[];const old=geo.index!;
    for(let i=0;i<old.count;i+=3){const ids=[old.getX(i),old.getX(i+1),old.getX(i+2)];indices.push(...ids);}
    geo.setIndex(indices);geo.setAttribute('color',new THREE.BufferAttribute(colors,3));geo.computeVertexNormals();
    const terrain=new THREE.Mesh(geo,new THREE.MeshStandardMaterial({vertexColors:true,roughness:.96,flatShading:false}));terrain.receiveShadow=true;this.terrainSurface=terrain;this.terrain.add(terrain);
    const base=new THREE.Mesh(new THREE.BoxGeometry(size,4,size),new THREE.MeshStandardMaterial({ color:tidal?'#182b28':alien?'#2a1d32':'#241c18',roughness:.9,flatShading:true }));base.position.y=-4.5;this.terrain.add(base);
    this.createWater(geo);
    this.scene.fog=null;
    this.scene.background=new THREE.Color(alien?'#14131e':'#10191d');
    this.resetCamera();
  }
  private buildSphere(config:WorldConfig){
    const g=geometry(config),segments=Math.max(80,g.grid*2),rows=Math.max(40,g.rows*2);
    const geo=new THREE.SphereGeometry(1,segments,rows),pos=geo.attributes.position,colors=new Float32Array(pos.count*3);
    for(let i=0;i<pos.count;i++){const n=new THREE.Vector3(pos.getX(i),pos.getY(i),pos.getZ(i)).normalize();const[x,z]=coordinates(n,config),h=ground(x,z,config);n.multiplyScalar(g.radius+h);pos.setXYZ(i,n.x,n.y,n.z);color.set('#394a3d').lerp(new THREE.Color('#aab58b'),THREE.MathUtils.clamp((h+.1)/3.5,0,1));color.toArray(colors,i*3);}
    geo.setAttribute('color',new THREE.BufferAttribute(colors,3));geo.computeVertexNormals();
    this.terrainSurface=new THREE.Mesh(geo,new THREE.MeshStandardMaterial({vertexColors:true,roughness:.96}));this.terrainSurface.receiveShadow=true;this.terrain.add(this.terrainSurface);
    this.createWater(geo);
    this.ring.visible=false;this.scene.fog=null;this.scene.background=new THREE.Color('#10191d');this.resetCamera();
  }
  private createWater(terrain:THREE.BufferGeometry){
    const config=this.config!,dimensions=geometry(config);
    const terrainPosition=terrain.attributes.position;
    this.terrainSamples=new Float32Array(terrainPosition.count*6);
    for(let i=0;i<terrainPosition.count;i++){
      const v=new THREE.Vector3().fromBufferAttribute(terrainPosition,i),[x,z]=coordinates(v,config),offset=i*6;
      this.terrainSamples[offset]=x;this.terrainSamples[offset+1]=z;this.terrainSamples[offset+2]=ground(x,z,config);
      if(dimensions.sphere){v.normalize();this.terrainSamples[offset+3]=v.x;this.terrainSamples[offset+4]=v.y;this.terrainSamples[offset+5]=v.z;}
    }
    // Match terrain topology exactly. Dry triangles never receive a water face.
    const geo=terrain.clone();geo.deleteAttribute('color');
    this.waterIndices=Array.from(geo.index!.array);
    this.waterIndexBuffer=new THREE.BufferAttribute(new Uint32Array(this.waterIndices),1);this.waterIndexBuffer.setUsage(THREE.DynamicDrawUsage);geo.setIndex(this.waterIndexBuffer);geo.setDrawRange(0,0);
    this.water=new THREE.Mesh(geo,new THREE.MeshStandardMaterial({color:'#469ed0',roughness:1,metalness:0,transparent:true,opacity:.48,depthWrite:false,polygonOffset:true,polygonOffsetFactor:-1,polygonOffsetUnits:-2}));
    this.water.visible=!this.overlay;this.terrain.add(this.water);
  }
  update(snapshot: Snapshot) {
    const preparationStart=performance.now();
    const matter=snapshot.autonomous?.systems?.particles??[];
    this.matter.count=Math.min(4096,matter.length);
    for(let i=0;i<this.matter.count;i++){const p=matter[i];temp.position.set(p.p.x,p.p.y,p.p.z);temp.rotation.set(0,0,0);temp.scale.setScalar(p.radius);temp.updateMatrix();this.matter.setMatrixAt(i,temp.matrix);color.set('#9c825c').lerp(mineralColor,p.mineral);this.matter.setColorAt(i,color);}
    this.matter.instanceMatrix.needsUpdate=true;if(this.matter.instanceColor)this.matter.instanceColor.needsUpdate=true;
    this.snapshot=snapshot;
    if(!this.config||snapshot.config.seed!==this.config.seed||snapshot.config.kind!==this.config.kind||snapshot.config.worldSize!==this.config.worldSize||snapshot.config.worldShape!==this.config.worldShape)this.build(snapshot.config);
    const planet=snapshot.autonomous?.planet,g=geometry(snapshot.config);
    const density=this.overlay?resourceValues(snapshot,this.resourceField):[];
    const fieldMax=density.reduce((m,v)=>Math.max(m,v),0);
    this.resourceRange={min:density.length?Math.min(...density):0,max:fieldMax,available:density.length>0};
    if(this.terrainSurface){
      const material=this.terrainSurface.material;
      if(!Array.isArray(material)&&((this.overlay&&!(material instanceof THREE.MeshBasicMaterial))||(!this.overlay&&material instanceof THREE.MeshBasicMaterial))){
        material.dispose();this.terrainSurface.material=this.overlay?new THREE.MeshBasicMaterial({vertexColors:true}):new THREE.MeshStandardMaterial({vertexColors:true,roughness:.96});
      }
    }
    if(this.water)this.water.visible=!this.overlay;
    if(planet){
      this.sun.intensity=1.2+planet.sunlight*snapshot.config.sunlight*(1-.8*(planet.cloudCover??0));if(g.sphere&&planet.sunDirection)this.sun.position.set(...planet.sunDirection as [number,number,number]).multiplyScalar(g.radius*4);else this.sun.position.set(-25,45,15);
    }else{this.sun.intensity=2.8;}
      if(this.terrainSurface&&(this.terrainDirty||(performance.now()-this.lastTerrainUpdate>=1000&&(g.sphere||this.overlay||planet?.terrainVersion!==this.terrainRevision||Math.abs((planet?.snow??0)-this.snowAmount)>.25)))){
        const geo=this.terrainSurface.geometry,pos=geo.attributes.position,colors=geo.attributes.color;
        const low=new THREE.Color(snapshot.config.kind==='tidal'?'#394a3d':snapshot.config.kind==='alien'?'#5f4567':'#47342c'),high=new THREE.Color('#aab58b');
        const depths=new Float32Array(pos.count),wp=this.overlay?undefined:this.water?.geometry.attributes.position;
        for(let i=0;i<pos.count;i++){
          const base=this.terrainSamples!,o=i*6,x=base[o],z=base[o+1],h=base[o+2]+sample(planet?.relief,x,z,snapshot.config);
          if(g.sphere){const radius=g.radius+h;pos.setXYZ(i,base[o+3]*radius,base[o+4]*radius,base[o+5]*radius);}else{pos.setXYZ(i,x,h,z);}
          color.copy(low).lerp(high,THREE.MathUtils.clamp((h+.1)/3.5,0,1)).lerp(snowColor,Math.min(1,sample(planet?.snowCover,x,z,snapshot.config)*3));
          if(this.overlay){const rgb=resourceRGB(this.resourceField,sample(density,x,z,snapshot.config),fieldMax);color.setRGB(rgb[0]/255,rgb[1]/255,rgb[2]/255,THREE.SRGBColorSpace);}
          colors.setXYZ(i,color.r,color.g,color.b);
          const depth=planet?.waterDepth?sample(planet.waterDepth,x,z,snapshot.config):Math.max(0,.12-h);depths[i]=depth;
          if(wp){if(g.sphere){const radius=g.radius+h+depth+.015;wp.setXYZ(i,base[o+3]*radius,base[o+4]*radius,base[o+5]*radius);}else{wp.setXYZ(i,x,h+depth+.015,z);}}
        }
        if(this.water&&wp&&this.waterIndexBuffer){
          this.waterIndexBuffer.array.set(this.waterIndices);this.water.geometry.computeVertexNormals();
          let wetCount=0;
          // Films below 0.015 units remain surface moisture, rather than a
          // coplanar transparent ocean. The display offset does not alter physics.
          for(let i=0;i<this.waterIndices.length;i+=3){const a=this.waterIndices[i],b=this.waterIndices[i+1],c=this.waterIndices[i+2];if(Math.min(depths[a],depths[b],depths[c])>.015){this.waterIndexBuffer.setX(wetCount++,a);this.waterIndexBuffer.setX(wetCount++,b);this.waterIndexBuffer.setX(wetCount++,c);}}
          this.water.geometry.setDrawRange(0,wetCount);this.waterIndexBuffer.needsUpdate=true;wp.needsUpdate=true;this.water.geometry.computeBoundingSphere();
        }
        pos.needsUpdate=true;colors.needsUpdate=true;if(!this.overlay&&(this.terrainDirty||planet?.terrainVersion!==this.terrainRevision))geo.computeVertexNormals();geo.computeBoundingSphere();this.terrainRevision=planet?.terrainVersion??-1;this.snowAmount=planet?.snow??0;this.lastTerrainUpdate=performance.now();this.terrainDirty=false;
      }
    let count=0,linkCount=0,selectedRadius=0;this.cellIds.length=0;this.selectedBounds.box.makeEmpty();
    const data=snapshot.data,physical=snapshot.cellData;
    const starts=new Int32Array(snapshot.ids.length).fill(-1),sizes=new Uint16Array(snapshot.ids.length);
    for(let p=0;p<physical.length/CELL_STRIDE;p++){const owner=physical[p*CELL_STRIDE];if(starts[owner]<0)starts[owner]=p;sizes[owner]++;}
    const nodeKeys=new Int32Array(physical.length/CELL_STRIDE).fill(-1);
    this.linkMotion=[];this.jointMotion=[];
    const quat=new THREE.Quaternion(),axis=new THREE.Vector3(0,1,0),materialTypes=!!snapshot.autonomous?.physics;
    for(let i=0;i<snapshot.ids.length&&count<MAX_CELLS;i++){
      const o=i*STRIDE,x=data[o],z=data[o+1],heading=data[o+2],actual=data[o+3],shape=data[o+4],pigment=data[o+5],photo=data[o+6],chemo=data[o+7];
      const worldY=snapshot.lab?.5:habitatY(x,z,.25+shape*.10,snapshot.config.seed);
      const detail=!!snapshot.lab||snapshot.ids[i]===this.selectedId||this.camera.position.distanceTo(new THREE.Vector3(x,worldY,z))<26;
      const displayed=snapshot.autonomous?actual:detail?Math.min(actual,MAX_BODY_CELLS):Math.min(actual,6),start=starts[i];
      quat.setFromAxisAngle(axis,-heading);
      for(let c=0;c<displayed&&count<MAX_CELLS;c++){
        const isPhysical=start>=0&&c<sizes[i];let role=0,health=1,chosen=false;
        const r=isPhysical?physical[(start+c)*CELL_STRIDE+4]:.25+shape*.10;
        if(isPhysical){
          const p=(start+c)*CELL_STRIDE;role=cellType(physical[p+5]);chosen=isChosen(physical[p+5]);health=physical[p+7];
          temp.position.set(physical[p+1],physical[p+2],physical[p+3]);
          nodeKeys[start+c]=count;
        }else{
          const offset=bodyOffset(c,displayed,shape,data[o+8]).applyQuaternion(quat);
          temp.position.set(x+offset.x,worldY+offset.y,z+offset.z);
        }
        if(snapshot.ids[i]===this.selectedId){this.selectedBounds.box.expandByPoint(temp.position);selectedRadius=Math.max(selectedRadius,r);}
        color.setHSL(.30+pigment*.42,.35+chemo*.3,.48+photo*.16);
        if(materialTypes){if(role>0&&role<CELL_TYPES.length)color.set(CELL_TYPES[role].color);if(chosen)color.lerp(chosenColor,.5);}
        else{if(role===1)color.set('#d8e5c4');if(role===2)color.set('#d88e72');if(role===3)color.set('#83c7d1');if(role===4)color.set('#ddb76b');if(role===5)color.set('#b5a2d9');}
        color.lerp(injuryColor,(1-health)*.8);
        this.cells.setColorAt(count,color);this.overviewPositions.setXYZ(count,temp.position.x,temp.position.y,temp.position.z);
        this.cellIds[count]=snapshot.ids[i];this.pickRadii[count]=r;count++;
      }
      if(snapshot.ids[i]===this.selectedId){
        if(g.sphere&&start>=0){const p=start*CELL_STRIDE;this.selected.position.set(physical[p+1],physical[p+2],physical[p+3]);this.selected.quaternion.setFromUnitVectors(new THREE.Vector3(0,0,1),this.selected.position.clone().normalize());}
        else{this.selected.position.set(x,snapshot.lab?.025:Math.max(.16,ground(x,z,snapshot.config))+.04,z);this.selected.rotation.set(-Math.PI/2,0,0);}
        this.selected.scale.setScalar(.65+Math.sqrt(actual)*.3);this.selected.visible=true;
        if(this.follow){const target=g.sphere?this.selected.position:new THREE.Vector3(x,worldY,z),delta=target.clone().sub(this.controls.target).multiplyScalar(.15);this.controls.target.add(delta);this.camera.position.add(delta);}
      }
    }

    if(!this.selectedBounds.box.isEmpty())this.selectedBounds.box.expandByScalar(selectedRadius);
    for(let p=0;p<snapshot.linkData.length;p+=LINK_STRIDE){
      const a=nodeKeys[snapshot.linkData[p]],b=nodeKeys[snapshot.linkData[p+1]];
      if(a===undefined||b===undefined||a<0||b<0||linkCount>=22000)continue;
      const kind=snapshot.linkData[p+2],activation=snapshot.linkData[p+3];
      this.linkMotion.push({index:linkCount,a,b,kind,activation});
      color.set(kind===2?'#e68c74':kind===1?'#e9e1bb':kind===3?'#7b9b9b':'#567c60');
      if(snapshot.linkData[p+5]<.15)color.set('#713946');
      if(kind===2)color.lerp(new THREE.Color('#ffdb9e'),activation*.7);
      this.bones.setColorAt(linkCount++,color);
    }
    this.cells.count=count;this.bones.count=linkCount;
    this.overviewCells.geometry.setDrawRange(0,count);this.overviewPositions.needsUpdate=true;this.applyCameraMode();
    for(let p=0;p<snapshot.jointData.length&&this.jointMotion.length<8000;p+=JOINT_STRIDE){
      const a=nodeKeys[snapshot.jointData[p]],b=nodeKeys[snapshot.jointData[p+1]],c=nodeKeys[snapshot.jointData[p+2]];
      if(a===undefined||b===undefined||c===undefined||a<0||b<0||c<0)continue;
      const index=this.jointMotion.length,flexion=snapshot.jointData[p+6],extension=snapshot.jointData[p+7];
      this.jointMotion.push({a,b,c,flexion,extension});
      for(let k=0;k<2;k++){color.set(k===0?'#d98569':'#b67f97').lerp(new THREE.Color('#ffe1a3'),(k===0?flexion:extension)*.7);this.jointMuscles.setColorAt(index*2+k,color);}
    }
    this.jointCuffs.count=this.jointMotion.length;this.jointMuscles.count=this.jointMotion.length*2;
    if(this.jointMuscles.instanceColor)this.jointMuscles.instanceColor.needsUpdate=true;
    this.animateBodies(1);
    if(this.cells.instanceColor)this.cells.instanceColor.needsUpdate=true;
    if(this.bones.instanceColor)this.bones.instanceColor.needsUpdate=true;
    if(this.selectedId&&!snapshot.ids.includes(this.selectedId))this.selected.visible=false;
    const physics=snapshot.autonomous?.physics,flow=physics?.flow;
    this.flow.visible=this.overlay&&!!flow;
    (this.flow.material as THREE.LineBasicMaterial).depthTest=g.sphere;
    if(flow&&this.overlay){const lines:number[]=[];
      if(g.sphere){for(const [x,y,z,u,v,w] of physics?.worldFlow??[]){const[xc,zc]=coordinates(new THREE.Vector3(x,y,z),snapshot.config),h=ground(xc,zc,snapshot.config)+sample(planet?.relief,xc,zc,snapshot.config)+sample(planet?.waterDepth,xc,zc,snapshot.config)+.4,p=position(xc,zc,h,snapshot.config);lines.push(p.x,p.y,p.z,p.x+u*18,p.y+v*18,p.z+w*18);}}
      else for(const [i,u,v] of flow){const[x,z]=point(i,snapshot.config),y=Math.max(.12,ground(x,z,snapshot.config))+sample(planet?.relief,x,z,snapshot.config)+.3,dx=u*18,dz=v*18;if(Math.hypot(dx,dz)<.025)continue;lines.push(x,y,z,x+dx,y,z+dz);}
      this.flow.geometry.dispose();this.flow.geometry=new THREE.BufferGeometry();this.flow.geometry.setAttribute('position',new THREE.Float32BufferAttribute(lines,3));
    }
    this.renderDirty=true;
    this.updateMs=performance.now()-preparationStart;
    this.trace.visible=!!snapshot.lab;
    if(snapshot.lab){this.trace.geometry.dispose();this.trace.geometry=new THREE.BufferGeometry().setFromPoints(snapshot.lab.trace.map(p=>new THREE.Vector3(p[0],.04,p[2])));}
  }
  private animateBodies(_alpha:number){
    if(this.overviewCells.visible)return;
    const a=new THREE.Vector3(),b=new THREE.Vector3(),c=new THREE.Vector3(),delta=new THREE.Vector3(),up=new THREE.Vector3(0,1,0);
    for(let i=0;i<this.cells.count;i++){
      temp.position.fromArray(this.positions,i*3);temp.rotation.set(0,0,0);temp.scale.setScalar(this.pickRadii[i]);temp.updateMatrix();this.cells.setMatrixAt(i,temp.matrix);
    }
    for(const link of this.linkMotion){
      a.fromArray(this.positions,link.a*3);b.fromArray(this.positions,link.b*3);delta.subVectors(b,a);
      const length=delta.length();temp.position.copy(a).add(b).multiplyScalar(.5);
      temp.quaternion.setFromUnitVectors(up,delta.normalize());
      const thickness=link.kind===2?.8+link.activation*.5:link.kind===1?.7:.3;
      temp.scale.set(thickness,length,thickness);temp.updateMatrix();this.bones.setMatrixAt(link.index,temp.matrix);
    }
    for(let i=0;i<this.jointMotion.length;i++){
      const j=this.jointMotion[i];a.fromArray(this.positions,j.a*3);b.fromArray(this.positions,j.b*3);c.fromArray(this.positions,j.c*3);
      const {axis,bands}=jointGeometry(a,b,c);
      temp.position.copy(b);temp.quaternion.setFromUnitVectors(new THREE.Vector3(0,0,1),axis);temp.scale.setScalar(1);temp.updateMatrix();this.jointCuffs.setMatrixAt(i,temp.matrix);
      for(let k=0;k<2;k++){
        const [from,to]=bands[k],activation=k===0?j.flexion:j.extension;delta.subVectors(to,from);const length=delta.length();
        temp.position.copy(from).add(to).multiplyScalar(.5);temp.quaternion.setFromUnitVectors(up,delta.normalize());temp.scale.set(1+activation*.5,length,1+activation*.5);temp.updateMatrix();this.jointMuscles.setMatrixAt(i*2+k,temp.matrix);
      }
    }
    this.jointCuffs.instanceMatrix.needsUpdate=true;this.jointMuscles.instanceMatrix.needsUpdate=true;
    this.cells.instanceMatrix.needsUpdate=true;this.bones.instanceMatrix.needsUpdate=true;
  }
  private applyCameraMode(){
    const markers=this.cameraMode==='overview'&&!!this.config&&geometry(this.config).sphere&&(this.config.worldSize??64)>256;
    this.overviewCells.visible=markers;
    for(const mesh of [this.cells,this.bones,this.jointCuffs,this.jointMuscles,this.matter])mesh.visible=!markers;
    if(markers)this.selected.visible=false;
    this.selectedBounds.visible=!markers&&!this.selectedBounds.box.isEmpty();
    this.renderDirty=true;
  }
  focusLife(id:number|null=this.selectedId,nearby=true):boolean{
    const s=this.snapshot;if(!s||!s.ids.length)return false;
    const owner=Math.max(0,id===null?0:s.ids.indexOf(id)),data=s.cellData;
    const g=geometry(s.config);let anchor:THREE.Vector3|null=null;
    for(let p=0;p<data.length;p+=CELL_STRIDE)if(data[p]===owner){anchor=new THREE.Vector3(data[p+1],data[p+2],data[p+3]);break;}
    if(!anchor)anchor=position(s.data[owner*STRIDE],s.data[owner*STRIDE+1],.5,s.config);
    const bounds=new THREE.Box3();bounds.expandByPoint(anchor);
    for(let p=0;p<data.length;p+=CELL_STRIDE){const v=new THREE.Vector3(data[p+1],data[p+2],data[p+3]);if(data[p]===owner||(nearby&&v.distanceToSquared(anchor)<24*24)){const r=data[p+4];bounds.expandByPoint(v.clone().addScalar(r));bounds.expandByPoint(v.clone().addScalar(-r));}}
    const extent=bounds.getBoundingSphere(new THREE.Sphere()),target=extent.center;
    const up=g.sphere?target.clone().normalize():new THREE.Vector3(0,1,0);
    const reference=Math.abs(up.y)>.9?new THREE.Vector3(1,0,0):new THREE.Vector3(0,1,0);
    const east=reference.cross(up).normalize(),north=up.clone().cross(east).normalize();
    const fov=Math.min(THREE.MathUtils.degToRad(this.camera.fov/2),Math.atan(Math.tan(THREE.MathUtils.degToRad(this.camera.fov/2))*this.camera.aspect));
    const distance=Math.max(6,extent.radius*1.25/Math.sin(fov));
    this.cameraMode='local';this.follow=!nearby;this.controls.autoRotate=false;this.controls.minDistance=1;this.controls.maxPolarAngle=Math.PI*.48;
    this.camera.up.copy(up);this.camera.near=.025;this.camera.far=Math.max(600,g.radius*4+100);this.camera.updateProjectionMatrix();
    this.controls.target.copy(target);this.camera.position.copy(target).add(up.clone().multiplyScalar(.85).add(east.multiplyScalar(.45)).add(north.multiplyScalar(.3)).normalize().multiplyScalar(distance));
    this.rebindControls();this.applyCameraMode();this.animateBodies(1);return true;
  }
  focusLocation(x:number,z:number){
    if(!this.config)return;const c=this.config,g=geometry(c),p=this.snapshot?.autonomous?.planet;
    const target=position(x,z,ground(x,z,c)+sample(p?.relief,x,z,c)+sample(p?.waterDepth,x,z,c)+.3,c),up=g.sphere?target.clone().normalize():new THREE.Vector3(0,1,0),side=(Math.abs(up.y)>.9?new THREE.Vector3(1,0,0):new THREE.Vector3(0,1,0)).cross(up).normalize();
    this.cameraMode='local';this.follow=false;this.controls.minDistance=1;this.controls.maxPolarAngle=Math.PI*.48;this.camera.up.copy(up);this.camera.near=.025;this.camera.far=Math.max(600,g.radius*4+100);this.camera.updateProjectionMatrix();this.controls.target.copy(target);this.camera.position.copy(target).add(up.clone().multiplyScalar(16).add(side.multiplyScalar(9)));this.rebindControls();this.applyCameraMode();this.animateBodies(1);
  }
  select(id: number|null,zoom=false) {
    this.selectedId=id;this.selected.visible=false;
    if(this.snapshot){this.update(this.snapshot);if(zoom&&id!==null)this.focusLife(id,false);}
  }
  invalidateTerrain(){this.terrainDirty=true;}
  setBrush(brush:BrushTool|null){
    this.finishBrush();this.brush=brush;this.controls.mouseButtons.RIGHT=brush?THREE.MOUSE.ROTATE:THREE.MOUSE.PAN;
    this.renderer.domElement.style.cursor=brush?'crosshair':this.placement?'crosshair':'';
    this.footprint.visible=false;if(!brush)this.brushCenter=null;if(brush&&this.brushCenter)this.drawBrush(this.brushCenter);this.renderDirty=true;
  }
  private finishBrush(){
    if(this.painting===null)return;const pointer=this.painting;this.painting=null;
    if(this.renderer.domElement.hasPointerCapture(pointer))this.renderer.domElement.releasePointerCapture(pointer);
    this.controls.enabled=true;this.rebindControls();this.brush?.end();
  }
  private brushHit(e:PointerEvent):BrushPoint|null{
    if(!this.config||!this.terrainSurface||document.elementFromPoint(e.clientX,e.clientY)!==this.renderer.domElement)return null;
    const rect=this.renderer.domElement.getBoundingClientRect();
    if(e.clientX<rect.left||e.clientX>rect.right||e.clientY<rect.top||e.clientY>rect.bottom)return null;
    this.camera.updateMatrixWorld();this.terrainSurface.updateMatrixWorld();
    const ray=new THREE.Raycaster();ray.setFromCamera(new THREE.Vector2((e.clientX-rect.left)/rect.width*2-1,-(e.clientY-rect.top)/rect.height*2+1),this.camera);
    const hit=ray.intersectObject(this.terrainSurface)[0];if(!hit)return null;
    const [x,z]=coordinates(hit.point,this.config);
    return {x,z,height:ground(x,z,this.config)+sample(this.snapshot?.autonomous?.planet?.relief,x,z,this.config)};
  }
  private drawBrush(p:BrushPoint){
    if(!this.config||!this.brush)return;this.brushCenter=p;
    const c=this.config,g=geometry(c),{radius,aspect,angle}=this.brush.settings,theta=angle*Math.PI/180;
    const center=position(p.x,p.z,0,c).normalize(),east=new THREE.Vector3(Math.cos(p.x/g.radius),0,-Math.sin(p.x/g.radius)),north=center.clone().cross(east);
    const vertices=this.footprint.geometry.getAttribute('position') as THREE.BufferAttribute,planet=this.snapshot?.autonomous?.planet;
    for(let i=0;i<64;i++){
      const a=i*Math.PI*2/64,u=Math.cos(a)*radius,v=Math.sin(a)*radius*aspect;
      const dx=u*Math.cos(theta)-v*Math.sin(theta),dz=u*Math.sin(theta)+v*Math.cos(theta);
      let x=p.x+dx,z=p.z+dz;
      if(g.sphere){const distance=Math.hypot(dx,dz),direction=east.clone().multiplyScalar(dx/distance).addScaledVector(north,dz/distance);[x,z]=coordinates(center.clone().multiplyScalar(Math.cos(distance/g.radius)).addScaledVector(direction,Math.sin(distance/g.radius)),c);}
      const h=ground(x,z,c)+sample(planet?.relief,x,z,c)+sample(planet?.waterDepth,x,z,c)+.06;
      const q=position(x,z,h,c);vertices.setXYZ(i,q.x,q.y,q.z);
    }
    vertices.needsUpdate=true;this.footprint.visible=true;this.renderDirty=true;
  }
  setPlacement(callback:((x:number,z:number)=>void)|null){this.placement=callback;this.renderer.domElement.style.cursor=callback?'crosshair':'';}
  resetCamera(){
    const g=this.config?geometry(this.config):null;this.cameraMode='overview';this.follow=false;this.controls.autoRotate=false;this.camera.up.set(0,1,0);
    this.controls.maxPolarAngle=g?.sphere?Math.PI:Math.PI*.47;this.camera.near=g?.sphere?Math.max(.1,g.radius*.02):.1;this.camera.far=Math.max(600,(g?.size??64)*8);this.camera.updateProjectionMatrix();
    if(this.config?.kind==='lab'){this.controls.target.set(0,.5,0);this.camera.position.set(4,3.5,5);this.controls.minDistance=1;}
    else{const distance=(this.camera.aspect>1.25?72:95)*(g?.sphere?g.radius/20.37:(g?.size??64)/64);this.controls.minDistance=g?.sphere?g.radius+5:3;this.controls.target.set(0,0,0);this.camera.position.set(.59*distance,.48*distance,.64*distance);}
    this.rebindControls();this.applyCameraMode();
  }
  setOverlay(on:boolean){this.overlay=on;this.terrainDirty=true;if(this.snapshot)this.update(this.snapshot);}
  setQuality(value:string){this.quality=value;this.renderer.setPixelRatio(value==='low'?1:Math.min(devicePixelRatio,1.75));this.renderer.shadowMap.enabled=false;this.resize();}
  private pick(e:PointerEvent){
    const rect=this.renderer.domElement.getBoundingClientRect(),ray=new THREE.Raycaster();
    this.camera.updateMatrixWorld();
    ray.setFromCamera(new THREE.Vector2((e.clientX-rect.left)/rect.width*2-1,-(e.clientY-rect.top)/rect.height*2+1),this.camera);
    if(this.placement){const hit=this.terrainSurface?ray.intersectObject(this.terrainSurface)[0]:undefined;if(hit&&this.config){const[x,z]=coordinates(hit.point,this.config);this.placement(x,z);}return;}
    // A fixed pixel tolerance makes single cells selectable without altering
    // their simulated size. Reject candidates hidden by the planet/terrain.
    const candidates:{id:number;p:THREE.Vector3;r:number;distance:number}[]=[],p=new THREE.Vector3(),projected=new THREE.Vector3();
    const sphere=!!this.config&&geometry(this.config).sphere;
    for(let i=0;i<this.cells.count;i++){
      p.fromBufferAttribute(this.overviewPositions,i);if(sphere&&p.dot(this.camera.position)-p.lengthSq()<=0)continue;projected.copy(p).project(this.camera);if(projected.z< -1||projected.z>1)continue;
      const distance=Math.hypot((projected.x+1)*rect.width/2-(e.clientX-rect.left),(1-projected.y)*rect.height/2-(e.clientY-rect.top));
      const pixelRadius=this.pickRadii[i]*rect.height/(2*Math.tan(THREE.MathUtils.degToRad(this.camera.fov/2))*this.camera.position.distanceTo(p));
      if(distance<=Math.max(12,pixelRadius))candidates.push({id:this.cellIds[i],p:p.clone(),r:this.pickRadii[i],distance});
    }
    candidates.sort((a,b)=>a.distance-b.distance||this.camera.position.distanceToSquared(a.p)-this.camera.position.distanceToSquared(b.p));
    for(const c of candidates.slice(0,12)){
      const ndc=c.p.clone().project(this.camera);ray.setFromCamera(new THREE.Vector2(ndc.x,ndc.y),this.camera);
      const surface=this.terrainSurface?ray.intersectObject(this.terrainSurface)[0]:undefined;
      if(surface&&surface.distance+c.r<this.camera.position.distanceTo(c.p))continue;
      if(this.overviewCells.visible)this.focusLife(c.id,true);this.onSelect(c.id);return;
    }
  }
  private animate=()=>{
    if(!this.alive)return;requestAnimationFrame(this.animate);
    if(document.hidden||!this.snapshot)return;
    const now=performance.now(),interval=1000/30;
    if(now-this.lastDraw<interval)return;
    if(this.painting===null)this.controls.update();
    if(!this.renderDirty)return;
    this.lastDraw=now;this.frame++;
    if(now-this.fpsTime>1000){this.fps=this.frame*1000/(now-this.fpsTime);this.frame=0;this.fpsTime=now;}
    this.renderer.render(this.scene,this.camera);this.renderDirty=false;
  };
  dispose(){this.finishBrush();window.removeEventListener('blur',this.blurBrush);this.alive=false;this.resizeObserver.disconnect();this.controls.dispose();this.scene.traverse(o=>{if(o instanceof THREE.Mesh||o instanceof THREE.Points||o instanceof THREE.Line){o.geometry.dispose();(Array.isArray(o.material)?o.material:[o.material]).forEach(m=>m.dispose());}});this.renderer.dispose();}
}
