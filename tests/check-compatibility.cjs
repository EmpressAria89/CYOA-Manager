const {chromium}=require('playwright'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});
 try {
  const page=await browser.newPage({viewport:{width:1000,height:900}}),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.route('https://api.github.com/**',r=>r.fulfill({status:404,body:'{}'}));
  await page.addInitScript(()=>{
   const callbacks=new Map();let id=0;window.calls=[];
   const project={id:'test',name:'Remix fixture',title:'Remix fixture',author:'Test',file_path:'/fixture/project.json',viewer_preference:'icc2-plus',description:'',tags:[],date_added:'2026-10-16',source_url:'https://example.org/game/',favorite:false,file_missing:false};
   window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
   window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks.set(++id,fn);return id;},unregisterCallback:()=>{},invoke:async(command,args)=>{
    window.calls.push({command,args});
    if(command==='plugin:event|listen')return id;
    if(command==='get_library')return [project];
    if(command==='get_viewers')return [{id:'icc2-plus',name:'ICC2 Plus'}];
    if(command==='get_fonts')return ['system-ui'];
    if(command==='get_author_aliases')return [];
    if(command==='open_viewer_window'&&!args.allowIncompatible)throw 'VIEWER_COMPATIBILITY|'+JSON.stringify({sourceUrl:project.source_url,report:{editor:'iccremix',scripts:5,variable_operations:3}});
    if(command==='download_website')return {unavailable:['https://example.org/dynamic']};
    return null;
   }};
  });
  await page.goto((process.env.CYOA_TEST_URL||'http://127.0.0.1:5797')+'/#/');
  await page.locator('.card').getByRole('button',{name:'Open',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Use this CYOA’s original viewer'});await dialog.waitFor();
  assert.match(await dialog.textContent(),/5 script expressions/);
  assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='set_project_viewer_preference').length),0);
  assert.equal(await dialog.getByRole('button',{name:'Open source website'}).evaluate(e=>document.activeElement===e),true);
  await dialog.getByRole('button',{name:'Open source website'}).click();
  assert.equal(await page.evaluate(()=>window.calls.find(c=>c.command==='plugin:opener|open_url').args.url),'https://example.org/game/');
  await page.locator('.card').getByRole('button',{name:'Open',exact:true}).click();
  await dialog.getByRole('button',{name:'Save website copy'}).click();await dialog.getByRole('status').waitFor();
  const saved=await page.evaluate(()=>window.calls.find(c=>c.command==='download_website').args);
  assert.equal(saved.existingProjectId,null);assert.equal(saved.title,'Remix fixture — website copy');
  assert.match(await dialog.textContent(),/1 resources could not be saved/);
  await dialog.getByRole('button',{name:'Try bundled viewer anyway'}).click();
  await page.waitForFunction(()=>window.calls.some(c=>c.command==='open_viewer_window'&&c.args.allowIncompatible===true));
  assert.equal(await dialog.count(),0);assert.deepEqual(errors,[]);
  console.log('Compatibility: warning, safe source action, separate website copy, missing-resource notice and explicit bypass passed.');
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
