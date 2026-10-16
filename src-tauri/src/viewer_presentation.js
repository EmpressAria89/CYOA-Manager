// Live display preferences. No game model, selection, requirement or export writes.
(() => {
 'use strict';
 let config={global:{parts:{}},local:{parts:{}},effective:{parts:{}}}, rootSize=16, panel=null;
 const originals=new WeakMap(), cards=new WeakMap(), faces=new Map();
 const partNames={body:'Whole CYOA',title:'Choice titles',description:'Descriptions',score:'Costs and gains',layout:'Layout and colors',sections:'Hide sections'};
 const state=()=>config.effective||{parts:{}};
 const body=()=>state().parts.body||{font:'',size:100};
 function face(font){
  if(!font)return '';
  if(!faces.has(font)){
   const name='Manager Reader '+faces.size, node=document.createElement('style');
   node.textContent=`@font-face{font-family:'${name}';src:url('cyoafont://localhost/${encodeURIComponent(font)}');font-weight:400;font-style:normal;font-display:swap;}`;
   document.head.append(node);faces.set(font,name);
  }
  return `'${faces.get(font)}',sans-serif`;
 }
 function applyText(element,record){
  element.style.cssText=record.css;
  const selected=state().parts[record.part]||body(), font=selected.font||record.font;
  if(font){element.style.setProperty('font-family',selected.font?face(selected.font):record.font,'important');}
  if(selected.font)element.style.setProperty('font-weight','400','important');
  const size=selected.size||100, rootFactor=(body().size||100)/100;
  const match=record.size.match(/^([\d.]+)(%|px|em|rem)$/);
  if(match){const value=Number(match[1])*size/100/(['%','em'].includes(match[2])?rootFactor:1);element.style.setProperty('font-size',value+match[2],'important');}
  if(record.kind&&state().colorScores!==false){
   const colors=config.theme==='latte'?{cost:['#d20f39','#f7d6df'],gain:['#28731d','#d8edce'],set:['#1e66f5','#d5e6fa']}:{cost:['#f38ba8','#412636'],gain:['#a6e3a1','#25392c'],set:['#89b4fa','#253249']};
   const color=colors[record.kind];
   if(color){element.style.setProperty('color',color[0],'important');element.style.background=color[1];element.style.border=`1px solid ${color[0]}`;element.style.borderRadius='7px';element.style.padding='3px 8px';element.style.marginBlock='3px';}
  }
 }
 function style(element,css,part='body',kind){
  css=css||'';element.style.cssText=css;
  if(!css.includes('font-family:')&&!kind)return;
  const record={css,part,kind,font:element.style.fontFamily,size:element.style.fontSize};
  originals.set(element,record);element.dataset.managerPart=part;
  if(kind)element.dataset.managerScoreKind=kind;
  applyText(element,record);
 }
 let cardFrame=0;
 function applyCards(){
  const elements=[...document.querySelectorAll('#app [data-manager-card]')];
  for(const element of elements){
   let saved=cards.get(element);
   if(!saved){saved=['flex-basis','max-width','width'].map(key=>[key,element.style.getPropertyValue(key),element.style.getPropertyPriority(key)]);cards.set(element,saved);}
   for(const [key,value,priority] of saved){if(value)element.style.setProperty(key,value,priority);else element.style.removeProperty(key);}
  }
  const scale=(state().scale||100)/100;if(scale===1)return;
  // Batch reads before writes: a long CYOA must not reflow once per choice.
  const parents=new Map(), widths=elements.map(element=>{
   const parent=element.parentElement;if(!parent)return 0;
   if(!parents.has(parent))parents.set(parent,parent.clientWidth);
   const available=parents.get(parent),width=element.getBoundingClientRect().width/scale;
   return available?Math.min(100,width/available*100*scale):0;
  });
  elements.forEach((element,index)=>{if(widths[index])for(const key of ['flex-basis','max-width','width'])element.style.setProperty(key,widths[index]+'%','important');});
 }
 function positionDock(){
  const host=document.getElementById('manager-quick-tools'),bar=document.querySelector('.pointBar');if(!host||!bar)return;
  const rect=bar.getBoundingClientRect(),bottom=Math.abs(rect.bottom-innerHeight)<8?innerHeight-rect.top+8:8;
  host.style.setProperty('bottom',bottom+'px','important');host.shadowRoot.dispatchEvent(new Event('manager-dock-layout'));
 }
 function scheduleCards(){if(!cardFrame)cardFrame=requestAnimationFrame(()=>{cardFrame=0;applyCards();positionDock();});}
 function card(element){element.dataset.managerCard='';scheduleCards();}
 // Only infer a parent when every mandatory positive choice dependency belongs
 // to the same row. OR/group/negative/point rules remain entirely author-controlled.
 function rowVisible(row,check,owner,seen=new Set()){
  if((state().hiddenRows||[]).includes(row.id))return false;
  if(state().autoHideSections===false||seen.has(row.id)||seen.size>=64)return true;
  seen.add(row.id);
  const requirements=row.requireds||[];
  if(!requirements.length||requirements.some(r=>r.type!=='id'||r.required!==true||(r.more||[]).length))return true;
  const parents=requirements.map(r=>owner(r.reqId));
  if(parents.some(parent=>!parent)||new Set(parents.map(parent=>parent.id)).size!==1)return true;
  const parent=parents[0];if(parent.id===row.id)return true;
  return check(parent.requireds||[])&&rowVisible(parent,check,owner,seen);
 }
 function apply(){
  const app=document.getElementById('app');if(!app)return;
  const initial=document.getElementById('manager-initial-theme');
  if(initial?.textContent.includes('@font-face'))initial.textContent=initial.textContent.split('@font-face')[0];
  app.style.fontSize=(rootSize*(body().size||100)/100)+'px';app.style.zoom=String((state().scale||100)/100);
  for(const element of app.querySelectorAll('[data-manager-part]')){const record=originals.get(element);if(record)applyText(element,record);}
  if(window.debugApp)window.debugApp.managerSectionRevision=(window.debugApp.managerSectionRevision||0)+1;
  scheduleCards();
 }
 async function request(change){
  const response=await fetch('/__manager_reader_settings',{method:change?'POST':'GET',headers:{'Content-Type':'application/json'},body:change?JSON.stringify(change):undefined});
  const result=await response.json();if(!response.ok)throw new Error(result.error||'Could not save display settings');
  config=result;apply();return result;
 }
 async function reload(){try{await request();}catch(error){if(panel)panel.querySelector('[role=status]').textContent=error.message;}}
 function field(parent,label,control){const wrapper=document.createElement('label');wrapper.textContent=label;wrapper.append(control);parent.append(wrapper);return control;}
 function select(parent,label,values){const node=document.createElement('select');node.setAttribute('aria-label',label);for(const [value,text] of values){const option=new Option(text,value);node.add(option);}return field(parent,label,node);}
 async function open(){
  const cheats=document.getElementById('cyoa-manager-cheat-overlay')?.shadowRoot?.getElementById('cyoa-manager-viewer-panel');if(cheats)cheats.hidden=true;
  const shadow=window.__cyoaManagerDock();
  if(panel){panel.hidden=!panel.hidden;if(!panel.hidden){const controls=[...panel.querySelectorAll('input,select,button')];controls.forEach(n=>n.disabled=true);try{await request();window.__cyoaManagerPresentation.fill?.();}finally{controls.forEach(n=>n.disabled=false);}}return;}
  panel=document.createElement('section');panel.id='manager-display-panel';panel.setAttribute('role','dialog');panel.setAttribute('aria-label','Reader display');
  const heading=document.createElement('h2');heading.textContent='Reader display';panel.append(heading);
  const scope=select(panel,'Save to',[['project','This CYOA'],['global','Global defaults']]);scope.id='reader-scope';
  const section=select(panel,'Text section',Object.entries(partNames));section.id='reader-section';
  const font=document.createElement('input');font.id='reader-font';font.setAttribute('list','reader-fonts');font.placeholder='Author default';field(panel,'Font (empty = author default)',font);
  const list=document.createElement('datalist');list.id='reader-fonts';panel.append(list);
  const size=document.createElement('input');size.type='number';size.min=50;size.max=250;size.step=5;size.id='reader-size';field(panel,'Text size (% of authored size)',size);
  const scale=document.createElement('input');scale.type='number';scale.min=50;scale.max=150;scale.step=5;scale.id='reader-scale';field(panel,'Choice scale (%)',scale);
  const colors=document.createElement('input');colors.type='checkbox';colors.id='reader-score-colors';field(panel,'Distinct cost / gain colors',colors);
  const automatic=document.createElement('input');automatic.type='checkbox';automatic.id='reader-auto-sections';field(panel,'Hide dependent sections when their parent tab is closed',automatic);
  const sectionList=document.createElement('div');sectionList.style.maxHeight='240px';sectionList.style.overflowY='auto';panel.append(sectionList);
  const note=document.createElement('p');note.className='reader-note';panel.append(note);
  const status=document.createElement('p');status.setAttribute('role','status');panel.append(status);
  const actions=document.createElement('div');actions.className='reader-actions';panel.append(actions);
  function button(text,handler){const node=document.createElement('button');node.textContent=text;node.type='button';node.onclick=handler;actions.append(node);return node;}
  async function save(reset){
   const action={scope:scope.value,section:section.value,reset};
   if(!reset){if(section.value==='sections'){action.autoHideSections=automatic.checked;if(scope.value==='project')action.hiddenRows=[...sectionList.querySelectorAll('input')].filter(n=>!n.checked).map(n=>n.value);}else if(section.value==='layout'){action.scale=Number(scale.value);action.colorScores=colors.checked;}else{action.font=font.value;action.size=Number(size.value);}}
   const buttons=[...actions.querySelectorAll('button')];buttons.forEach(b=>b.disabled=true);
   try{await request(action);fill();status.textContent=reset?'Reset to inherited defaults.':'Saved and applied. Game choices are unchanged.';}catch(error){status.textContent=error.message;}
   finally{buttons.forEach(b=>b.disabled=false);}
  }
  button('Apply and save',()=>save(false));button('Reset section',()=>save(true));button('Close',()=>panel.hidden=true);
  function fill(){
   const isSections=section.value==='sections',isLayout=section.value==='layout', selected=scope.value==='global'?config.global:config.effective;
   const value=selected.parts?.[section.value]||selected.parts?.body||{font:'',size:100};
   font.value=value.font||'';size.value=value.size||100;scale.value=selected.scale||100;colors.checked=selected.colorScores!==false;
   font.parentElement.hidden=size.parentElement.hidden=isLayout||isSections;scale.parentElement.hidden=colors.parentElement.hidden=!isLayout;
   automatic.parentElement.hidden=sectionList.hidden=!isSections;automatic.checked=selected.autoHideSections!==false;
   sectionList.replaceChildren();
   if(isSections&&scope.value==='project')for(const row of window.debugApp?.rows||[]){
    const label=document.createElement('textarea');label.innerHTML=(row.title||'').replace(/<[^>]*>/g,'');const title=label.value.trim();if(!title)continue;
    const toggle=document.createElement('input');toggle.type='checkbox';toggle.value=row.id;toggle.checked=!(config.local.hiddenRows||[]).includes(row.id);field(sectionList,title,toggle);
   }
   const sample=[...document.querySelectorAll('[data-manager-part]')].find(node=>node.dataset.managerPart===section.value), original=sample&&originals.get(sample);
   const pinned=scope.value==='project'&&(isLayout?config.local.scale!=null:!!config.local.parts?.[section.value]);
   note.textContent=isSections?'Checked sections remain visible when their authored requirements allow. Uncheck to hide; selections and points are preserved. Automatic hiding only follows unambiguous parent dependencies.':isLayout?'Smaller values fit more cards where the authored grid allows. Artwork and the CYOA scale together; game rules stay unchanged.':`${pinned?'Pinned for this CYOA until reset.':'Using inherited settings until you save an override.'} 100% preserves each authored size.${original?' Example authored style: '+original.font+' / '+original.size+'.':''}`;
  }
  window.__cyoaManagerPresentation.fill=fill;
  scope.onchange=section.onchange=()=>{fill();status.textContent='';};
  panel.addEventListener('keydown',event=>{if(event.key==='Escape')panel.hidden=true;});shadow.append(panel);
  const controls=[...panel.querySelectorAll('input,select,button')];controls.forEach(n=>n.disabled=true);status.textContent='Loading display settings…';
  try{await request();fill();const fonts=await(await fetch('/__manager_reader_fonts')).json();for(const family of fonts){const item=new Option(family,family);list.append(item);}status.textContent='';}catch(error){status.textContent=error.message;}finally{controls.forEach(n=>n.disabled=false);}
 }
 window.__cyoaManagerPresentation={style,card,rowVisible,reload,open,close:()=>{if(panel)panel.hidden=true;}};
 let resizeTimer;window.addEventListener('resize',()=>{clearTimeout(resizeTimer);resizeTimer=setTimeout(apply,150);});
 window.addEventListener('pagehide',()=>{clearTimeout(resizeTimer);cancelAnimationFrame(cardFrame);}, {once:true});
 function start(){const app=document.getElementById('app');if(app)rootSize=parseFloat(getComputedStyle(app).fontSize)||16;reload();}
 if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',start,{once:true});else start();
})();
