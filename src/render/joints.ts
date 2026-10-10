import * as THREE from 'three';

/** Visible cuffs and paired tendon bands follow solved cell positions. Angular
 * force is solved in Rust; these bands are a diagram of its attachment levers. */
export function jointGeometry(a:THREE.Vector3,b:THREE.Vector3,c:THREE.Vector3){
  const u=a.clone().sub(b),v=c.clone().sub(b),axis=u.clone().cross(v);
  if(axis.lengthSq()<1e-6)axis.copy(u).cross(Math.abs(u.y)<.8*u.length()?new THREE.Vector3(0,1,0):new THREE.Vector3(1,0,0));
  axis.normalize();
  const pa=axis.clone().cross(u).normalize().multiplyScalar(.13),pc=axis.clone().cross(v).normalize().multiplyScalar(.13);
  const left=b.clone().addScaledVector(u,.5),right=b.clone().addScaledVector(v,.5);
  return {axis,bands:[
    [left.clone().add(pa),right.clone().add(pc)],
    [left.clone().sub(pa),right.clone().sub(pc)],
  ] as [THREE.Vector3,THREE.Vector3][]};
}
