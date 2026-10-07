import {invoke} from "@tauri-apps/api/core";
const sources=new Map<string,Promise<string|null>>();
export function resolveCover(filePath:string,coverImage:string|null):Promise<string|null>{
 const key=JSON.stringify([filePath,coverImage]);let pending=sources.get(key);
 if(!pending){pending=invoke<string|null>("resolve_cover_image_src",{filePath,coverImage}).catch(()=>null);sources.set(key,pending);if(sources.size>512)sources.delete(sources.keys().next().value!);}return pending;
}
