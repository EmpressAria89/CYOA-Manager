(function(){
'use strict';
const delay=ms=>new Promise(r=>setTimeout(r,ms));
window.__cyoaManagerDock=()=>{
 let host=document.getElementById('manager-quick-tools');if(host)return host.shadowRoot;
 host=document.createElement('div');host.id='manager-quick-tools';
 host.style.cssText='all:initial!important;position:fixed!important;right:8px!important;bottom:56px!important;z-index:2147483646!important;max-width:calc(100vw - 16px)!important';
 const root=host.attachShadow({mode:'open'});
 root.innerHTML=`<style>
 :host{font:400 14px/1.4 system-ui;--surface:#313244;--text:#cdd6f4;--border:#585b70;--base:#1e1e2e;color:var(--text)}
 *{box-sizing:border-box}nav{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:6px;padding:6px;background:var(--base);border:1px solid var(--border);border-radius:12px;box-shadow:0 3px 14px #0005}
 button{font:400 14px/1.4 system-ui;min-height:40px;background:var(--surface);color:var(--text);border:1px solid var(--border);border-radius:8px;padding:8px 12px;cursor:pointer}
 button:focus-visible,input:focus-visible,select:focus-visible{outline:2px solid #89b4fa;outline-offset:2px}button[aria-pressed=true]{border-color:#a6e3a1}button:disabled{opacity:.5}
 #manager-display-panel{font:400 14px/1.5 system-ui;color:var(--text);position:fixed;right:8px;bottom:var(--manager-panel-bottom,116px);width:min(430px,calc(100vw - 16px));max-height:calc(100vh - var(--manager-panel-bottom,116px) - 8px);overflow:auto;padding:20px;background:var(--base);border:1px solid var(--border);border-radius:12px;box-shadow:0 10px 30px #0006}
 [hidden]{display:none!important}h2{font:500 19px system-ui;margin:0 0 16px}label{display:flex;flex-direction:column;gap:6px;margin:12px 0}input,select{width:100%;font:400 14px system-ui;background:var(--surface);color:var(--text);border:1px solid var(--border);border-radius:7px;padding:9px;color-scheme:var(--scheme,dark)}input[type=checkbox]{width:20px;height:20px}.reader-note,[role=status]{font-size:13px;line-height:1.5}.reader-actions{display:flex;gap:8px;flex-wrap:wrap;margin-top:16px}
 </style><nav aria-label="CYOA tools"></nav>`;
 const resize=()=>{const bottom=root.querySelector('nav').getBoundingClientRect().height+(parseFloat(host.style.bottom)||56)+8;document.documentElement.style.setProperty('--manager-panel-bottom',bottom+'px');};
 root.addEventListener('manager-dock-layout',resize);
 const observer=new ResizeObserver(resize);observer.observe(host);window.addEventListener('pagehide',()=>observer.disconnect(),{once:true});
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
 const colors={mocha:['#313244','#cdd6f4','#585b70','#1e1e2e'],latte:['#ccd0da','#4c4f69','#9ca0b0','#eff1f5'],frappe:['#414559','#c6d0f5','#737994','#303446'],macchiato:['#363a4f','#cad3f5','#6e738d','#24273a']}[session.theme]||['#313244','#cdd6f4','#585b70','#1e1e2e'];
 host.style.setProperty('--scheme',session.theme==='latte'?'light':'dark');
 ['--surface','--text','--border','--base'].forEach((key,i)=>host.style.setProperty(key,colors[i]));
 const nav=shadow.querySelector('nav');
 function add(text,title,action){const button=document.createElement('button');button.textContent=text;button.title=title;button.onclick=action;nav.append(button);return button;}
 add('Top','Go to top',()=>window.scrollTo({top:0,behavior:matchMedia('(prefers-reduced-motion: reduce)').matches?'auto':'smooth'})).setAttribute('aria-label','Go to top');
 add('Bottom','Go to bottom',()=>window.scrollTo({top:document.scrollingElement.scrollHeight,behavior:matchMedia('(prefers-reduced-motion: reduce)').matches?'auto':'smooth'})).setAttribute('aria-label','Go to bottom');
 if(window.__cyoaManagerPresentation)add('Display','Reader display',()=>window.__cyoaManagerPresentation.open().catch(console.error));
 const search=document.createElement('button');search.textContent='Search choices';search.title='Search choices';shadow.querySelector('nav').append(search);
 search.onclick=async()=>{
  // Forward the native Search Choice command, preserving its search semantics.
  let target=[...document.querySelectorAll('button,[role=button],.v-list-item,.sidebar-menu-item,span,div')].find(e=>/^Search Choice$/i.test(e.childElementCount? '':e.textContent.trim())||/^Search choices$/i.test(e.getAttribute('aria-label')||''));
  if(!target){
   const toggle=document.querySelector('[aria-label="Open Import Window"], [aria-label="Open Menu"], [title="Open Menu"]');toggle?.click();await delay(80);
   target=[...document.querySelectorAll('button,[role=button],.v-list-item,.sidebar-menu-item,span,div')].find(e=>/^Search Choice$/i.test(e.childElementCount? '':e.textContent.trim()));
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
