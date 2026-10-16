const {chromium}=require('playwright'),fs=require('fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});const p=await browser.newPage({viewport:{width:1440,height:950}});const errors=[];p.on('pageerror',e=>errors.push(e.message));
 let global={parts:{body:{font:'',size:100}},scale:100,colorScores:true},local={parts:{}};
 const response=()=>({global,local,effective:{...global,...local,scale:local.scale??global.scale,colorScores:local.colorScores??global.colorScores,parts:{...global.parts,...local.parts}}});
 await p.route('http://reader.test/**',async route=>{
  const path=new URL(route.request().url()).pathname;
  if(path==='/__manager_reader_settings'){
   if(route.request().method()==='POST'){
    const c=route.request().postDataJSON(), target=c.scope==='project'?local:global;
    if(c.section==='sections'){if(c.reset){delete target.hiddenRows;delete target.autoHideSections;}else{target.autoHideSections=c.autoHideSections;target.hiddenRows=c.hiddenRows;}}else if(c.section==='layout'){if(c.reset){delete target.scale;delete target.colorScores;}else{target.scale=c.scale;target.colorScores=c.colorScores;}}
    else if(c.reset)delete target.parts[c.section];else target.parts[c.section]={font:c.font,size:c.size};
   }
   return route.fulfill({json:response()});
  }
  if(path==='/__manager_reader_fonts')return route.fulfill({json:['serif','sans-serif','monospace']});
  if(path==='/__manager_session')return route.fulfill({json:{theme:'mocha',cheatsEnabled:true}});
  if(path==='/__cyoa_manager_viewer_overlay.html')return route.fulfill({contentType:'text/html',body:fs.readFileSync('src-tauri/src/viewer_overlay.html','utf8')});
  return route.fulfill({contentType:'text/html',body:'<html><head><style>body{margin:20px;background:#181825;color:#cdd6f4;font:16px system-ui}#app{font-size:16px}.row{display:flex;flex-wrap:wrap}.card{flex:0 0 25%;max-width:25%;padding:8px;box-sizing:border-box}.inside{border:1px solid #585b70;border-radius:12px;padding:20px;min-height:240px;background:#1e1e2e}button{font-size:36px!important}h2{font-weight:400}img{width:100%}</style></head><body><div id="app"><h1>Reader controls preview</h1><p>Display settings preserve authored proportions and game choices.</p><div class="row"></div><div style="height:1400px"></div></div><div style="position:fixed;bottom:0;left:0;right:0;height:42px;background:#313244;text-align:center;padding-top:10px">Potential: 100</div></body></html>'});
 });
 await p.goto('http://reader.test/');
 await p.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_presentation.js','utf8')});
 await p.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_tools.js','utf8')});
 await p.evaluate(()=>{
  window.__cyoaManagerNative={capture:()=>window.choiceCode||'a',load:code=>window.choiceCode=code,ready:()=>true,setRevealHidden:value=>window.reveal=value};
  window.debugApp={pointTypes:[{name:'Potential',startingSum:100}],rows:[{id:'race',title:'<p>Races</p>',objects:[],allowedChoices:3}]};
  const row=document.querySelector('.row');
  for(let i=0;i<12;i++){
   const card=document.createElement('article');card.className='card';card.innerHTML='<div class="inside"><h2>Choice '+(i+1)+'</h2><p class="description">An authored description with its own typography.</p><p class="score">'+(i%2?'Gain: 25 Potential':'Cost: 10 Potential')+'</p></div>';row.append(card);
   __cyoaManagerPresentation.card(card);
   __cyoaManagerPresentation.style(card.querySelector('h2'),'font-family:serif;font-size:150%;','title');
   __cyoaManagerPresentation.style(card.querySelector('.description'),'font-family:sans-serif;font-size:100%;','description');
   __cyoaManagerPresentation.style(card.querySelector('.score'),'font-family:serif;font-size:90%;','score',i%2?'gain':'cost');
  }
 });
 await p.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_builds.js','utf8')});
 await p.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_overlay.js','utf8')});
 await p.getByRole('button',{name:'Cheats',exact:true}).waitFor();
 const initial=await p.locator('.card').first().boundingBox();
 const cost=await p.locator('.score').nth(0).evaluate(e=>getComputedStyle(e).color),gain=await p.locator('.score').nth(1).evaluate(e=>getComputedStyle(e).color);assert.notEqual(cost,gain);
 await p.getByRole('button',{name:'Go to bottom',exact:true}).click();await p.waitForTimeout(650);assert(await p.evaluate(()=>scrollY>1000));await p.getByRole('button',{name:'Go to top',exact:true}).click();await p.waitForTimeout(650);assert(await p.evaluate(()=>scrollY<5));
 await p.getByRole('button',{name:'Display',exact:true}).click();const panel=p.getByRole('dialog',{name:'Reader display'});
 assert.equal(await panel.evaluate(e=>getComputedStyle(e).color),'rgb(205, 214, 244)');
 await panel.getByLabel('Text section',{exact:true}).selectOption('title');await panel.getByLabel('Font (empty = author default)',{exact:true}).fill('monospace');await panel.getByLabel('Text size (% of authored size)',{exact:true}).fill('125');await panel.getByRole('button',{name:'Apply and save',exact:true}).click();await p.waitForTimeout(100);
 const title=await p.locator('.card h2').first().evaluate(e=>({size:getComputedStyle(e).fontSize,font:getComputedStyle(e).fontFamily}));assert.equal(title.size,'30px');assert.match(title.font,/Manager Reader/);assert.equal(await p.locator('.description').first().evaluate(e=>getComputedStyle(e).fontSize),'16px');
 await panel.getByLabel('Save to',{exact:true}).selectOption('global');await panel.getByLabel('Text size (% of authored size)',{exact:true}).fill('175');await panel.getByRole('button',{name:'Apply and save',exact:true}).click();await p.waitForTimeout(100);assert.equal(await p.locator('.card h2').first().evaluate(e=>getComputedStyle(e).fontSize),'30px');
 await panel.getByLabel('Save to',{exact:true}).selectOption('project');await panel.getByRole('button',{name:'Reset section',exact:true}).click();await p.waitForTimeout(100);assert.equal(await p.locator('.card h2').first().evaluate(e=>getComputedStyle(e).fontSize),'42px');
 await panel.getByLabel('Text section',{exact:true}).selectOption('layout');await panel.getByLabel('Choice scale (%)',{exact:true}).fill('70');await panel.getByRole('button',{name:'Apply and save',exact:true}).click();await p.waitForTimeout(150);
 const resized=await p.locator('.card').first().boundingBox();assert(resized.width<initial.width*.8,JSON.stringify({initial,resized}));assert.equal(await p.evaluate(()=>__cyoaManagerNative.capture()),'a');
 await panel.getByLabel('Text section',{exact:true}).selectOption('sections');await panel.getByLabel('Races',{exact:true}).uncheck();await panel.getByRole('button',{name:'Apply and save',exact:true}).click();await p.waitForTimeout(100);assert.deepEqual(local.hiddenRows,['race']);assert.equal(await p.evaluate(()=>__cyoaManagerPresentation.rowVisible({id:'race'},()=>true,()=>null)),false);assert.equal(await p.evaluate(()=>__cyoaManagerNative.capture()),'a');await panel.getByRole('button',{name:'Reset section',exact:true}).click();await p.waitForTimeout(100);assert.equal(await p.evaluate(()=>__cyoaManagerPresentation.rowVisible({id:'race'},()=>true,()=>null)),true);
 await panel.getByRole('button',{name:'Close',exact:true}).click();await p.getByRole('button',{name:'Cheats',exact:true}).click();const cheats=p.locator('#cyoa-manager-viewer-panel');await cheats.waitFor({state:'visible'});assert((await cheats.boundingBox()).width>=400);
 const buttons=await p.locator('#manager-quick-tools').locator('button').evaluateAll(bs=>bs.filter(e=>e.getClientRects().length).map(e=>e.getBoundingClientRect().height));assert(buttons.every(height=>height===40),JSON.stringify(buttons));
 const output=process.env.CYOA_SCREENSHOT_DIR||'tests/results';fs.mkdirSync(output,{recursive:true});await cheats.screenshot({path:output+'/reader-cheat-menu.png'});await p.getByRole('button',{name:'Cheats',exact:true}).click();await p.locator('#manager-quick-tools').screenshot({path:output+'/reader-bottom-controls.png'});
 await p.getByRole('button',{name:'Display',exact:true}).click();await p.waitForFunction(()=>!document.getElementById('manager-quick-tools').shadowRoot.getElementById('manager-display-panel').querySelector('button').disabled);await p.screenshot({path:output+'/reader-display-settings.png'});
 assert.deepEqual(errors,[]);await browser.close();console.log('Reader: semantic cost/gain colors, navigation, consistent buttons, live section typography, pinned overrides/reset, scaling, expanded cheats and screenshots passed.');
})().catch(e=>{console.error(e);process.exit(1)});
