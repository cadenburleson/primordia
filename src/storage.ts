const dbPromise = new Promise<IDBDatabase>((resolve,reject)=>{
  const request=indexedDB.open('primordia');
  request.onupgradeneeded=()=>{if(!request.result.objectStoreNames.contains('worlds'))request.result.createObjectStore('worlds');};
  request.onerror=()=>reject(request.error);
  request.onsuccess=()=>{request.result.onversionchange=()=>request.result.close();resolve(request.result);};
});
export async function storeWorld(json:string,key='last'){
  const db=await dbPromise;
  return new Promise<void>((resolve,reject)=>{const tx=db.transaction('worlds','readwrite');tx.objectStore('worlds').put(json,key);tx.oncomplete=()=>resolve();tx.onerror=()=>reject(tx.error);});
}
import type {SavedSpecimen} from './sim/types';
// Separate database avoids blocking live world tabs during a schema upgrade.
const collectionDb=new Promise<IDBDatabase>((resolve,reject)=>{
  const request=indexedDB.open('primordia-organism-collection',1);
  request.onupgradeneeded=()=>request.result.createObjectStore('specimens');
  request.onsuccess=()=>{request.result.onversionchange=()=>request.result.close();resolve(request.result);};request.onerror=()=>reject(request.error);
});
export async function listSpecimens():Promise<SavedSpecimen[]>{
  const db=await collectionDb;
  return new Promise((resolve,reject)=>{const tx=db.transaction('specimens','readonly'),req=tx.objectStore('specimens').getAll();tx.oncomplete=()=>resolve(req.result);tx.onerror=()=>reject(tx.error);});
}
export async function storeSpecimen(s:SavedSpecimen){
  const db=await collectionDb;return new Promise<void>((resolve,reject)=>{const tx=db.transaction('specimens','readwrite');tx.objectStore('specimens').put(s,s.id);tx.oncomplete=()=>resolve();tx.onerror=()=>reject(tx.error);});
}
export async function deleteSpecimen(id:string){
  const db=await collectionDb;return new Promise<void>((resolve,reject)=>{const tx=db.transaction('specimens','readwrite');tx.objectStore('specimens').delete(id);tx.oncomplete=()=>resolve();tx.onerror=()=>reject(tx.error);});
}
export async function readWorld(key='last'):Promise<string|null>{
  const db=await dbPromise;
  return new Promise((resolve,reject)=>{const req=db.transaction('worlds','readonly').objectStore('worlds').get(key);req.onsuccess=()=>resolve(typeof req.result==='string'?req.result:null);req.onerror=()=>reject(req.error);});
}
