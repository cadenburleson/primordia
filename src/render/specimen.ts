import * as THREE from 'three';
import { GI, type OrganismView } from '../sim/types';
import {jointGeometry} from './joints';
import {CELL_TYPES,CHOSEN_COLOR,cellType,isChosen} from '../sim/roles';
import type {PhysicalJoint} from '../sim/types';

/** Inspection uses the same solver state as the world, including actual links. */
export class SpecimenView {
  renderer:THREE.WebGLRenderer;
  scene=new THREE.Scene();camera=new THREE.PerspectiveCamera(35,1,.1,50);body=new THREE.Group();
  signature='';alive=true;observer:ResizeObserver;
  /** The physical world uses material cell types; older labs use tissue roles. */
  physics=false;
  private cells:THREE.Mesh[]=[];private links:THREE.Mesh[]=[];private contacts:THREE.Mesh[]=[];
  private targets:THREE.Vector3[]=[];private linkEnds:[number,number][]=[];
  private radius=.7;
  private dirty=true;
  private lastDraw=0;
  private jointVisuals:{ring:THREE.Mesh;bands:THREE.Mesh[]}[]=[];
  private joints:PhysicalJoint[]=[];
  constructor(private host:HTMLElement){
    this.renderer=new THREE.WebGLRenderer({alpha:true,antialias:false});this.renderer.setPixelRatio(1);
    this.renderer.toneMapping=THREE.NoToneMapping;host.append(this.renderer.domElement);
    this.scene.add(new THREE.HemisphereLight('#dcffce','#374b58',3));const light=new THREE.DirectionalLight('#f8ecd3',4);light.position.set(-3,4,5);this.scene.add(light);this.scene.add(this.body);
    this.observer=new ResizeObserver(()=>{const w=host.clientWidth,h=host.clientHeight;this.renderer.setSize(w,h);this.camera.aspect=w/Math.max(1,h);this.fitCamera();this.dirty=true;});this.observer.observe(host);
    this.camera.position.set(0,1,5);this.camera.lookAt(0,0,0);this.animate();
  }
  update(o:OrganismView|null){
    this.dirty=true;
    const nodes=(o?.body.nodes??[]).slice(0,256),links=(o?.body.links??[]).filter(l=>l.a<256&&l.b<256);
    const visibleJoints=(o?.body.joints??[]).filter(j=>j.a<256&&j.b<256&&j.c<256);
    const n=o?(nodes.length||1):0;
    const signature=o?`${o.id}:${n}:${nodes.map(n=>n.role).join(',')}:${links.map(l=>`${l.a},${l.b},${l.kind}`).join(';')}:${visibleJoints.map(j=>`${j.a},${j.b},${j.c}`).join(';')}`:'';
    if(signature!==this.signature){
      this.signature=signature;
      this.body.traverse(child=>{if(child instanceof THREE.Mesh){child.geometry.dispose();(child.material as THREE.Material).dispose();}});
      this.body.clear();this.cells=[];this.links=[];this.contacts=[];this.linkEnds=[];this.targets=[];this.jointVisuals=[];
      for(let i=0;i<n;i++){
        const raw=nodes[i]?.role??0,role=this.physics?cellType(raw):raw;
        const color=new THREE.Color().setHSL(.30+(o?.genes[GI.pigment]??.5)*.42,.5,.6);
        if(this.physics){if(role>0&&role<CELL_TYPES.length)color.set(CELL_TYPES[role].color);if(isChosen(raw))color.lerp(new THREE.Color(CHOSEN_COLOR),.5);}
        else{if(role===1)color.set('#d8e5c4');if(role===2)color.set('#d88e72');if(role===3)color.set('#83c7d1');if(role===4)color.set('#ddb76b');if(role===5)color.set('#b5a2d9');}
        const cell=new THREE.Mesh(new THREE.SphereGeometry(nodes[i]?.radius??.26,12,8),new THREE.MeshStandardMaterial({color,roughness:1,metalness:0}));
        this.cells.push(cell);this.body.add(cell);
        const contact=new THREE.Mesh(new THREE.TorusGeometry(.22,.015,5,24),new THREE.MeshBasicMaterial({color:'#c6ef9e',transparent:true,opacity:.7}));contact.rotation.x=Math.PI/2;this.contacts.push(contact);this.body.add(contact);
      }
      for(const l of links){
        const link=new THREE.Mesh(new THREE.CylinderGeometry(l.kind===2?.055:l.kind===1?.035:.014,l.kind===2?.055:l.kind===1?.035:.014,1,8),new THREE.MeshStandardMaterial({color:l.kind===2?'#db8a70':l.kind===1?'#eee1bb':'#6a9792',roughness:.5}));
        this.links.push(link);this.linkEnds.push([l.a,l.b]);this.body.add(link);
      }
      for(const _j of visibleJoints){
        const ring=new THREE.Mesh(new THREE.TorusGeometry(.23,.026,8,20),new THREE.MeshStandardMaterial({color:'#e7e0bb',metalness:.25,roughness:.4}));
        const bands=[0,1].map(()=>new THREE.Mesh(new THREE.CylinderGeometry(.035,.035,1,8),new THREE.MeshStandardMaterial({color:'#da876e',roughness:.4})));
        this.body.add(ring,...bands);this.jointVisuals.push({ring,bands});
      }
    }
    this.joints=visibleJoints;
    if(!o)return;
    const center=new THREE.Vector3();for(const node of nodes)center.add(new THREE.Vector3(node.p.x,node.p.y,node.p.z));if(nodes.length)center.multiplyScalar(1/nodes.length);
    this.targets=nodes.length?nodes.map(node=>new THREE.Vector3(node.p.x,node.p.y,node.p.z).sub(center)):[new THREE.Vector3()];
    for(let i=0;i<n;i++){
      this.cells[i].position.copy(this.targets[i]);
      this.contacts[i].visible=nodes[i]?.contact??false;
      this.contacts[i].position.copy(this.targets[i]);this.contacts[i].position.y-=nodes[i]?.radius??.26;
    }
    for(let i=0;i<links.length;i++){
      this.links[i].visible=links[i].integrity>.08;
      const material=this.links[i].material as THREE.MeshStandardMaterial;
      if(links[i].kind===2){material.color.set('#db8a70').lerp(new THREE.Color('#ffd799'),links[i].activation);material.emissive.set('#ad482a');material.emissiveIntensity=links[i].activation*.35;}
    }
    this.radius=Math.max(.7,...this.targets.map((p,i)=>p.length()+(nodes[i]?.radius??.26)+.04));
    this.fitCamera();
  }
  private fitCamera(){
    const vertical=THREE.MathUtils.degToRad(this.camera.fov/2);
    const horizontal=Math.atan(Math.tan(vertical)*Math.max(.1,this.camera.aspect));
    const distance=1.12*this.radius/Math.sin(Math.min(vertical,horizontal));
    this.camera.position.copy(new THREE.Vector3(.6,.4,.9).normalize().multiplyScalar(distance));
    this.camera.far=Math.max(50,distance+this.radius*2);this.camera.lookAt(0,0,0);this.camera.updateProjectionMatrix();
  }
  animate=()=>{
    if(!this.alive)return;requestAnimationFrame(this.animate);
    if(document.hidden||this.host.clientWidth===0||this.host.clientHeight===0||!this.dirty||performance.now()-this.lastDraw<1000/15)return;
    this.lastDraw=performance.now();
    for(let i=0;i<this.links.length;i++){
      const [a,b]=this.linkEnds[i],from=this.cells[a].position,to=this.cells[b].position,delta=to.clone().sub(from);
      this.links[i].position.copy(from).add(to).multiplyScalar(.5);this.links[i].scale.set(1,delta.length(),1);
      this.links[i].quaternion.setFromUnitVectors(new THREE.Vector3(0,1,0),delta.normalize());
    }
    for(let i=0;i<this.joints.length;i++){
      const j=this.joints[i],visual=this.jointVisuals[i];
      visual.ring.visible=j.enabled;visual.bands.forEach(b=>b.visible=j.enabled);
      const a=this.cells[j.a].position,b=this.cells[j.b].position,c=this.cells[j.c].position;
      const {axis,bands}=jointGeometry(a,b,c);visual.ring.position.copy(b);visual.ring.quaternion.setFromUnitVectors(new THREE.Vector3(0,0,1),axis);
      for(let k=0;k<2;k++){
        const [from,to]=bands[k],delta=to.clone().sub(from),mesh=visual.bands[k],activation=k===0?j.flexion:j.extension;
        mesh.position.copy(from).add(to).multiplyScalar(.5);mesh.quaternion.setFromUnitVectors(new THREE.Vector3(0,1,0),delta.clone().normalize());mesh.scale.set(1+activation*.5,delta.length(),1+activation*.5);
        const material=mesh.material as THREE.MeshStandardMaterial;material.color.set(k===0?'#d98569':'#b67f97').lerp(new THREE.Color('#ffe1a3'),activation*.7);
      }
    }
    this.renderer.render(this.scene,this.camera);this.dirty=false;
  };
  dispose(){this.alive=false;this.observer.disconnect();this.body.traverse(o=>{if(o instanceof THREE.Mesh){o.geometry.dispose();(o.material as THREE.Material).dispose();}});this.renderer.dispose();}
}
