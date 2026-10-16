const {chromium}=require('playwright'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});
 try{
 const page=await browser.newPage({viewport:{width:1100,height:900}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.route('https://api.github.com/**',r=>r.fulfill({status:404,body:'{}'}));
 await page.route('**/zip_link_catalog_data.js',r=>r.fulfill({body:'window.CATALOG_DATA = '+JSON.stringify([{name:'Story',date:'2020-01-01',website:'https://catalog.test/story/',link:'https://archive.test/story.zip'}])+';'}));
 await page.addInitScript(()=>{
  const callbacks=new Map();let id=0;window.calls=[];window.planMode='project';
  const project={id:'current',name:'Story',title:'Story',author:'Personal author',source_url:'https://catalog.test/story/',file_path:'/fixture/story.json',viewer_preference:'icc2-plus',tags:[],description:'Personal notes',favorite:false,file_missing:false,date_added:'2026-10-16'};
  window.liveProjects=[project];
  const rows=[['Title','Interactive','Engine','Author','Universe','Updated','Status','Alias'],['Story','https://catalog.test/story/','ICC','Source author','World','2026-10-16','MOD','Other Story']];
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks.set(++id,fn);return id;},unregisterCallback:()=>{},invoke:async(command,args)=>{
   window.calls.push({command,args});
   if(command==='plugin:event|listen')return id;
   if(command==='get_library')return structuredClone(window.liveProjects);
   if(command==='get_catalog_snapshot')return {sources:[{id:'beta',name:'Beta Index',rows,fetchedAt:'2026-10-16T10:00:00Z',stale:false,error:null}],fetchedAt:'2026-10-16T10:00:00Z',stale:false,warnings:[]};
   if(command==='get_catalog_rows')return rows;
   if(command==='get_viewers')return [{id:'icc2-plus',name:'ICC2 Plus'}];
   if(command==='get_fonts')return ['system-ui'];if(command==='get_author_aliases')return [];
   if(command==='inspect_catalog_download')return {mode:window.planMode,sourceUrl:args.websiteUrl,projectUrl:args.websiteUrl,reason:window.planMode==='project'?'Project structure inspected; compatibility checked.':'Original viewer required for scripts.',compatibility:window.planMode==='website'?{editor:'iccremix',scripts:3,variable_operations:2}:null};
   if(command==='preview_catalog_update')return {token:'review-token',projectName:'Story',sourceUrl:args.websiteUrl,diff:{changed:true,added:['New choice'],removed:[],edited:['Changed choice'],other_changes:[]}};
   if(command==='apply_catalog_update') {
    if(window.rejectApply) { window.rejectApply=false; throw new Error('Update preview expired; inspect again.'); }
    return project;
   }
   if(command==='import_catalog_download')return {...project,id:'recovered',author:'',fandom:'',description:''};
   if(command==='download_website')return {project:{...project,id:'website-copy'},unavailable:['https://catalog.test/dynamic']};
   return null;
  }};
 });
 await page.goto((process.env.CYOA_TEST_URL||'http://127.0.0.1:5797')+'/#/catalog');
 await page.locator('.catalog-view .card').filter({hasText:'Story'}).getByRole('button',{name:'Check for updates',exact:true}).click();
 const dialog=page.getByRole('dialog',{name:'Story — Download options'});await dialog.waitFor();
 await dialog.getByText('Project structure inspected; compatibility checked.').waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='apply_catalog_update'||c.command==='start_overwrite_catalog_entry').length),0);
 await dialog.getByRole('button',{name:'Preview update',exact:true}).click();
 await dialog.getByText('New choice',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='apply_catalog_update').length),0);
 await dialog.getByRole('button',{name:'Close',exact:true}).click();
 await page.waitForFunction(()=>window.calls.some(c=>c.command==='discard_catalog_update'&&c.args.token==='review-token'));
 assert.match(await page.evaluate(()=>document.activeElement?.textContent||''),/Check for updates/);
 await page.locator('.catalog-view .card').getByRole('button',{name:'Check for updates',exact:true}).click();await dialog.getByRole('button',{name:'Preview update',exact:true}).click();
 await page.evaluate(()=>window.rejectApply=true);
 await dialog.getByRole('button',{name:'Apply reviewed update',exact:true}).click();
 await dialog.getByRole('alert').filter({hasText:'Update preview expired; inspect again.'}).waitFor();
 await dialog.getByRole('button',{name:'Preview update',exact:true}).waitFor({timeout:3000});
 assert.equal(await dialog.getByRole('button',{name:'Apply reviewed update',exact:true}).count(),0,'consumed preview cannot be applied again');
 const previousPreviews=await page.evaluate(()=>window.calls.filter(c=>c.command==='preview_catalog_update').length);
 await dialog.getByRole('button',{name:'Preview update',exact:true}).click();
 await dialog.getByText('New choice',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='preview_catalog_update').length),previousPreviews+1,'recovery obtains a fresh comparison');
 await dialog.getByRole('button',{name:'Apply reviewed update',exact:true}).click();
 await dialog.getByText('Reviewed update applied. The previous edition is preserved.',{exact:true}).waitFor();
 await page.waitForFunction(()=>window.calls.some(c=>c.command==='apply_catalog_update'&&c.args.token==='review-token'));
 await dialog.getByRole('button',{name:'Close',exact:true}).click();
 await page.evaluate(()=>window.planMode='website');
 await page.locator('.catalog-view .card').getByRole('button',{name:'Check for updates',exact:true}).click();await dialog.getByText('Original viewer required for scripts.').waitFor();
 assert.equal(await dialog.getByRole('button',{name:'Preview update',exact:true}).count(),0);
 await dialog.getByRole('button',{name:'Save separate website copy',exact:true}).click();
 await dialog.getByText(/1 resources could not be saved/).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.find(c=>c.command==='download_website').args.existingProjectId),null);
 await dialog.getByRole('button',{name:'Close',exact:true}).click();
 await page.evaluate(()=>{window.planMode='project';window.liveProjects.push({...window.liveProjects[0],id:'other',name:'Other edition',title:'Other edition'});});
 // A website-copy completion refreshes library membership before the next dialog.
 await page.locator('.catalog-view .card').getByRole('button',{name:'Check for updates',exact:true}).click();await dialog.getByRole('button',{name:'Save separate website copy',exact:true}).click();await dialog.getByText(/resources could not be saved/).waitFor();await dialog.getByRole('button',{name:'Close',exact:true}).click();
 await page.locator('.catalog-view .card').getByRole('button',{name:'Check for updates',exact:true}).click();
 const editions=dialog.getByLabel('Library edition to update');await editions.waitFor();assert.equal(await editions.inputValue(),'');
 await editions.selectOption('other');await dialog.getByRole('button',{name:'Preview update',exact:true}).click();await dialog.getByText('New choice',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='preview_catalog_update').at(-1).args.projectId),'other');
 await dialog.getByRole('button',{name:'Close',exact:true}).click();
 await page.locator('.catalog-view .card').getByRole('button',{name:'Check for updates',exact:true}).click();await dialog.getByText('Recovery links — may be older or incomplete',{exact:true}).click();
 await dialog.getByRole('button',{name:'Download archive as separate copy',exact:true}).click();await dialog.getByText(/Added a separate recovery copy/).waitFor();
 const recovered=await page.evaluate(()=>window.calls.find(c=>c.command==='import_catalog_download').args);assert.equal(recovered.allowArchive,true);assert.equal(recovered.archiveUrl,'https://archive.test/story.zip');
 const metadata=await page.evaluate(()=>window.calls.find(c=>c.command==='update_project').args.patch);assert.equal(metadata.author,'Source author');assert.equal(metadata.fandom,'World');assert.equal(metadata.is_mod,true);assert.equal(metadata.modder,undefined);assert.equal(metadata.completion,undefined);
 assert.deepEqual(errors,[]);console.log('Catalog download actions: actual-content plan, preview before apply, discard, explicit apply, native website copy stays separate.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
