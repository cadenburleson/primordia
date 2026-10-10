import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
import { elevation, habitatY, noise, Random } from '../sim/random';
import { STRIDE, CELL_STRIDE, LINK_STRIDE, JOINT_STRIDE, MAX_BODY_CELLS, type Snapshot, type WorldConfig } from '../sim/types';
import {jointGeometry} from './joints';

const MAX_CELLS = 65000;
const temp = new THREE.Object3D(), color = new THREE.Color();
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
  camera = new THREE.PerspectiveCamera(40, 1, .1, 600);
  controls: OrbitControls;
  composer: EffectComposer;
  cells: THREE.InstancedMesh;
  nuclei: THREE.InstancedMesh;
  bones: THREE.InstancedMesh;
  jointCuffs:THREE.InstancedMesh;
  jointMuscles:THREE.InstancedMesh;
  terrain = new THREE.Group();
  water: THREE.Mesh|null=null;
  trace = new THREE.Line(new THREE.BufferGeometry(),new THREE.LineBasicMaterial({color:'#a5d89d',transparent:true,opacity:.6}));
  private motion = new Map<string,{index:number;current:THREE.Vector3;target:THREE.Vector3;radius:number}>();
  private linkMotion:{index:number;a:string;b:string;kind:number;activation:number}[]=[];
  private jointMotion:{a:string;b:string;c:string;flexion:number;extension:number}[]=[];
  private lastMotion=performance.now();
  ring: THREE.Mesh;
  particles: THREE.Points;
  snapshot: Snapshot | null = null;
  cellIds: number[] = [];
  selectedId: number | null = null;
  selected = new THREE.Group();
  follow = false;
  overlay = false;
  quality = 'high';
  resources: THREE.Points | null = null;
  config: WorldConfig | null = null;
  fps = 60;
  private frame = 0;
  private fpsTime = performance.now();
  private alive = true;
  private resizeObserver: ResizeObserver;
  private onSelect: (id: number | null) => void;

  constructor(public host: HTMLElement, onSelect: (id: number | null) => void) {
    this.onSelect = onSelect;
    this.renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance' });
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 1.75));
    this.renderer.setClearColor('#10191d');
    this.scene.background = new THREE.Color('#10191d');
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.host.appendChild(this.renderer.domElement);
    this.scene.fog = new THREE.FogExp2('#10191d', .005);
    this.camera.position.set(55, 45, 60);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.target.set(0, 0, 0);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = .045;
    this.controls.minDistance = 3; this.controls.maxDistance = 120;
    this.controls.maxPolarAngle = Math.PI*.47;
    this.controls.autoRotate = true; this.controls.autoRotateSpeed = .24;
    this.controls.addEventListener('start', () => { this.controls.autoRotate = false; this.follow = false; });
    const ambient = new THREE.HemisphereLight('#d8ffe6', '#172338', 1.8);
    this.scene.add(ambient);
    const sun = new THREE.DirectionalLight('#fff4d8', 2.8); sun.position.set(-25, 45, 15);
    sun.castShadow = true; sun.shadow.mapSize.set(2048,2048);
    Object.assign(sun.shadow.camera, { left: -38, right: 38, top: 38, bottom: -38, near: 1, far: 110 });
    sun.shadow.bias = -.0006; this.scene.add(sun);
    const rim = new THREE.DirectionalLight('#7de8dc', 3); rim.position.set(10,8,-35); this.scene.add(rim);

    this.cells = new THREE.InstancedMesh(new THREE.SphereGeometry(1, 12, 8), new THREE.MeshPhysicalMaterial({
      roughness: .24, metalness: .12, clearcoat: 1, clearcoatRoughness: .15,
      emissive: '#5a775c', emissiveIntensity: .32,
    }), MAX_CELLS);
    this.nuclei = new THREE.InstancedMesh(new THREE.SphereGeometry(1,8,6), new THREE.MeshStandardMaterial({
      roughness: .2, emissive: '#b8f298', emissiveIntensity: 1.2, color: '#dcffc1',
    }), MAX_CELLS);
    this.bones = new THREE.InstancedMesh(new THREE.CylinderGeometry(.06,.06,1,5),new THREE.MeshStandardMaterial({ color:'#dfdabd',roughness:.6 }), 22000);
    this.jointCuffs=new THREE.InstancedMesh(new THREE.TorusGeometry(.23,.026,6,14),new THREE.MeshStandardMaterial({color:'#e7e0bb',roughness:.4}),8000);
    this.jointMuscles=new THREE.InstancedMesh(new THREE.CylinderGeometry(.035,.035,1,6),new THREE.MeshStandardMaterial({roughness:.4}),16000);
    for(const mesh of [this.jointCuffs,this.jointMuscles]){mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);mesh.frustumCulled=false;mesh.count=0;this.scene.add(mesh);}
    this.cells.instanceMatrix.setUsage(THREE.DynamicDrawUsage); this.nuclei.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.bones.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.cells.frustumCulled = false; this.nuclei.frustumCulled = false; this.bones.frustumCulled = false;
    this.cells.count = 0; this.nuclei.count = 0; this.bones.count = 0;
    this.scene.add(this.cells, this.nuclei, this.bones, this.terrain, this.selected,this.trace);
    this.selected.add(new THREE.Mesh(new THREE.TorusGeometry(1,.025,8,64),new THREE.MeshBasicMaterial({ color:'#ebf5be',depthTest:false })));
    this.selected.rotation.x = Math.PI/2; this.selected.visible = false;
    this.ring = new THREE.Mesh(new THREE.TorusGeometry(33,.025,6,200),new THREE.MeshBasicMaterial({ color:'#577569',transparent:true,opacity:.45 }));
    this.ring.rotation.x = Math.PI/2; this.ring.position.y = -2; this.scene.add(this.ring);
    const starPositions = new Float32Array(1800), rng = new Random(911);
    for(let i=0;i<600;i++){const a=rng.between(0,Math.PI*2), r=rng.between(85,180); starPositions[i*3]=Math.cos(a)*r;starPositions[i*3+1]=rng.between(-30,90);starPositions[i*3+2]=Math.sin(a)*r;}
    const stars = new THREE.BufferGeometry(); stars.setAttribute('position',new THREE.BufferAttribute(starPositions,3));
    this.particles = new THREE.Points(stars,new THREE.PointsMaterial({ color:'#c4e4d5',size:.14,transparent:true,opacity:.38 })); this.scene.add(this.particles);
    this.composer = new EffectComposer(this.renderer);
    this.composer.addPass(new RenderPass(this.scene,this.camera));
    this.composer.addPass(new UnrealBloomPass(new THREE.Vector2(800,600),.3,.55,1.2));
    this.composer.addPass(new OutputPass());
    this.resizeObserver = new ResizeObserver(()=>this.resize()); this.resizeObserver.observe(host);
    let downX=0,downY=0;
    this.renderer.domElement.addEventListener('pointerdown',e=>{downX=e.clientX;downY=e.clientY;});
    this.renderer.domElement.addEventListener('pointerup',e=>{if(Math.hypot(e.clientX-downX,e.clientY-downY)<5)this.pick(e);});
    this.resize(); this.animate();
  }
  private resize() {
    const {width,height}=this.host.getBoundingClientRect();
    this.camera.aspect=width/Math.max(1,height);this.camera.updateProjectionMatrix();
    if(this.controls.autoRotate&&!this.follow)this.camera.position.normalize().multiplyScalar(this.camera.aspect>1.25?72:95);
    this.renderer.setSize(width,height);this.composer.setSize(width,height);
  }
  build(config: WorldConfig) {
    this.config=config;
    while(this.terrain.children.length){const child=this.terrain.children[0];this.terrain.remove(child);child.traverse(obj=>{if(obj instanceof THREE.Mesh||obj instanceof THREE.Points){obj.geometry.dispose();const materials=Array.isArray(obj.material)?obj.material:[obj.material];materials.forEach(m=>m.dispose());}});}
    this.water=null;this.resources=null;this.ring.visible=config.kind!=='lab';
    if(config.kind==='lab'){
      const floor=new THREE.Mesh(new THREE.CircleGeometry(31,128),new THREE.MeshStandardMaterial({color:'#25372e',roughness:.9}));
      floor.rotation.x=-Math.PI/2;floor.receiveShadow=true;this.terrain.add(floor);
      const grid=new THREE.GridHelper(60,120,'#526b53','#354c3e');grid.position.y=.002;this.terrain.add(grid);
      const base=new THREE.Mesh(new THREE.CylinderGeometry(31,30,1,128),new THREE.MeshStandardMaterial({color:'#15241d',roughness:.9}));base.position.y=-.51;this.terrain.add(base);
      this.scene.background=new THREE.Color('#10191d');this.resetCamera();return;
    }
    const seed=config.seed, rng=new Random(seed);
    const geo=new THREE.PlaneGeometry(64,64,128,128);geo.rotateX(-Math.PI/2);
    const pos=geo.attributes.position, colors=new Float32Array(pos.count*3);
    const tidal=config.kind==='tidal', alien=config.kind==='alien';
    const low=new THREE.Color(tidal?'#394a3d':alien?'#5f4567':'#47342c');
    const high=new THREE.Color(tidal?'#aab58b':alien?'#ae98b9':'#b7a58e');
    for(let i=0;i<pos.count;i++){
      const x=pos.getX(i),z=pos.getZ(i),h=elevation(x,z,seed);pos.setY(i,h);
      color.copy(low).lerp(high,THREE.MathUtils.clamp((h+.1)/3.5,0,1));
      color.multiplyScalar(.86+noise(x*.8,z*.8,seed)*.22);color.toArray(colors,i*3);
    }
    const indices:number[]=[];const old=geo.index!;
    for(let i=0;i<old.count;i+=3){const ids=[old.getX(i),old.getX(i+1),old.getX(i+2)];if(ids.every(j=>Math.hypot(pos.getX(j),pos.getZ(j))<31))indices.push(...ids);}
    geo.setIndex(indices);geo.setAttribute('color',new THREE.BufferAttribute(colors,3));geo.computeVertexNormals();
    const terrain=new THREE.Mesh(geo,new THREE.MeshStandardMaterial({vertexColors:true,roughness:.96,flatShading:false}));terrain.receiveShadow=true;this.terrain.add(terrain);
    const base=new THREE.Mesh(new THREE.CylinderGeometry(31,28,4,128),new THREE.MeshStandardMaterial({ color:tidal?'#182b28':alien?'#2a1d32':'#241c18',roughness:.9,flatShading:true }));base.position.y=-4.5;this.terrain.add(base);
    const waterGeo=new THREE.CircleGeometry(31.2,160);waterGeo.rotateX(-Math.PI/2);
    this.water=new THREE.Mesh(waterGeo,new THREE.ShaderMaterial({
      transparent:true,depthWrite:false,uniforms:{time:{value:0},waterColor:{value:new THREE.Color(tidal?'#387f76':alien?'#625087':'#804e39')}},
      vertexShader:`varying vec3 vWorld; void main(){vec4 world=modelMatrix*vec4(position,1.);vWorld=world.xyz;gl_Position=projectionMatrix*viewMatrix*world;}`,
      fragmentShader:`uniform float time;uniform vec3 waterColor;varying vec3 vWorld;void main(){float w=sin(vWorld.x*.8+time*.28)*sin(vWorld.z*.65-time*.2);float fine=sin((vWorld.x+vWorld.z)*3.+time*.4);vec3 c=waterColor+vec3(.08,.1,.08)*w*.23+vec3(.06)*pow(max(fine,0.),18.);float glow=pow(max(sin(vWorld.x*.11+vWorld.z*.06+time*.05),0.),12.);gl_FragColor=vec4(c+vec3(.09,.12,.09)*glow,.73);}`,
    }));this.water.position.y=.12;this.terrain.add(this.water);
    const rockGeo=new THREE.IcosahedronGeometry(1,0), rockMat=new THREE.MeshStandardMaterial({color:tidal?'#849484':alien?'#91799e':'#847363',roughness:.88,flatShading:true});
    for(let i=0;i<90;i++){
      const a=rng.between(0,Math.PI*2),r=rng.between(3,29),x=Math.cos(a)*r,z=Math.sin(a)*r,y=elevation(x,z,seed);
      if(y<.12)continue;
      const rock=new THREE.Mesh(rockGeo,rockMat);rock.position.set(x,y+.1,z);rock.scale.set(rng.between(.35,1.4),rng.between(.4,1.6),rng.between(.35,1.2));rock.rotation.set(rng.next(),rng.next()*6,rng.next());rock.castShadow=true;this.terrain.add(rock);
    }
    const deposits=new THREE.InstancedMesh(new THREE.ConeGeometry(.17,1,5),new THREE.MeshStandardMaterial({color:alien?'#ddadf8':tidal?'#b6d1ab':'#efad75',emissive:alien?'#4b215c':tidal?'#1a4939':'#773713',emissiveIntensity:.6,roughness:.25,metalness:.3}),80);
    let d=0;
    for(let i=0;i<80;i++){const a=rng.between(0,Math.PI*2),r=rng.between(1,28),x=Math.cos(a)*r,z=Math.sin(a)*r,y=elevation(x,z,seed);temp.position.set(x,Math.max(.12,y)+.3,z);temp.rotation.set(rng.next()*.4,rng.next()*6,rng.next()*.4);temp.scale.setScalar(.4+rng.next()*config.minerals*1.8);temp.updateMatrix();deposits.setMatrixAt(d++,temp.matrix);}
    this.terrain.add(deposits);
    const resourceGeo=new THREE.BufferGeometry();const positions=new Float32Array(1600*3);const resourceColors=new Float32Array(1600*3);
    for(let i=0;i<1600;i++){const x=(i%40)/40*64-31.2,z=Math.floor(i/40)/40*64-31.2;positions[i*3]=x;positions[i*3+1]=Math.max(.15,elevation(x,z,seed))+.3;positions[i*3+2]=z;}
    resourceGeo.setAttribute('position',new THREE.BufferAttribute(positions,3));resourceGeo.setAttribute('color',new THREE.BufferAttribute(resourceColors,3));
    resourceGeo.setIndex(Array.from({length:1600},(_,i)=>i).filter(i=>Math.hypot(positions[i*3],positions[i*3+2])<29));
    this.resources=new THREE.Points(resourceGeo,new THREE.PointsMaterial({vertexColors:true,size:.55,transparent:true,opacity:.8,depthTest:false}));this.resources.visible=this.overlay;this.terrain.add(this.resources);
    this.scene.fog=new THREE.FogExp2(alien?'#14131e':'#10191d',.005);
    this.scene.background=new THREE.Color(alien?'#14131e':'#10191d');
    this.resetCamera();
  }
  update(snapshot: Snapshot) {
    this.snapshot=snapshot;
    if(!this.config||snapshot.config.seed!==this.config.seed||snapshot.config.kind!==this.config.kind)this.build(snapshot.config);
    let count=0,linkCount=0;this.cellIds.length=0;
    const data=snapshot.data,physical=snapshot.cellData;
    const starts=new Int32Array(snapshot.ids.length).fill(-1),sizes=new Uint16Array(snapshot.ids.length);
    for(let p=0;p<physical.length/CELL_STRIDE;p++){const owner=physical[p*CELL_STRIDE];if(starts[owner]<0)starts[owner]=p;sizes[owner]++;}
    const nodeKeys:string[]=[];const nextMotion=new Map<string,{index:number;current:THREE.Vector3;target:THREE.Vector3;radius:number}>();
    this.linkMotion=[];this.jointMotion=[];
    const quat=new THREE.Quaternion(),axis=new THREE.Vector3(0,1,0);
    for(let i=0;i<snapshot.ids.length&&count<MAX_CELLS;i++){
      const o=i*STRIDE,x=data[o],z=data[o+1],heading=data[o+2],actual=data[o+3],shape=data[o+4],pigment=data[o+5],photo=data[o+6],chemo=data[o+7];
      const worldY=snapshot.lab?.5:habitatY(x,z,.25+shape*.10,snapshot.config.seed);
      const detail=!!snapshot.lab||snapshot.ids[i]===this.selectedId||this.camera.position.distanceTo(new THREE.Vector3(x,worldY,z))<26;
      const displayed=detail?Math.min(actual,snapshot.autonomous?1024:MAX_BODY_CELLS):Math.min(actual,6),start=starts[i];
      quat.setFromAxisAngle(axis,-heading);
      for(let c=0;c<displayed&&count<MAX_CELLS;c++){
        const isPhysical=start>=0&&c<sizes[i];let role=0,health=1;
        const r=isPhysical?physical[(start+c)*CELL_STRIDE+4]:.25+shape*.10;
        if(isPhysical){
          const p=(start+c)*CELL_STRIDE;role=physical[p+5];health=physical[p+7];
          const target=new THREE.Vector3(physical[p+1],physical[p+2],physical[p+3]);
          const key=`${snapshot.ids[i]}:${c}`,previous=this.motion.get(key);
          const current=previous?.current??target.clone();nodeKeys[start+c]=key;
          nextMotion.set(key,{index:count,current,target,radius:r});temp.position.copy(current);
        }else{
          const offset=bodyOffset(c,displayed,shape,data[o+8]).applyQuaternion(quat);
          temp.position.set(x+offset.x,worldY+offset.y,z+offset.z);
        }
        temp.rotation.set(0,0,0);temp.scale.setScalar(r);temp.updateMatrix();this.cells.setMatrixAt(count,temp.matrix);
        color.setHSL(.30+pigment*.42,.35+chemo*.3,.48+photo*.16);
        if(role===1)color.set('#d8e5c4');if(role===2)color.set('#d88e72');if(role===3)color.set('#83c7d1');if(role===4)color.set('#ddb76b');if(role===5)color.set('#b5a2d9');color.lerp(new THREE.Color('#653e48'),(1-health)*.8);
        this.cells.setColorAt(count,color);temp.scale.multiplyScalar(.3);temp.position.y+=r*.55;temp.updateMatrix();this.nuclei.setMatrixAt(count,temp.matrix);
        this.cellIds[count]=snapshot.ids[i];count++;
      }
      if(snapshot.ids[i]===this.selectedId){
        this.selected.position.set(x,snapshot.lab?.025:Math.max(.16,elevation(x,z,snapshot.config.seed))+.04,z);
        this.selected.scale.setScalar(.65+Math.sqrt(actual)*.3);this.selected.visible=true;
        if(this.follow)this.controls.target.lerp(new THREE.Vector3(x,worldY,z),.15);
      }
    }
    this.motion=nextMotion;
    for(let p=0;p<snapshot.linkData.length;p+=LINK_STRIDE){
      const a=nodeKeys[snapshot.linkData[p]],b=nodeKeys[snapshot.linkData[p+1]];
      if(!a||!b||linkCount>=22000)continue;
      const kind=snapshot.linkData[p+2],activation=snapshot.linkData[p+3];
      this.linkMotion.push({index:linkCount,a,b,kind,activation});
      color.set(kind===2?'#e68c74':kind===1?'#e9e1bb':kind===3?'#7b9b9b':'#567c60');
      if(snapshot.linkData[p+5]<.15)color.set('#713946');
      if(kind===2)color.lerp(new THREE.Color('#ffdb9e'),activation*.7);
      this.bones.setColorAt(linkCount++,color);
    }
    this.cells.count=count;this.nuclei.count=count;this.bones.count=linkCount;
    for(let p=0;p<snapshot.jointData.length&&this.jointMotion.length<8000;p+=JOINT_STRIDE){
      const a=nodeKeys[snapshot.jointData[p]],b=nodeKeys[snapshot.jointData[p+1]],c=nodeKeys[snapshot.jointData[p+2]];
      if(!a||!b||!c)continue;
      const index=this.jointMotion.length,flexion=snapshot.jointData[p+6],extension=snapshot.jointData[p+7];
      this.jointMotion.push({a,b,c,flexion,extension});
      for(let k=0;k<2;k++){color.set(k===0?'#d98569':'#b67f97').lerp(new THREE.Color('#ffe1a3'),(k===0?flexion:extension)*.7);this.jointMuscles.setColorAt(index*2+k,color);}
    }
    this.jointCuffs.count=this.jointMotion.length;this.jointMuscles.count=this.jointMotion.length*2;
    if(this.jointMuscles.instanceColor)this.jointMuscles.instanceColor.needsUpdate=true;
    this.animateBodies(.15);
    if(this.cells.instanceColor)this.cells.instanceColor.needsUpdate=true;
    if(this.bones.instanceColor)this.bones.instanceColor.needsUpdate=true;
    if(this.selectedId&&!snapshot.ids.includes(this.selectedId))this.selected.visible=false;
    if(this.resources&&this.overlay){const colors=this.resources.geometry.attributes.color;for(let i=0;i<snapshot.field.length;i++){color.setHSL(.12+Math.min(1,snapshot.field[i]/12)*.26,.7,.4+Math.min(1,snapshot.field[i]/12)*.15);colors.setXYZ(i,color.r,color.g,color.b);}colors.needsUpdate=true;}
    this.trace.visible=!!snapshot.lab;
    if(snapshot.lab){this.trace.geometry.dispose();this.trace.geometry=new THREE.BufferGeometry().setFromPoints(snapshot.lab.trace.map(p=>new THREE.Vector3(p[0],.04,p[2])));}
  }
  private animateBodies(alpha:number){
    for(const item of this.motion.values()){
      item.current.lerp(item.target,alpha);temp.position.copy(item.current);temp.rotation.set(0,0,0);temp.scale.setScalar(item.radius);temp.updateMatrix();this.cells.setMatrixAt(item.index,temp.matrix);
      temp.position.y+=item.radius*.55;temp.scale.multiplyScalar(.3);temp.updateMatrix();this.nuclei.setMatrixAt(item.index,temp.matrix);
    }
    for(const link of this.linkMotion){
      const a=this.motion.get(link.a)!.current,b=this.motion.get(link.b)!.current;
      const delta=b.clone().sub(a);temp.position.copy(a).add(b).multiplyScalar(.5);
      temp.quaternion.setFromUnitVectors(new THREE.Vector3(0,1,0),delta.clone().normalize());
      const thickness=link.kind===2?.8+link.activation*.5:link.kind===1?.7:.3;
      temp.scale.set(thickness,delta.length(),thickness);temp.updateMatrix();this.bones.setMatrixAt(link.index,temp.matrix);
    }
    for(let i=0;i<this.jointMotion.length;i++){
      const j=this.jointMotion[i],a=this.motion.get(j.a)!.current,b=this.motion.get(j.b)!.current,c=this.motion.get(j.c)!.current;
      const {axis,bands}=jointGeometry(a,b,c);
      temp.position.copy(b);temp.quaternion.setFromUnitVectors(new THREE.Vector3(0,0,1),axis);temp.scale.setScalar(1);temp.updateMatrix();this.jointCuffs.setMatrixAt(i,temp.matrix);
      for(let k=0;k<2;k++){
        const [from,to]=bands[k],delta=to.clone().sub(from),activation=k===0?j.flexion:j.extension;
        temp.position.copy(from).add(to).multiplyScalar(.5);temp.quaternion.setFromUnitVectors(new THREE.Vector3(0,1,0),delta.clone().normalize());temp.scale.set(1+activation*.5,delta.length(),1+activation*.5);temp.updateMatrix();this.jointMuscles.setMatrixAt(i*2+k,temp.matrix);
      }
    }
    this.jointCuffs.instanceMatrix.needsUpdate=true;this.jointMuscles.instanceMatrix.needsUpdate=true;
    this.cells.instanceMatrix.needsUpdate=true;this.nuclei.instanceMatrix.needsUpdate=true;this.bones.instanceMatrix.needsUpdate=true;
  }
  select(id: number|null,zoom=false) {
    this.selectedId=id;this.selected.visible=false;
    if(this.snapshot){this.update(this.snapshot);if(zoom&&id){const i=this.snapshot.ids.indexOf(id),o=i*STRIDE;if(i>=0){const point=new THREE.Vector3(this.snapshot.data[o],this.snapshot.lab?.5:habitatY(this.snapshot.data[o],this.snapshot.data[o+1],.25+this.snapshot.data[o+4]*.10,this.snapshot.config.seed),this.snapshot.data[o+1]);this.controls.target.copy(point);this.camera.position.copy(point).add(new THREE.Vector3(4,3.5,5));this.follow=true;this.controls.autoRotate=false;}}}
  }
  resetCamera(){if(this.config?.kind==='lab'){this.controls.target.set(0,.5,0);this.camera.position.set(4,3.5,5);this.controls.autoRotate=false;this.follow=true;return;}const distance=this.camera.aspect>1.25?72:95;this.controls.target.set(0,0,0);this.camera.position.set(.59*distance,.48*distance,.64*distance);this.controls.autoRotate=true;this.follow=false;}
  setOverlay(on:boolean){this.overlay=on;if(this.resources)this.resources.visible=on;}
  setQuality(value:string){this.quality=value;this.renderer.setPixelRatio(value==='low'?1:Math.min(devicePixelRatio,1.75));this.renderer.shadowMap.enabled=value==='high';this.resize();}
  private pick(e:PointerEvent){
    const rect=this.renderer.domElement.getBoundingClientRect(),ray=new THREE.Raycaster();
    ray.setFromCamera(new THREE.Vector2((e.clientX-rect.left)/rect.width*2-1,-(e.clientY-rect.top)/rect.height*2+1),this.camera);
    const hits=ray.intersectObject(this.cells);
    if(hits[0]?.instanceId!==undefined)this.onSelect(this.cellIds[hits[0].instanceId]);
  }
  private animate=()=>{
    if(!this.alive)return;requestAnimationFrame(this.animate);
    const now=performance.now();this.frame++;
    this.animateBodies(1-Math.exp(-Math.min(.1,(now-this.lastMotion)/1000)*14));this.lastMotion=now;
    if(now-this.fpsTime>1000){this.fps=this.frame*1000/(now-this.fpsTime);this.frame=0;this.fpsTime=now;}
    if(this.water)(this.water.material as THREE.ShaderMaterial).uniforms.time.value=now*.001;
    this.particles.rotation.y=now*.000006;this.controls.update();
    if(this.quality==='high')this.composer.render();else this.renderer.render(this.scene,this.camera);
  };
  dispose(){this.alive=false;this.resizeObserver.disconnect();this.controls.dispose();this.composer.dispose();this.scene.traverse(o=>{if(o instanceof THREE.Mesh||o instanceof THREE.Points){o.geometry.dispose();(Array.isArray(o.material)?o.material:[o.material]).forEach(m=>m.dispose());}});this.renderer.dispose();}
}
