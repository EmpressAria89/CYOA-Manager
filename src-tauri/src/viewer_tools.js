(function(){
'use strict';
const delay=ms=>new Promise(r=>setTimeout(r,ms));
window.__cyoaManagerDock=()=>{
 let host=document.getElementById('manager-quick-tools');if(host)return host.shadowRoot;
 host=document.createElement('div');host.id='manager-quick-tools';
 host.style.cssText='all:initial!important;position:fixed!important;right:48px!important;bottom:8px!important;z-index:2147483646!important;max-width:calc(100vw - 64px)!important';
 const root=host.attachShadow({mode:'open'});
 root.innerHTML='<style>:host{font:400 13px system-ui;--surface:#313244;--text:#cdd6f4;--border:#585b70}nav{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:8px}button{font:inherit;font-weight:400;background:var(--surface);color:var(--text);border:1px solid var(--border);border-radius:8px;padding:7px 12px;cursor:pointer;box-shadow:0 2px 10px #0004}button:focus-visible{outline:2px solid #89b4fa;outline-offset:2px}button[aria-pressed=true]{border-color:#a6e3a1}button:disabled{opacity:.5}</style><nav aria-label="CYOA tools"></nav>';
 document.body.append(host);return root;
};
function rootVue(){const root=document.getElementById('app');return window.app?.__vue__||root?.__vue__||root?.firstElementChild?.__vue__;}
function findVue(vm,test){if(!vm)return null;if(test(vm))return vm;for(const child of vm.$children||[]){const found=findVue(child,test);if(found)return found;}return null;}
async function originalImporter(){
 const root=rootVue();if(!root)return null;
 const main=findVue(root,vm=>'currentComponent'in vm);
 if(main){main.currentComponent='appActivatedViewer';await main.$nextTick();}
 return findVue(root,vm=>typeof vm.cleanActivated==='function'&&'newActivated'in vm);
}
async function native(){
 for(let i=0;i<80;i++){
  if(window.__cyoaManagerNative){
   if(window.__cyoaManagerNative.open&&!window.__cyoaManagerImport){window.__cyoaManagerNative.open();await delay(60);}
   const adapter=window.__cyoaManagerImport||window.__cyoaManagerNative;
   if(adapter.ready?.()===false){await delay(100);continue;}
   return adapter;
  }
  if(rootVue())return null;
  await delay(100);
 }
 throw new Error('The viewer has not finished loading. Try again after its choices appear.');
}
window.__cyoaManagerAdapter={
 async capture(){const adapter=await native();if(adapter?.capture){const code=adapter.capture();window.__cyoaManagerNative.close?.();return code;}const app=rootVue()?.$store?.state?.app;return Array.isArray(app?.activated)?app.activated.join(','):null;},
 async load(code){const adapter=await native();if(adapter?.load){adapter.load(code);window.__cyoaManagerNative.close?.();return true;}const vm=await originalImporter();if(!vm)return false;vm.newActivated=code;vm.cleanActivated();return true;},
 async legacy(){
  for(let i=0;i<100;i++){const response=await(await fetch('/__manager_legacy')).json();if(response.ready)return response.builds;await delay(100);}
  throw new Error('Legacy recovery is still loading. Reopen Builds to retry; original saves remain untouched.');
 }
};
async function start(){
 const session=await(await fetch('/__manager_session')).json();

 const shadow=window.__cyoaManagerDock(), host=shadow.host;
 const colors={mocha:['#313244','#cdd6f4','#585b70'],latte:['#ccd0da','#4c4f69','#9ca0b0'],frappe:['#414559','#c6d0f5','#737994'],macchiato:['#363a4f','#cad3f5','#6e738d']}[session.theme]||['#313244','#cdd6f4','#585b70'];
 ['--surface','--text','--border'].forEach((key,i)=>host.style.setProperty(key,colors[i]));
 const search=document.createElement('button');search.textContent='Search choices';search.title='Search choices';shadow.querySelector('nav').append(search);
 search.onclick=async()=>{
  // Forward the native Search Choice command, preserving its search semantics.
  let target=[...document.querySelectorAll('button,[role=button],.v-list-item,.sidebar-menu-item,span,div')].find(e=>/^Search Choice$/i.test(e.textContent.trim())||/^Search choices$/i.test(e.getAttribute('aria-label')||''));
  if(!target){
   const toggle=document.querySelector('[aria-label="Open Import Window"], [aria-label="Open Menu"], [title="Open Menu"]');toggle?.click();await delay(80);
   target=[...document.querySelectorAll('button,[role=button],.v-list-item,.sidebar-menu-item,span,div')].find(e=>/^Search Choice$/i.test(e.textContent.trim()));
  }
  if(target){target.click();return;}
  // Formats without a native search command get local, non-destructive text matching.
  let query=window.prompt('Find text in this CYOA:');if(!query)return;
  const selected=[...document.querySelectorAll('h1,h2,h3,h4,.v-card__title,.choice-title,.perk-title')].find(e=>e.textContent.toLowerCase().includes(query.toLowerCase()));
  if(selected){selected.scrollIntoView({block:'center'});selected.style.outline='2px solid #89b4fa';setTimeout(()=>selected.style.outline='',2500);}else window.alert('No matching choice title found.');
 };

}
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',()=>start().catch(console.error),{once:true});else start().catch(console.error);
})();
