const {chromium}=require('playwright'),assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true});try{
 const page=await browser.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.route('https://api.github.com/**',r=>r.fulfill({status:404,body:'{}'}));
 await page.addInitScript(()=>{
  const callbacks=new Map();let id=0;window.calls=[];window.library=[];window.failBeta=true;
  const rows=[['Title','Interactive','Engine'],['Alpha','https://queue.test/alpha/','?'],['Beta','https://queue.test/beta/','?']];
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks.set(++id,fn);return id;},unregisterCallback:()=>{},invoke:async(command,args)=>{
   window.calls.push({command,args});if(command==='plugin:event|listen')return id;
   if(command==='get_library')return structuredClone(window.library);
   if(command==='get_catalog_snapshot')return {sources:[{id:'beta',name:'Beta Index',rows,fetchedAt:'2026-10-16T12:00:00Z',stale:false,error:null}],fetchedAt:'2026-10-16T12:00:00Z',stale:false,warnings:[]};
   if(command==='get_viewers')return [{id:'icc2-plus',name:'ICC2 Plus'}];if(command==='get_fonts')return ['system-ui'];if(command==='get_author_aliases')return [];
   if(command==='inspect_catalog_download')return {mode:'project',reason:'Inspected project',sourceUrl:args.websiteUrl};
   if(command==='import_catalog_download'){
    if(args.projectName==='Alpha')await new Promise(resolve=>window.finishAlpha=resolve);
    if(args.projectName==='Beta'&&window.failBeta){window.failBeta=false;throw 'Transient failure';}
    const p={id:args.projectName,name:args.projectName,title:args.projectName,source_url:args.websiteUrl,author:'Author',fandom:'World',description:'Project metadata',file_path:'/fixture/'+args.projectName+'.json',viewer_preference:'icc2-plus',favorite:false,file_missing:false,tags:[],date_added:'2026-10-16'};window.library.push(p);return p;
   }
   return null;
  }};
 });
 await page.goto((process.env.CYOA_TEST_URL||'http://127.0.0.1:5797')+'/#/catalog');await page.locator('.catalog-view .card').first().waitFor();
 await page.getByRole('checkbox',{name:'Select all 2 results',exact:true}).check();await page.getByRole('button',{name:'Download Selected (2)',exact:true}).click();
 await page.waitForFunction(()=>typeof window.finishAlpha==='function');await page.getByRole('button',{name:'Stop after current',exact:true}).click();await page.evaluate(()=>window.finishAlpha());
 await page.getByText('Queue stopped; remaining entries stay selected.',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='import_catalog_download').length),1);
 await page.getByRole('textbox',{name:'Search catalog'}).fill('nothing-matches');await page.getByRole('button',{name:'Download Selected (1)',exact:true}).click();
 await page.getByRole('button',{name:/Retry failed/}).waitFor();await page.getByRole('button',{name:/Retry failed/}).click();
 await page.getByText('Batch complete',{exact:true}).waitFor();
 assert.deepEqual(await page.evaluate(()=>window.calls.filter(c=>c.command==='import_catalog_download').map(c=>c.args.projectName)),['Alpha','Beta','Beta']);
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='download_website'||c.command==='start_overwrite_catalog_entry').length),0);assert.deepEqual(errors,[]);
 console.log('Catalog queue: content inspected despite unknown engine; stop keeps pending selection; hidden selected entry retries; successful import never repeated.');
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
