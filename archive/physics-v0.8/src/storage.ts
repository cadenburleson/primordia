const dbPromise = new Promise<IDBDatabase>((resolve,reject)=>{
  const request=indexedDB.open('primordia',1);
  request.onupgradeneeded=()=>request.result.createObjectStore('worlds');
  request.onsuccess=()=>resolve(request.result);
  request.onerror=()=>reject(request.error);
});
export async function storeWorld(json:string,key='last'){
  const db=await dbPromise;
  return new Promise<void>((resolve,reject)=>{const tx=db.transaction('worlds','readwrite');tx.objectStore('worlds').put(json,key);tx.oncomplete=()=>resolve();tx.onerror=()=>reject(tx.error);});
}
export async function readWorld(key='last'):Promise<string|null>{
  const db=await dbPromise;
  return new Promise((resolve,reject)=>{const req=db.transaction('worlds','readonly').objectStore('worlds').get(key);req.onsuccess=()=>resolve(typeof req.result==='string'?req.result:null);req.onerror=()=>reject(req.error);});
}
